// =============================================================================
// main.rs — Employee MCP Client: Interactive Terminal CLI (Phase 2)
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
    intent::detector::detect_intent,
    intent::patterns::{RE_EXIT, RE_HELP},
    mcp::client::McpClient,
    models::{Intent, IntentResult, ToolExecutionResult},
};

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All tracing goes to STDERR — STDOUT is for user-facing output only
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("employee_mcp_client=info".parse()?)
                .add_directive("warn".parse()?)
        )
        .init();

    let config = Config::from_env();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Employee Management Agent — MCP Client v1.0.0     ║");
    println!("║   Phase 2: Deterministic Regex Intent Engine         ║");
    println!("║   No LLM. No AI. Pure Regex routing via MCP/STDIO.  ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    println!("  Starting .NET MCP Server...");

    let mut client = match McpClient::connect(&config).await {
        Ok(c) => {
            println!("  ✓ MCP Server connected. {} tools discovered.", c.discovered_tools.len());
            println!("  ✓ Tools: {}", c.tool_names().join(", "));
            println!();
            c
        }
        Err(e) => {
            eprintln!("  ✗ Failed to connect to MCP Server: {}", e);
            eprintln!("  Make sure you have run 'dotnet build' in dotnet-mcp-server/EmployeeMcpServer");
            eprintln!("  and that your MySQL connection string in appsettings.json is correct.");
            return Err(e.into());
        }
    };

    print_help();

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

        let prompt = line.trim().to_string();
        if prompt.is_empty() {
            continue;
        }

        // Exit commands
        if RE_EXIT.is_match(&prompt) {
            println!("Goodbye!");
            break;
        }

        // Help command
        if RE_HELP.is_match(&prompt) {
            print_help();
            continue;
        }

        // Detect intent deterministically via Regex
        let intent_result = detect_intent(&prompt);

        match &intent_result.intent {
            Intent::Unknown(text) => {
                if text == "exit" { break; }
                println!("  ✗ I didn't understand that prompt.");
                println!("    Supported commands: show/list/find/add/update employees, count, departments.");
                println!("    Type 'help' for examples.");
                continue;
            }
            _ => {}
        }

        println!("  → Intent : {:?}", intent_result.intent);

        let tool_name = match intent_result.intent.tool_name() {
            Some(t) => t.to_string(),
            None => {
                println!("  ✗ No tool mapped for this intent.");
                continue;
            }
        };

        // Build tool arguments from extracted parameters
        let args = match build_tool_args(&intent_result) {
            Ok(a) => a,
            Err(msg) => {
                println!("  ✗ Missing information: {}", msg);
                println!("    Please provide more details and try again.");
                continue;
            }
        };

        // Confirmation gate for write operations
        if intent_result.intent.requires_confirmation() {
            let confirmed = request_confirmation(&intent_result);
            if !confirmed {
                println!("  ✗ Operation cancelled.");
                continue;
            }
        }

        // Execute the MCP tool
        println!("  → Calling tool: {} ...", tool_name);
        match client.call_tool(&tool_name, args).await {
            Ok(result) => {
                display_result(&result, &intent_result.params.requested_fields);
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
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
fn build_tool_args(intent: &IntentResult) -> Result<Value, String> {
    let p = &intent.params;
    match &intent.intent {

        Intent::GetEmployeeById => {
            let id = p.employee_id.ok_or("Employee ID is required. Example: 'Find employee 1001'")?;
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
                return Err("Please specify a filter. Example: 'Search employees in IT' or 'Find John'".to_string());
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
            let first = p.first_name.as_deref().ok_or("First name is required for adding an employee.")?;
            let last  = p.last_name.as_deref().ok_or("Last name is required for adding an employee.")?;
            let email = p.email.as_deref().ok_or("Email is required. Example: john.smith@example.com")?;

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
                println!("     The .NET server requires IDs. Use 'show departments' and 'show designations' first.");
                println!("     Then re-run: add employee ... departmentId 1 designationId 1");
            }

            Ok(args)
        }

        Intent::UpdateEmployee => {
            let id = p.employee_id.ok_or("Employee ID is required. Example: 'Update employee 1001 status to Inactive'")?;

            let mut args = serde_json::Map::new();
            args.insert("employeeId".to_string(), json!(id));

            if let Some(ref f) = p.first_name       { args.insert("firstName".to_string(), json!(f)); }
            if let Some(ref l) = p.last_name        { args.insert("lastName".to_string(), json!(l)); }
            if let Some(ref e) = p.email            { args.insert("email".to_string(), json!(e)); }
            if let Some(ref ph) = p.phone           { args.insert("phone".to_string(), json!(ph)); }
            if let Some(ref st) = p.status          { args.insert("status".to_string(), json!(st)); }
            if let Some(salary) = p.salary          { args.insert("salary".to_string(), json!(salary)); }
            if let Some(ref date) = p.hire_date     { args.insert("hireDate".to_string(), json!(date)); }

            if args.len() == 1 {
                return Err(
                    "Please specify what to update. Example: 'Update employee 1001 status to Inactive'".to_string()
                );
            }

            Ok(Value::Object(args))
        }

        // Read-only tools with no required args
        Intent::GetAllEmployees | Intent::GetDepartmentList | Intent::GetDesignationList => {
            Ok(json!({}))
        }

        Intent::Unknown(_) => Ok(json!({})),
    }
}

// =============================================================================
// Pretty-print tool results to the terminal
// Supports selective field projection when the user requests specific fields
// =============================================================================
fn display_result(result: &ToolExecutionResult, requested_fields: &[String]) {
    println!();
    if result.success {
        println!("  ✓ Success{}", result.message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default());
    } else {
        println!("  ✗ Error{}", result.message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default());
    }

    if let Some(ref data) = result.data {
        println!();
        match data {
            Value::Array(rows) => {
                println!("  Results ({} row{}):", rows.len(), if rows.len() == 1 { "" } else { "s" });
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
        Value::Bool(b)   => b.to_string(),
        Value::Null      => "—".to_string(),
        other            => other.to_string(),
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
