// =============================================================================
// suggestions/mod.rs — Prompt suggestion and recovery module
// =============================================================================

pub mod context_analyzer;
pub mod error_classifier;
pub mod suggestion_engine;
pub mod suggestion_templates;

pub use error_classifier::{ClassificationResult, PromptErrorType, classify_prompt_error};
pub use suggestion_engine::{SuggestionOutput, generate_suggestions};
