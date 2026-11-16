// =============================================================================
// planner/execution_plan.rs — Structured Execution Plan & Step Models
// =============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl ComparisonOperator {
    pub fn matches(&self, val: f64, target: f64) -> bool {
        match self {
            ComparisonOperator::Equal => (val - target).abs() < f64::EPSILON,
            ComparisonOperator::GreaterThan => val > target,
            ComparisonOperator::GreaterThanOrEqual => val >= target,
            ComparisonOperator::LessThan => val < target,
            ComparisonOperator::LessThanOrEqual => val <= target,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: usize,
    pub description: String,
    pub tool_name: Option<String>,
    pub arguments: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedConditions {
    pub employee_id: Option<i64>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub salary: Option<f64>,
    pub salary_operator: Option<ComparisonOperator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub intent_label: String,
    pub steps: Vec<ExecutionStep>,
    pub extracted_conditions: ExtractedConditions,
    pub requested_fields: Vec<String>,
    pub requires_confirmation: bool,
}
