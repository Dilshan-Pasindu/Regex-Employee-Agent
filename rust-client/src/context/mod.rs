// =============================================================================
// context/mod.rs — Conversational context and memory module
// =============================================================================

pub mod context_resolver;
pub mod conversation_context;

pub use context_resolver::{ResolvedContextAction, enrich_params_with_context, resolve_context};
pub use conversation_context::{
    ConversationContext, SESSION_STORE, clear_session_context, get_or_create_context,
    update_context,
};
