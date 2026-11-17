// =============================================================================
// main.rs — Employee MCP Client
//
// Two modes (selected by CLI args):
//   --server [--port PORT]  → Axum HTTP server on PORT (default 5001)
//   (no args)               → Interactive terminal CLI
//
// Architecture:
//   User Prompt → Regex Intent Detector → Parameter Extractor →
//   (Confirmation if write op) → MCP Tool Router → .NET MCP Server (STDIO)
//   → MySQL → Result Display
//
// NO LLM. NO AI. Pure Regex.
// =============================================================================

use employee_mcp_client::{
    config::Config,
    confirmation::request_confirmation,
    context::{
        ResolvedContextAction, clear_session_context, get_or_create_context, resolve_context,
        update_context,
    },
    intent::detector::detect_intent,
    intent::patterns::{RE_EXIT, RE_HELP},
    mcp::client::McpClient,
    models::{Intent, IntentResult, ToolExecutionResult},
    planner::{create_execution_plan, execute_plan, project_fields, validate_plan},
    response::formatter::format_mcp_data,
    server::{AppState, build_router},
    suggestions::generate_suggestions,
};

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All tracing goes to STDERR — STDOUT is for user-facing output only
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("employee_mcp_client=info".parse()?)
                .add_directive("warn".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let server_mode = args.contains(&"--server".to_string());
    let port: u16 = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(5001);

    let config = Config::from_env();

    eprintln!("  Starting .NET MCP Server...");
    let client = match McpClient::connect(&config).await {
        Ok(c) => {
            eprintln!(
                "  ✓ MCP Server connected. {} tools discovered.",
                c.discovered_tools.len()
            );
            eprintln!("  ✓ Tools: {}", c.tool_names().join(", "));
            c
        }
        Err(e) => {
            eprintln!("  ✗ Failed to connect to MCP Server: {}", e);
            eprintln!(
                "  Make sure you have run 'dotnet build' in dotnet-mcp-server/EmployeeMcpServer"
            );
            eprintln!("  and that your MySQL connection string in appsettings.json is correct.");
            return Err(e.into());
        }
    };

    if server_mode {
        // -----------------------------------------------------------------------
        // HTTP Server mode — Axum on :PORT
        // -----------------------------------------------------------------------
        let state = Arc::new(AppState {
            client: Mutex::new(client),
        });
        let app = build_router(state);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        eprintln!("  ✓ Rust HTTP service listening on http://0.0.0.0:{}", port);
        axum::serve(listener, app).await?;
        return Ok(());
    }

    // -------------------------------------------------------------------------
    // Interactive CLI mode with Context, Planner, and Suggestions
    // -------------------------------------------------------------------------
    let mut client = client;
    let cli_session = "cli_session";

    let stdin = io::stdin();
    loop {
        print!("\n> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let raw_prompt = line.trim().to_string();
        if raw_prompt.is_empty() {
            continue;
        }

        // Exit commands
        if RE_EXIT.is_match(&raw_prompt) {
            println!("Goodbye!");
            break;
        }

        // Help command
        if RE_HELP.is_match(&raw_prompt) {
            print_help();
            continue;
        }

        let mut context = get_or_create_context(cli_session);

        // Stage 1: Context Resolution
        let context_action = resolve_context(&raw_prompt, &context);
        let prompt = match context_action {
            ResolvedContextAction::ContextCleared => {
                clear_session_context(cli_session);
                println!("  ✓ Conversation memory has been cleared.");
                continue;
            }
            ResolvedContextAction::InMemoryProjection { requested_fields } => {
                if let Some(ref data) = context.last_result_data {
                    let projected = project_fields(data, &requested_fields);
                    println!("{}", format_mcp_data(&projected));
                    context.last_requested_fields = requested_fields;
                    update_context(cli_session, context);
                    continue;
                }
                raw_prompt
            }
            ResolvedContextAction::EmployeeFollowUp {
                rewritten_prompt, ..
            } => {
                println!("  [Context Resolved] → {}", rewritten_prompt);
                rewritten_prompt
            }
            ResolvedContextAction::DepartmentSwitch {
                rewritten_prompt, ..
            } => {
                println!("  [Context Resolved] → {}", rewritten_prompt);
                rewritten_prompt
            }
            ResolvedContextAction::None => raw_prompt,
        };

        // Stage 2: Intent Detection
        let intent_result = detect_intent(&prompt);

        if matches!(intent_result.intent, Intent::Unknown(_)) {
            let suggestions_out = generate_suggestions(&prompt, &intent_result, &context);
            println!("  ✗ {}", suggestions_out.explanation);
            if !suggestions_out.suggestions.is_empty() {
                println!("    Suggested prompts:");
                for s in suggestions_out.suggestions {
                    println!("      • {}", s);
                }
            }
            context.record_failure();
            update_context(cli_session, context);
            continue;
        }

        println!("  → Intent : {:?}", intent_result.intent);

        // Stage 3: Task Planning & Validation
        let plan = create_execution_plan(&prompt, &intent_result);
        let validation = validate_plan(&plan, &intent_result, &client.discovered_tools);
        if !validation.is_valid {
            println!("  ✗ Plan validation failed:");
            for err in validation.errors {
                println!("    - {}", err);
            }
            let suggestions_out = generate_suggestions(&prompt, &intent_result, &context);
            if !suggestions_out.suggestions.is_empty() {
                println!("    Suggested prompts:");
                for s in suggestions_out.suggestions {
                    println!("      • {}", s);
                }
            }
            context.record_failure();
            update_context(cli_session, context);
            continue;
        }

        // Confirmation gate for write operations
        if plan.requires_confirmation {
            let confirmed = request_confirmation(&intent_result);
            if !confirmed {
                println!("  ✗ Operation cancelled.");
                continue;
            }
        }

        // Stage 4: Execution Engine
        println!("  → Executing task plan ({} steps)...", plan.steps.len());
        match execute_plan(&plan, &mut client).await {
            Ok(outcome) => {
                if outcome.success {
                    if let Some(ref data) = outcome.data {
                        println!("{}", format_mcp_data(data));
                    } else if let Some(ref msg) = outcome.message {
                        println!("  ✓ {}", msg);
                    }

                    // Update context
                    match intent_result.intent {
                        Intent::GetEmployeeById => {
                            let emp_id = plan.extracted_conditions.employee_id.unwrap_or(0);
                            let name = outcome.data.as_ref().and_then(|d| {
                                if let (Some(f), Some(l)) = (
                                    d.get("firstName").and_then(|v| v.as_str()),
                                    d.get("lastName").and_then(|v| v.as_str()),
                                ) {
                                    Some(format!("{} {}", f, l))
                                } else {
                                    None
                                }
                            });
                            let email = outcome
                                .data
                                .as_ref()
                                .and_then(|d| d.get("email").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());
                            let dept = outcome
                                .data
                                .as_ref()
                                .and_then(|d| d.get("departmentName").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());
                            let desig = outcome
                                .data
                                .as_ref()
                                .and_then(|d| d.get("designationName").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());
                            let salary = outcome
                                .data
                                .as_ref()
                                .and_then(|d| d.get("salary").and_then(|v| v.as_f64()));
                            let status = outcome
                                .data
                                .as_ref()
                                .and_then(|d| d.get("status").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());

                            context.record_employee_lookup(
                                emp_id,
                                name,
                                email,
                                dept,
                                desig,
                                salary,
                                status,
                                outcome.data.clone(),
                            );
                        }
                        Intent::SearchEmployees | Intent::GetAllEmployees => {
                            context.record_search(
                                plan.extracted_conditions.department.clone(),
                                plan.extracted_conditions.designation.clone(),
                                plan.extracted_conditions.status.clone(),
                                outcome.data.clone(),
                                outcome.matching_count,
                            );
                        }
                        _ => {}
                    }
                    update_context(cli_session, context);
                } else {
                    println!(
                        "  ✗ {}",
                        outcome
                            .message
                            .unwrap_or_else(|| "Operation failed".to_string())
                    );
                    context.record_failure();
                    update_context(cli_session, context);
                }
            }
            Err(e) => {
                println!("  ✗ Plan execution error: {}", e);
                context.record_failure();
                update_context(cli_session, context);
            }
        }
    }

    client.shutdown().await;
    Ok(())
}

// =============================================================================
// Build MCP tool arguments from extracted parameters
// Returns Err with a helpful message if required params are missing.
// =============================================================================
#[allow(dead_code)]
fn build_tool_args(intent: &IntentResult) -> Result<Value, String> {
    let p = &intent.params;
    match &intent.intent {
        Intent::GetEmployeeById => {
            let id = p
                .employee_id
                .ok_or("Employee ID is required. Example: 'Find employee 1001'")?;
            Ok(json!({ "employeeId": id }))
        }

        Intent::SearchEmployees => {
            let mut args = serde_json::Map::new();
            if let Some(ref dept) = p.department_name {
                args.insert("departmentName".to_string(), json!(dept));
            }
            if let Some(ref desig) = p.designation_name {
                args.insert("designationName".to_string(), json!(desig));
            }
            if let Some(ref first) = p.first_name {
                args.insert("firstName".to_string(), json!(first));
            }
            if let Some(ref last) = p.last_name {
                args.insert("lastName".to_string(), json!(last));
            }
            if let Some(ref email) = p.email {
                args.insert("email".to_string(), json!(email));
            }
            if let Some(ref status) = p.status {
                args.insert("status".to_string(), json!(status));
            }
            if args.is_empty() {
                return Err(
                    "Please specify a filter. Example: 'Search employees in IT' or 'Find John'"
                        .to_string(),
                );
            }
            Ok(Value::Object(args))
        }

        Intent::GetEmployeeCount => {
            let mut args = serde_json::Map::new();
            if let Some(ref dept) = p.department_name {
                args.insert("department".to_string(), json!(dept));
            }
            if let Some(ref status) = p.status {
                args.insert("status".to_string(), json!(status));
            }
            Ok(Value::Object(args))
        }

        Intent::AddEmployee => {
            let first = p
                .first_name
                .as_deref()
                .ok_or("First name is required for adding an employee.")?;
            let last = p
                .last_name
                .as_deref()
                .ok_or("Last name is required for adding an employee.")?;
            let email = p
                .email
                .as_deref()
                .ok_or("Email is required. Example: john.smith@example.com")?;

            let args = json!({
                "firstName": first,
                "lastName": last,
                "email": email,
                "salary": p.salary.unwrap_or(0.0),
                "hireDate": p.hire_date.as_deref().unwrap_or("2024-01-01"),
                "status": p.status.as_deref().unwrap_or("Active"),
                // Note: departmentId and designationId require prior lookup.
                // For now we pass 0 — the .NET server will return a validation error
                // prompting the user to specify valid IDs.
                "departmentId": 0,
                "designationId": 0,
            });

            // If names were provided, the user should also specify department/designation.
            // We surface a helpful error rather than silently passing 0.
            if p.department_name.is_some() || p.designation_name.is_some() {
                println!("  ℹ  Department/Designation specified by name.");
                println!(
                    "     The .NET server requires IDs. Use 'show departments' and 'show designations' first."
                );
                println!("     Then re-run: add employee ... departmentId 1 designationId 1");
            }

            Ok(args)
        }

        Intent::UpdateEmployee => {
            let id = p.employee_id.ok_or(
                "Employee ID is required. Example: 'Update employee 1001 status to Inactive'",
            )?;

            let mut args = serde_json::Map::new();
            args.insert("employeeId".to_string(), json!(id));

            if let Some(ref f) = p.first_name {
                args.insert("firstName".to_string(), json!(f));
            }
            if let Some(ref l) = p.last_name {
                args.insert("lastName".to_string(), json!(l));
            }
            if let Some(ref e) = p.email {
                args.insert("email".to_string(), json!(e));
            }
            if let Some(ref ph) = p.phone {
                args.insert("phone".to_string(), json!(ph));
            }
            if let Some(ref st) = p.status {
                args.insert("status".to_string(), json!(st));
            }
            if let Some(salary) = p.salary {
                args.insert("salary".to_string(), json!(salary));
            }
            if let Some(ref date) = p.hire_date {
                args.insert("hireDate".to_string(), json!(date));
            }

            if args.len() == 1 {
                return Err(
                    "Please specify what to update. Example: 'Update employee 1001 status to Inactive'".to_string()
                );
            }

            Ok(Value::Object(args))
        }

        // Read-only tools with no required args
        Intent::GetAllEmployees
        | Intent::GetDepartmentList
        | Intent::GetDesignationList
        | Intent::CalculateAnalytics
        | Intent::UndoLastUpdate
        | Intent::BatchUpdate => {
            Ok(json!({}))
        }

        Intent::Unknown(_) => Ok(json!({})),
    }
}

// =============================================================================
// Pretty-print tool results to the terminal
// Supports selective field projection when the user requests specific fields
// =============================================================================
#[allow(dead_code)]
fn display_result(result: &ToolExecutionResult, requested_fields: &[String]) {
    println!();
    if result.success {
        println!(
            "  ✓ Success{}",
            result
                .message
                .as_ref()
                .map(|m| format!(": {}", m))
                .unwrap_or_default()
        );
    } else {
        println!(
            "  ✗ Error{}",
            result
                .message
                .as_ref()
                .map(|m| format!(": {}", m))
                .unwrap_or_default()
        );
    }

    if let Some(ref data) = result.data {
        println!();
        match data {
            Value::Array(rows) => {
                println!(
                    "  Results ({} row{}):",
                    rows.len(),
                    if rows.len() == 1 { "" } else { "s" }
                );
                for (i, row) in rows.iter().enumerate() {
                    println!("  [{:>2}] {}", i + 1, format_row(row, requested_fields));
                }
            }
            Value::Object(_) => {
                println!("  {}", format_row(data, requested_fields));
            }
            other => {
                println!("  {}", other);
            }
        }
    }
}

fn format_row(val: &Value, requested_fields: &[String]) -> String {
    if let Some(obj) = val.as_object() {
        if requested_fields.is_empty() {
            // Display all fields
            obj.iter()
                .map(|(k, v)| format!("{}: {}", k, format_field_value(v)))
                .collect::<Vec<_>>()
                .join("  |  ")
        } else {
            // Display employeeId first for context, then each requested field in order
            let mut parts = Vec::new();
            if let Some(id_val) = obj.get("employeeId") {
                parts.push(format!("employeeId: {}", format_field_value(id_val)));
            }
            for rf in requested_fields {
                if rf.eq_ignore_ascii_case("employeeId") {
                    continue;
                }
                if let Some((k, v)) = obj.iter().find(|(k, _)| k.eq_ignore_ascii_case(rf)) {
                    let field_entry = format!("{}: {}", k, format_field_value(v));
                    if !parts.contains(&field_entry) {
                        parts.push(field_entry);
                    }
                }
            }
            if parts.is_empty() {
                // Fallback to all fields if no requested fields matched object properties
                obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, format_field_value(v)))
                    .collect::<Vec<_>>()
                    .join("  |  ")
            } else {
                parts.join("  |  ")
            }
        }
    } else {
        val.to_string()
    }
}

fn format_field_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "—".to_string(),
        other => other.to_string(),
    }
}

fn print_help() {
    println!("  ─────────────────────────────────────────────────────");
    println!("  SUPPORTED COMMANDS (Regex-matched, no AI):");
    println!("  ─────────────────────────────────────────────────────");
    println!("  Show all employees");
    println!("  List all employees");
    println!("  Find employee 1001");
    println!("  Show employee ID 1005");
    println!("  Search employees in IT");
    println!("  Search employees in Engineering department");
    println!("  Find employees named John");
    println!("  Show active employees");
    println!("  Show all departments");
    println!("  How many employees are there?");
    println!("  Count employees in HR");
    println!("  Add employee John Smith email j.smith@ex.com");
    println!("  Update employee 1001 status to Inactive");
    println!("  Update employee 1001 department to Engineering");
    println!("  Update employee 1001 salary to 95000");
    println!("  help  |  exit");
    println!("  ─────────────────────────────────────────────────────");
}
