// =============================================================================
// types/index.ts — Shared TypeScript types for the agentic chat UI
// =============================================================================

export interface StepTrace {
  step: number;
  label: string;
  value: unknown;
}

export interface ChatResponse {
  session_id: string;
  prompt: string;
  intent: string;
  tool?: string;
  params: Record<string, unknown>;
  steps: StepTrace[];
  result?: unknown;
  success: boolean;
  requires_confirmation?: boolean;
  message?: string;
  suggestions?: string[];
  request_id?: string;
  status?: string;
  error_code?: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'agent';
  content: string;
  timestamp: Date;
  response?: ChatResponse;        // agent messages carry the full response
  pending?: boolean;              // streaming/loading state
}

export interface McpTool {
  name: string;
  description?: string;
  inputSchema?: {
    type?: string;
    properties?: Record<string, { type?: string; description?: string }>;
    required?: string[];
  };
}

export interface ConfirmationState {
  prompt: string;
  intent: string;
  params: Record<string, unknown>;
  sessionId: string;
}

// ---------------------------------------------------------------------------
// Phase 1 — Persistent conversation types
// ---------------------------------------------------------------------------

export interface ConversationSummary {
  conversation_id: string;
  title: string;
  created_at: string;
  updated_at: string;
  last_message_at: string | null;
  is_pinned: boolean;
  pinned_at: string | null;
}

export interface PersistedMessage {
  message_id: string;
  sequence_number: number;
  role: string;        // "user" | "assistant"
  content: string;
  intent: string | null;
  success: boolean | null;
  metadata_json?: string | null;
  created_at: string;
  status?: string;
  request_id?: string | null;
  error_code?: string | null;
  processing_started_at?: string | null;
  processing_completed_at?: string | null;
}
