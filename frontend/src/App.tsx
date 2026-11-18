// =============================================================================
// App.tsx — Root application shell (Phase 1: Persistent Chat History)
// New: ConversationSidebar + MySQL-backed conversation switching
// All existing agentic logic (voice, confirmation, tools) preserved.
// =============================================================================

import { useState, useRef, useCallback, useEffect } from 'react';
import { ChatPanel } from './components/ChatPanel';
import { ConfirmationModal } from './components/ConfirmationModal';
import { ToolCatalogDrawer } from './components/ToolCatalogDrawer';
import { ConversationSidebar } from './components/ConversationSidebar';
import {
  sendChat,
  createConversation,
  getConversationMessages,
} from './api/client';
import type { ChatMessage, ChatResponse, ConfirmationState, PersistedMessage } from './types';
import botAvatar from './assets/bot-avatar.jpg';
import micIcon from './assets/blue-mic.png';
import './index.css';

function nanoid() {
  return Math.random().toString(36).slice(2, 11);
}

function persistedToChat(m: PersistedMessage): ChatMessage {
  let response: ChatResponse | undefined = undefined;
  let content = m.content;

  if (m.role === 'assistant') {
    if (m.metadata_json) {
      try {
        const meta = JSON.parse(m.metadata_json);
        response = {
          session_id: meta.session_id || '',
          prompt: meta.prompt || '',
          intent: m.intent || meta.intent || 'unknown',
          tool: meta.tool,
          params: meta.params || {},
          steps: meta.steps || [],
          result: meta.result,
          success: m.success ?? meta.success ?? true,
          requires_confirmation: meta.requires_confirmation,
          message: content,
          suggestions: meta.suggestions || [],
        };
      } catch (err) {
        console.warn('Failed to parse metadata_json', err);
      }
    } else {
      // Graceful fallback for messages saved before metadata_json column:
      // If content was a stringified Python dict, convert to JSON so EmployeeTable renders it
      const trimmed = content.trim();
      if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
        try {
          const normalizedJson = trimmed.replace(/'/g, '"');
          const parsed = JSON.parse(normalizedJson);
          response = {
            session_id: '',
            prompt: '',
            intent: m.intent || 'unknown',
            params: {},
            steps: [],
            result: parsed,
            success: m.success ?? true,
          };
          content = 'Done.';
        } catch {
          // keep as raw markdown
        }
      }
    }
  }

  return {
    id: m.message_id,
    role: m.role === 'user' ? 'user' : 'agent',
    content,
    timestamp: new Date(m.created_at),
    response,
  };
}

export default function App() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<ConfirmationState | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [isDark, setIsDark] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [sidebarRefresh, setSidebarRefresh] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const toggleVoiceInput = () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const win = window as any;
    const SpeechRecognition = win.SpeechRecognition || win.webkitSpeechRecognition;
    if (!SpeechRecognition) {
      alert('Speech recognition is not supported in this browser. Please use Chrome or Edge.');
      return;
    }
    if (isListening) { setIsListening(false); return; }
    try {
      const recognition = new SpeechRecognition();
      recognition.lang = 'en-US';
      recognition.interimResults = false;
      recognition.maxAlternatives = 1;
      recognition.onstart = () => setIsListening(true);
      recognition.onend = () => setIsListening(false);
      recognition.onerror = () => setIsListening(false);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      recognition.onresult = (event: any) => {
        const transcript = event.results[0][0].transcript;
        setInput(prev => (prev ? `${prev} ${transcript}` : transcript));
      };
      recognition.start();
    } catch (err) {
      console.error('Speech recognition error:', err);
      setIsListening(false);
    }
  };

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  }, [isDark]);

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

  // Load an existing conversation's messages from MySQL
  async function loadConversation(conversationId: string) {
    setLoading(true);
    try {
      const res = await getConversationMessages(conversationId);
      const chatMsgs = res.messages.map(persistedToChat);
      setMessages(chatMsgs);
      setSessionId(conversationId);
    } catch (e) {
      console.error('Failed to load conversation:', e);
    } finally {
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }

  // Start a new blank conversation
  function handleNewConversation() {
    setMessages([]);
    setSessionId(null);
    setConfirmation(null);
    setTimeout(() => inputRef.current?.focus(), 50);
  }

  async function submit(prompt: string, confirmed = false) {
    if (!prompt.trim() || loading) return;

    addMessage({
      id: nanoid(),
      role: 'user',
      content: prompt.trim(),
      timestamp: new Date(),
    });

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
      const resp = await sendChat(prompt.trim(), confirmed, sessionId ?? undefined);

      if (!sessionId) {
        setSessionId(resp.session_id);
        // Trigger sidebar to refresh so the new conversation appears
        setSidebarRefresh(r => r + 1);
      }

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

      const content = resp.message ?? (resp.success ? 'Done.' : 'Something went wrong.');
      updateLastAgentMessage({ id: agentId, content, pending: false, response: resp });

      // Refresh sidebar to show updated last_message_at
      setSidebarRefresh(r => r + 1);
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

  function handleSend() { submit(input); setInput(''); }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend(); }
  }

  function handleConfirm() {
    if (!confirmation) return;
    const { prompt } = confirmation;
    setConfirmation(null);
    submit(prompt, true);
  }

  function handleCancel() {
    setConfirmation(null);
    addMessage({ id: nanoid(), role: 'agent', content: 'Operation cancelled.', timestamp: new Date() });
  }

  const headerGrad = 'linear-gradient(135deg, #7c3aed 0%, #db2777 55%, #f97316 100%)';

  // suppress unused import warning — createConversation is available for future use
  void createConversation;

  return (
    <div
      style={{ background: 'var(--bg)', color: 'var(--text)', display: 'flex', flexDirection: 'column', height: '100vh' }}
    >
      {/* ---- Header --------------------------------------------------- */}
      <header
        style={{ background: headerGrad, flexShrink: 0 }}
        className="flex items-center justify-between px-4 py-3 shadow-lg"
        role="banner"
      >
        {/* Left: sidebar toggle + avatar + title */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => setSidebarOpen(o => !o)}
            aria-label={sidebarOpen ? 'Close sidebar' : 'Open sidebar'}
            style={{
              background: 'rgba(255,255,255,0.18)', border: 'none', color: '#fff',
              borderRadius: 8, padding: '5px 8px', cursor: 'pointer', fontSize: '1rem',
            }}
          >
            {sidebarOpen ? '◀' : '▶'}
          </button>
          <img
            src={botAvatar}
            alt="Employee Agent Logo"
            style={{ width: 42, height: 42, borderRadius: 14, objectFit: 'cover', boxShadow: '0 2px 10px rgba(0,0,0,0.25)', border: '2px solid rgba(255,255,255,0.4)' }}
          />
          <div>
            <h1 style={{ fontSize: '0.95rem', fontWeight: 800, color: '#fff', lineHeight: 1.2, margin: 0 }}>
              Employee Agent
            </h1>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginTop: 2 }}>
              <span style={{ width: 7, height: 7, borderRadius: '50%', background: '#4ade80', boxShadow: '0 0 0 2px rgba(74,222,128,0.4)', animation: 'pulseGlow 2s infinite', display: 'inline-block' }} />
              <span style={{ fontSize: '0.68rem', color: 'rgba(255,255,255,0.9)', fontWeight: 600 }}>Online</span>
            </div>
          </div>
        </div>

        {/* Right: session id + dark toggle + tools */}
        <div className="flex gap-2 items-center">
          {sessionId && (
            <span style={{ fontSize: '0.65rem', color: 'rgba(255,255,255,0.6)', fontFamily: 'monospace', maxWidth: 100, overflow: 'hidden', whiteSpace: 'nowrap', textOverflow: 'ellipsis' }} className="hidden sm:block">
              {sessionId.slice(0, 8)}…
            </span>
          )}
          <button onClick={() => setIsDark(d => !d)} aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
            style={{ background: 'rgba(255,255,255,0.18)', border: 'none', color: '#fff', borderRadius: 10, padding: '5px 10px', cursor: 'pointer', fontSize: '0.8rem', fontWeight: 700 }}>
            {isDark ? '☀️' : '🌙'}
          </button>
          <button onClick={() => setDrawerOpen(o => !o)} aria-label="Open MCP tool catalogue"
            style={{ background: 'rgba(255,255,255,0.18)', border: 'none', color: '#fff', borderRadius: 10, padding: '5px 10px', cursor: 'pointer', fontSize: '0.8rem', fontWeight: 700 }}>
            🔧 Tools
          </button>
          <button onClick={handleNewConversation} aria-label="New chat"
            style={{ background: 'rgba(255,255,255,0.18)', border: 'none', color: '#fff', borderRadius: 10, padding: '5px 10px', cursor: 'pointer', fontSize: '0.8rem', fontWeight: 700 }}>
            ✏️ New
          </button>
        </div>
      </header>

      {/* ---- Main area: sidebar + chat ---------------------------------- */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Sidebar */}
        {sidebarOpen && (
          <ConversationSidebar
            activeConversationId={sessionId}
            onSelectConversation={loadConversation}
            onNewConversation={handleNewConversation}
            refreshTrigger={sidebarRefresh}
            isDark={isDark}
          />
        )}

        {/* Chat area */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <ChatPanel messages={messages} onChipClick={(text) => { submit(text); }} />

          {/* ---- Input bar ----------------------------------------- */}
          <div style={{ background: 'var(--surface)', borderTop: '1px solid var(--border)', padding: '12px 16px 14px', flexShrink: 0 }}>
            <div style={{ display: 'flex', gap: 10, maxWidth: 860, margin: '0 auto', background: 'var(--surface-2)', borderRadius: 20, padding: '6px 6px 6px 16px', border: '1.5px solid var(--border)', boxShadow: '0 2px 20px rgba(124,58,237,0.08)', transition: 'box-shadow 0.2s' }}
              onFocusCapture={e => (e.currentTarget.style.boxShadow = '0 0 0 3px rgba(124,58,237,0.25)')}
              onBlurCapture={e => (e.currentTarget.style.boxShadow = '0 2px 20px rgba(124,58,237,0.08)')}>
              <input
                ref={inputRef}
                type="text"
                value={input}
                onChange={e => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                disabled={loading}
                placeholder="e.g. Show all employees · Find employee 1001 · Add employee…"
                aria-label="Chat input"
                style={{ flex: 1, background: 'transparent', border: 'none', outline: 'none', color: 'var(--text)', fontSize: '0.9rem', fontFamily: 'var(--font-body)', opacity: loading ? 0.5 : 1 }}
              />
              <button type="button" onClick={toggleVoiceInput} aria-label="Voice input"
                title={isListening ? 'Listening... click to stop' : 'Voice input (dictate prompt)'}
                style={{ background: isListening ? 'rgba(239,68,68,0.15)' : 'rgba(9,106,217,0.08)', border: isListening ? '1.5px solid #ef4444' : '1px solid rgba(9,106,217,0.3)', borderRadius: 12, padding: isListening ? '6px 12px' : '6px 10px', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6, transition: 'all 0.2s cubic-bezier(0.4,0,0.2,1)', animation: isListening ? 'micPulse 1.5s infinite ease-in-out' : 'none' }}>
                <img src={micIcon} alt="Microphone" style={{ width: 20, height: 20, objectFit: 'contain', filter: isListening ? 'drop-shadow(0 0 4px #ef4444)' : 'none' }} />
                {isListening && <span style={{ fontSize: '0.78rem', fontWeight: 600, color: '#ef4444' }}>Listening...</span>}
              </button>
              <button onClick={handleSend} disabled={loading || !input.trim()} aria-label="Send message"
                style={{ background: loading || !input.trim() ? 'rgba(124,58,237,0.3)' : 'linear-gradient(135deg, #7c3aed, #db2777)', border: 'none', color: '#fff', borderRadius: 14, padding: '8px 20px', fontWeight: 700, fontSize: '0.85rem', fontFamily: 'var(--font-body)', cursor: loading || !input.trim() ? 'not-allowed' : 'pointer', transition: 'all 0.2s', whiteSpace: 'nowrap' }}>
                {loading ? '…' : 'Send ↑'}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* ---- Overlays ------------------------------------------------- */}
      {confirmation && (
        <ConfirmationModal intent={confirmation.intent} params={confirmation.params} onConfirm={handleConfirm} onCancel={handleCancel} />
      )}
      <ToolCatalogDrawer open={drawerOpen} onClose={() => setDrawerOpen(false)} />
    </div>
  );
}
