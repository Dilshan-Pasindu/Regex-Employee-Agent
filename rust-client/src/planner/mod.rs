// =============================================================================
// planner/mod.rs — Multi-step task planning module
// =============================================================================

pub mod execution_engine;
pub mod execution_plan;
pub mod plan_validator;
pub mod task_planner;

pub use execution_engine::{ExecutionOutcome, execute_plan, project_fields};
pub use execution_plan::{ComparisonOperator, ExecutionPlan, ExecutionStep, ExtractedConditions};
pub use plan_validator::{ValidationResult, validate_plan};
pub use task_planner::{create_execution_plan, extract_salary_comparison};
