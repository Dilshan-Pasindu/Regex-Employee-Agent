// =============================================================================
// api/client.ts — Axios-free fetch wrapper for the FastAPI backend (port 5000)
// =============================================================================

import type { ChatResponse, McpTool } from '../types';

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
