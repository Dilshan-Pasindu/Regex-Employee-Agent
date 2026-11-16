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
