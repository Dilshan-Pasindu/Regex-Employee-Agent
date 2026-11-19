// =============================================================================
// api/client.ts — Axios-free fetch wrapper for the FastAPI backend (port 5000)
// =============================================================================

import type { ChatResponse, McpTool, ConversationSummary, PersistedMessage } from '../types';

const BASE_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:5000';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const resp = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`${resp.status} ${resp.statusText}: ${text}`);
  }
  return resp.json() as Promise<T>;
}

// POST /api/chat
export async function sendChat(
  prompt: string,
  confirmed = false,
  sessionId?: string,
): Promise<ChatResponse> {
  return request<ChatResponse>('/api/chat', {
    method: 'POST',
    body: JSON.stringify({ prompt, confirmed, session_id: sessionId }),
  });
}

// GET /api/tools
export async function fetchTools(): Promise<{ tools: McpTool[]; count: number }> {
  return request<{ tools: McpTool[]; count: number }>('/api/tools');
}

// GET /api/health
export async function fetchHealth(): Promise<Record<string, unknown>> {
  return request<Record<string, unknown>>('/api/health');
}

// ---------------------------------------------------------------------------
// Phase 1 — Conversation management API
// ---------------------------------------------------------------------------

export async function createConversation(title?: string): Promise<ConversationSummary> {
  return request<ConversationSummary>('/api/conversations', {
    method: 'POST',
    body: JSON.stringify({ title: title ?? null }),
  });
}

export async function listConversations(
  page = 1,
  pageSize = 30,
): Promise<{ conversations: ConversationSummary[]; page: number; page_size: number }> {
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  return request<{ conversations: ConversationSummary[]; page: number; page_size: number }>(
    `/api/conversations?${params}`,
  );
}

export async function searchConversations(q: string): Promise<{ conversations: ConversationSummary[] }> {
  return request<{ conversations: ConversationSummary[] }>(
    `/api/conversations/search?q=${encodeURIComponent(q)}`,
  );
}

export async function renameConversation(id: string, title: string): Promise<ConversationSummary> {
  return request<ConversationSummary>(`/api/conversations/${id}`, {
    method: 'PATCH',
    body: JSON.stringify({ title }),
  });
}

export async function pinConversation(id: string, pinned: boolean): Promise<ConversationSummary> {
  return request<ConversationSummary>(`/api/conversations/${id}/pin`, {
    method: 'PATCH',
    body: JSON.stringify({ pinned }),
  });
}

export async function deleteConversation(id: string): Promise<void> {
  await request<unknown>(`/api/conversations/${id}`, { method: 'DELETE' });
}

export async function getConversationContext(
  id: string,
): Promise<{ conversation_id: string; version: number; schema_version: number; context: Record<string, unknown> }> {
  return request<{ conversation_id: string; version: number; schema_version: number; context: Record<string, unknown> }>(
    `/api/conversations/${id}/context`,
  );
}

export async function getConversationMessages(
  id: string,
  page = 1,
  pageSize = 50,
): Promise<{ conversation_id: string; messages: PersistedMessage[]; page: number; page_size: number }> {
  const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) });
  return request<{ conversation_id: string; messages: PersistedMessage[]; page: number; page_size: number }>(
    `/api/conversations/${id}/messages?${params}`,
  );
}
