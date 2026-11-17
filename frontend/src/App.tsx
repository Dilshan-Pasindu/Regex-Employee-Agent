// =============================================================================
// App.tsx — Root application shell (vibrant redesign)
// All existing logic preserved; only UI layer changed.
// =============================================================================

import { useState, useRef, useCallback, useEffect } from 'react';
import { ChatPanel } from './components/ChatPanel';
import { ConfirmationModal } from './components/ConfirmationModal';
import { ToolCatalogDrawer } from './components/ToolCatalogDrawer';
import { sendChat } from './api/client';
import type { ChatMessage, ConfirmationState } from './types';
import botAvatar from './assets/bot-avatar.jpg';
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
  const [isDark, setIsDark] = useState(false);
  const [isListening, setIsListening] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const toggleVoiceInput = () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const win = window as any;
    const SpeechRecognition = win.SpeechRecognition || win.webkitSpeechRecognition;

    if (!SpeechRecognition) {
      alert("Speech recognition is not supported in this browser. Please use Chrome or Edge.");
      return;
    }

    if (isListening) {
      setIsListening(false);
      return;
    }

    try {
      const recognition = new SpeechRecognition();
      recognition.lang = "en-US";
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
      console.error("Speech recognition error:", err);
      setIsListening(false);
    }
  };

  // Sync dark-mode token on <html>
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
      const resp = await sendChat(prompt.trim(), confirmed, sessionId);

      if (!sessionId) setSessionId(resp.session_id);

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

  const headerGrad = 'linear-gradient(135deg, #7c3aed 0%, #db2777 55%, #f97316 100%)';

  return (
    <div
      style={{ background: 'var(--bg)', color: 'var(--text)' }}
      className="flex flex-col h-screen"
    >
      {/* ---- Header --------------------------------------------------- */}
      <header
        style={{ background: headerGrad }}
        className="shrink-0 flex items-center justify-between px-4 py-3 shadow-lg"
        role="banner"
      >
        {/* Left: avatar + title */}
        <div className="flex items-center gap-3">
          <img
            src={botAvatar}
            alt="Employee Agent Logo"
            style={{
              width: 42,
              height: 42,
              borderRadius: 14,
              objectFit: 'cover',
              boxShadow: '0 2px 10px rgba(0,0,0,0.25)',
              border: '2px solid rgba(255,255,255,0.4)',
            }}
          />
          <div>
            <h1 style={{ fontSize: '0.95rem', fontWeight: 800, color: '#fff', lineHeight: 1.2, margin: 0 }}>
              Employee Agent
            </h1>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginTop: 2 }}>
              {/* Online dot */}
              <span style={{
                width: 7, height: 7, borderRadius: '50%',
                background: '#4ade80',
                boxShadow: '0 0 0 2px rgba(74,222,128,0.4)',
                animation: 'pulseGlow 2s infinite',
                display: 'inline-block',
              }} />
              <span style={{ fontSize: '0.68rem', color: 'rgba(255,255,255,0.9)', fontWeight: 600 }}>
                Online
              </span>
            </div>
          </div>
        </div>

        {/* Right: session id + dark toggle + tools + clear */}
        <div className="flex gap-2 items-center">
          {sessionId && (
            <span style={{
              fontSize: '0.65rem',
              color: 'rgba(255,255,255,0.6)',
              fontFamily: 'monospace',
              maxWidth: 100,
              overflow: 'hidden',
              whiteSpace: 'nowrap',
              textOverflow: 'ellipsis',
            }}
              className="hidden sm:block"
            >
              {sessionId.slice(0, 8)}…
            </span>
          )}

          {/* Dark mode toggle */}
          <button
            onClick={() => setIsDark(d => !d)}
            aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
            style={{
              background: 'rgba(255,255,255,0.18)',
              border: 'none',
              color: '#fff',
              borderRadius: 10,
              padding: '5px 10px',
              cursor: 'pointer',
              fontSize: '0.8rem',
              fontWeight: 700,
              transition: 'background 0.2s',
            }}
            onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.3)')}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.18)')}
          >
            {isDark ? '☀️' : '🌙'}
          </button>

          <button
            onClick={() => setDrawerOpen(o => !o)}
            aria-label="Open MCP tool catalogue"
            style={{
              background: 'rgba(255,255,255,0.18)',
              border: 'none',
              color: '#fff',
              borderRadius: 10,
              padding: '5px 10px',
              cursor: 'pointer',
              fontSize: '0.8rem',
              fontWeight: 700,
              transition: 'background 0.2s',
            }}
            onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.3)')}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.18)')}
          >
            🔧 Tools
          </button>

          <button
            onClick={() => { setMessages([]); setSessionId(undefined); }}
            aria-label="Clear chat history"
            style={{
              background: 'rgba(255,255,255,0.18)',
              border: 'none',
              color: '#fff',
              borderRadius: 10,
              padding: '5px 10px',
              cursor: 'pointer',
              fontSize: '0.8rem',
              fontWeight: 700,
              transition: 'background 0.2s',
            }}
            onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.3)')}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.18)')}
          >
            🗑 Clear
          </button>
        </div>
      </header>

      {/* ---- Chat panel ----------------------------------------------- */}
      <ChatPanel messages={messages} onChipClick={(text) => { submit(text); }} />

      {/* ---- Input bar ------------------------------------------------ */}
      <div
        style={{
          background: 'var(--surface)',
          borderTop: '1px solid var(--border)',
          padding: '12px 16px 14px',
        }}
        className="shrink-0"
      >
        <div
          style={{
            display: 'flex',
            gap: 10,
            maxWidth: 860,
            margin: '0 auto',
            background: 'var(--surface-2)',
            borderRadius: 20,
            padding: '6px 6px 6px 16px',
            border: '1.5px solid var(--border)',
            boxShadow: '0 2px 20px rgba(124,58,237,0.08)',
            transition: 'box-shadow 0.2s',
          }}
          onFocusCapture={e => (e.currentTarget.style.boxShadow = '0 0 0 3px rgba(124,58,237,0.25)')}
          onBlurCapture={e => (e.currentTarget.style.boxShadow = '0 2px 20px rgba(124,58,237,0.08)')}
        >
          <input
            ref={inputRef}
            type="text"
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={loading}
            placeholder="e.g. Show all employees · Find employee 1001 · Add employee…"
            aria-label="Chat input"
            style={{
              flex: 1,
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: 'var(--text)',
              fontSize: '0.9rem',
              fontFamily: 'var(--font-body)',
              opacity: loading ? 0.5 : 1,
            }}
          />
          <button
            type="button"
            onClick={toggleVoiceInput}
            aria-label="Voice input"
            title={isListening ? "Listening... click to stop" : "Voice input (dictate prompt)"}
            style={{
              background: isListening ? '#dc2626' : 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 12,
              padding: '6px 12px',
              fontSize: '1rem',
              cursor: 'pointer',
              color: isListening ? '#fff' : 'var(--text)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s',
              animation: isListening ? 'pulse 1.5s infinite' : 'none',
            }}
          >
            {isListening ? '⏹️ Rec' : '🎙️'}
          </button>
          <button
            onClick={handleSend}
            disabled={loading || !input.trim()}
            aria-label="Send message"
            style={{
              background: loading || !input.trim()
                ? 'rgba(124,58,237,0.3)'
                : 'linear-gradient(135deg, #7c3aed, #db2777)',
              border: 'none',
              color: '#fff',
              borderRadius: 14,
              padding: '8px 20px',
              fontWeight: 700,
              fontSize: '0.85rem',
              fontFamily: 'var(--font-body)',
              cursor: loading || !input.trim() ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s',
              whiteSpace: 'nowrap',
            }}
          >
            {loading ? '…' : 'Send ↑'}
          </button>
        </div>
      </div>

      {/* ---- Overlays ------------------------------------------------- */}
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
