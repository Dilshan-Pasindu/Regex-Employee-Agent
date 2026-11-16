// =============================================================================
// App.tsx — Root application shell
//
// Layout:
//   ┌──────────────────────────────────┐
//   │  Header (title, health, tools)   │
//   ├──────────────────────────────────┤
//   │                                  │
//   │       ChatPanel (messages)       │
//   │                                  │
//   ├──────────────────────────────────┤
//   │  Input bar + send button         │
//   └──────────────────────────────────┘
//   [ConfirmationModal overlay]
//   [ToolCatalogDrawer sidebar]
// =============================================================================

import { useState, useRef, useCallback } from 'react';
import { ChatPanel } from './components/ChatPanel';
import { ConfirmationModal } from './components/ConfirmationModal';
import { ToolCatalogDrawer } from './components/ToolCatalogDrawer';
import { sendChat } from './api/client';
import type { ChatMessage, ConfirmationState } from './types';
import './index.css';

function nanoid() {
  return Math.random().toString(36).slice(2, 11);
}

export default function App() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [sessionId, setSessionId] = useState<string | undefined>(undefined);
  const [confirmation, setConfirmation] = useState<ConfirmationState | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const addMessage = useCallback((msg: ChatMessage) => {
    setMessages(prev => [...prev, msg]);
  }, []);

  const updateLastAgentMessage = useCallback((update: Partial<ChatMessage>) => {
    setMessages(prev => {
      const copy = [...prev];
      for (let i = copy.length - 1; i >= 0; i--) {
        if (copy[i].role === 'agent') {
          copy[i] = { ...copy[i], ...update };
          break;
        }
      }
      return copy;
    });
  }, []);

  async function submit(prompt: string, confirmed = false) {
    if (!prompt.trim() || loading) return;

    // Add user bubble
    addMessage({
      id: nanoid(),
      role: 'user',
      content: prompt.trim(),
      timestamp: new Date(),
    });

    // Add pending agent bubble
    const agentId = nanoid();
    addMessage({
      id: agentId,
      role: 'agent',
      content: '',
      timestamp: new Date(),
      pending: true,
    });

    setLoading(true);
    try {
      const resp = await sendChat(prompt.trim(), confirmed, sessionId);

      // Persist session
      if (!sessionId) setSessionId(resp.session_id);

      // Write ops needing confirmation
      if (resp.requires_confirmation) {
        setConfirmation({
          prompt: prompt.trim(),
          intent: resp.intent,
          params: resp.params,
          sessionId: resp.session_id,
        });
        updateLastAgentMessage({
          id: agentId,
          content: 'Confirmation required — please review the extracted parameters.',
          pending: false,
          response: resp,
        });
        return;
      }

      const content = resp.message
        ?? (resp.success ? 'Done.' : 'Something went wrong.');

      updateLastAgentMessage({
        id: agentId,
        content,
        pending: false,
        response: resp,
      });
    } catch (err) {
      updateLastAgentMessage({
        id: agentId,
        content: `Error: ${err instanceof Error ? err.message : String(err)}`,
        pending: false,
        response: undefined,
      });
    } finally {
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }

  function handleSend() {
    submit(input);
    setInput('');
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  function handleConfirm() {
    if (!confirmation) return;
    const { prompt } = confirmation;
    setConfirmation(null);
    submit(prompt, true);
  }

  function handleCancel() {
    setConfirmation(null);
    addMessage({
      id: nanoid(),
      role: 'agent',
      content: 'Operation cancelled.',
      timestamp: new Date(),
    });
  }

  return (
    <div className="flex flex-col h-screen bg-slate-900 text-slate-200">
      {/* ------------------------------------------------------------------ */}
      {/* Header */}
      {/* ------------------------------------------------------------------ */}
      <header className="shrink-0 flex items-center justify-between px-4 py-3
                         border-b border-slate-700 bg-slate-900/90 backdrop-blur">
        <div className="flex items-center gap-2">
          <span className="text-2xl">🤖</span>
          <div>
            <h1 className="text-sm font-semibold text-slate-100 leading-tight">
              Employee Management Agent
            </h1>
            <p className="text-xs text-slate-500">
              MCP · .NET · Rust · FastAPI · MySQL · <span className="text-emerald-400">No LLM</span>
            </p>
          </div>
        </div>

        <div className="flex gap-2 items-center">
          {sessionId && (
            <span className="hidden sm:block text-xs text-slate-600 font-mono truncate max-w-[140px]">
              {sessionId.slice(0, 8)}…
            </span>
          )}
          <button
            onClick={() => setDrawerOpen(o => !o)}
            className="text-xs bg-slate-800 hover:bg-slate-700 border border-slate-600
                       text-slate-300 px-3 py-1.5 rounded-lg transition-colors"
          >
            🔧 Tools
          </button>
          <button
            onClick={() => { setMessages([]); setSessionId(undefined); }}
            className="text-xs bg-slate-800 hover:bg-slate-700 border border-slate-600
                       text-slate-300 px-3 py-1.5 rounded-lg transition-colors"
          >
            🗑 Clear
          </button>
        </div>
      </header>

      {/* ------------------------------------------------------------------ */}
      {/* Chat panel */}
      {/* ------------------------------------------------------------------ */}
      <ChatPanel messages={messages} />

      {/* ------------------------------------------------------------------ */}
      {/* Input bar */}
      {/* ------------------------------------------------------------------ */}
      <div className="shrink-0 border-t border-slate-700 bg-slate-900/90 backdrop-blur px-4 py-3">
        <div className="flex gap-2 max-w-4xl mx-auto">
          <input
            ref={inputRef}
            type="text"
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={loading}
            placeholder="e.g. Show all employees · Find employee 1001 · Add employee…"
            className="flex-1 bg-slate-800 border border-slate-600 rounded-xl px-4 py-2.5
                       text-sm text-slate-100 placeholder-slate-500 outline-none
                       focus:border-sky-500 transition-colors disabled:opacity-50"
          />
          <button
            onClick={handleSend}
            disabled={loading || !input.trim()}
            className="bg-sky-600 hover:bg-sky-500 disabled:opacity-40 disabled:cursor-not-allowed
                       text-white font-semibold px-4 py-2.5 rounded-xl transition-colors text-sm
                       shrink-0"
          >
            {loading ? '…' : 'Send'}
          </button>
        </div>
        <p className="text-center text-xs text-slate-700 mt-1.5">
          Deterministic Rust Regex intent detection · Zero AI/LLM
        </p>
      </div>

      {/* ------------------------------------------------------------------ */}
      {/* Overlays */}
      {/* ------------------------------------------------------------------ */}
      {confirmation && (
        <ConfirmationModal
          intent={confirmation.intent}
          params={confirmation.params}
          onConfirm={handleConfirm}
          onCancel={handleCancel}
        />
      )}

      <ToolCatalogDrawer open={drawerOpen} onClose={() => setDrawerOpen(false)} />
    </div>
  );
}
