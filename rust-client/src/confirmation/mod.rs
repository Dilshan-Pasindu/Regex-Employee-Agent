// =============================================================================
// confirmation/mod.rs — Write-operation confirmation handler
// add_employee and update_employee MUST be confirmed before execution.
// =============================================================================

use crate::models::{Intent, IntentResult};
use std::io::{self, Write};

/// Display a confirmation prompt to the user for write operations.
/// Returns true only if the user explicitly types "yes" or "y".
pub fn request_confirmation(intent: &IntentResult) -> bool {
    println!();
    println!("╔══════════════════════════════════════════════╗");

    match &intent.intent {
        Intent::AddEmployee => {
            println!("║          CONFIRM: ADD NEW EMPLOYEE           ║");
            println!("╚══════════════════════════════════════════════╝");

            let p = &intent.params;
            if let Some(ref first) = p.first_name {
                println!("  First Name   : {}", first);
            }
            if let Some(ref last) = p.last_name {
                println!("  Last Name    : {}", last);
            }
            if let Some(ref email) = p.email {
                println!("  Email        : {}", email);
            }
            if let Some(ref dept) = p.department_name {
                println!("  Department   : {}", dept);
            }
            if let Some(ref desig) = p.designation_name {
                println!("  Designation  : {}", desig);
            }
            if let Some(salary) = p.salary {
                println!("  Salary       : {}", salary);
            }
            if let Some(ref date) = p.hire_date {
                println!("  Hire Date    : {}", date);
            }
            if let Some(ref status) = p.status {
                println!("  Status       : {}", status);
            }
        }

        Intent::UpdateEmployee => {
            println!("║          CONFIRM: UPDATE EMPLOYEE            ║");
            println!("╚══════════════════════════════════════════════╝");

            let p = &intent.params;
            if let Some(id) = p.employee_id {
                println!("  Employee ID  : {}", id);
            }
            if let Some(ref field) = p.field_to_update {
                println!("  Field        : {}", field);
                if let Some(ref val) = p.new_value {
                    println!("  New Value    : {}", val);
                }
            } else {
                // Show all updated fields
                if let Some(ref f) = p.first_name {
                    println!("  First Name   : {}", f);
                }
                if let Some(ref l) = p.last_name {
                    println!("  Last Name    : {}", l);
                }
                if let Some(ref e) = p.email {
                    println!("  Email        : {}", e);
                }
                if let Some(ref d) = p.department_name {
                    println!("  Department   : {}", d);
                }
                if let Some(ref g) = p.designation_name {
                    println!("  Designation  : {}", g);
                }
                if let Some(s) = p.salary {
                    println!("  Salary       : {}", s);
                }
                if let Some(ref st) = p.status {
                    println!("  Status       : {}", st);
                }
            }
        }

        _ => {
            println!("║          CONFIRM THIS OPERATION?             ║");
            println!("╚══════════════════════════════════════════════╝");
        }
    }

    println!();
    print!("  Do you want to proceed? (yes / no) > ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let answer = input.trim().to_lowercase();
        matches!(answer.as_str(), "yes" | "y")
    } else {
        false
    }
}
