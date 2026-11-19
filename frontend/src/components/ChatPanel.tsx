// =============================================================================
// components/ChatPanel.tsx — Animated chat timeline (Phase 7 refinements)
// Adds: ExecutionPlan panel, Write-Safety amber badge, Error recovery UX,
//       correlation ID footer, improved step icons
// =============================================================================

import { useRef, useEffect, useState } from 'react';
import { Lightbulb, ArrowDown, AlertTriangle, CheckCircle2, XCircle } from 'lucide-react';
import type { ChatMessage } from '../types';
import { StepTracePanel, ExecutionPlanPanel } from './StepTrace';
import { EmployeeTable } from './EmployeeTable';
import { MarkdownRenderer } from './MarkdownRenderer';
import botAvatar from '../assets/bot-avatar.jpg';

// ---------- Intent badge --------------------------------------------------
function IntentBadge({ intent }: { intent: string }) {
  const isWrite = intent.startsWith('add') || intent.startsWith('update') || intent.startsWith('batch');
  const isUnknown = intent === 'unknown';
  const color = isUnknown ? '#6b7280' : isWrite ? '#d97706' : '#7c3aed';
  const bg = isUnknown ? 'rgba(107,114,128,0.12)' : isWrite ? 'rgba(245,158,11,0.12)' : 'rgba(124,58,237,0.12)';
  const border = isUnknown ? 'rgba(107,114,128,0.25)' : isWrite ? 'rgba(245,158,11,0.3)' : 'rgba(124,58,237,0.3)';
  return (
    <span style={{
      display: 'inline-block',
      fontSize: '0.68rem', fontWeight: 700, fontFamily: 'monospace',
      padding: '1px 8px', borderRadius: 99,
      background: bg, color, border: `1px solid ${border}`,
    }}>
      {intent}
    </span>
  );
}

// ---------- Write-safety badge (amber) ------------------------------------
function PendingConfirmationBadge() {
  return (
    <div style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      fontSize: '0.72rem', fontWeight: 700,
      padding: '3px 10px', borderRadius: 8,
      background: 'rgba(217,119,6,0.1)',
      color: '#d97706',
      border: '1px solid rgba(217,119,6,0.3)',
      marginBottom: 8,
    }}>
      <AlertTriangle size={12} />
      Awaiting Confirmation — Write not executed yet
    </div>
  );
}

// ---------- Success / failure pill ----------------------------------------
function StatusPill({ success }: { success: boolean }) {
  return success ? (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 3,
      fontSize: '0.66rem', fontWeight: 700,
      padding: '1px 7px', borderRadius: 99,
      background: 'rgba(5,150,105,0.1)', color: '#059669',
      border: '1px solid rgba(5,150,105,0.25)',
    }}>
      <CheckCircle2 size={10} /> OK
    </span>
  ) : (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 3,
      fontSize: '0.66rem', fontWeight: 700,
      padding: '1px 7px', borderRadius: 99,
      background: 'rgba(220,38,38,0.1)', color: '#dc2626',
      border: '1px solid rgba(220,38,38,0.25)',
    }}>
      <XCircle size={10} /> Failed
    </span>
  );
}

// ---------- Typing indicator ----------------------------------------------
function TypingIndicator() {
  return (
    <div className="flex gap-3 items-start msg-enter">
      <img
        src={botAvatar} alt="Bot"
        style={{
          width: 34, height: 34, borderRadius: 11, objectFit: 'cover',
          flexShrink: 0, marginTop: 2,
          boxShadow: '0 2px 8px rgba(124,58,237,0.25)',
          border: '1.5px solid rgba(124,58,237,0.3)',
        }}
      />
      <div style={{
        background: 'var(--bot-bubble)', color: 'var(--bot-text)',
        borderRadius: '18px 18px 18px 4px', padding: '12px 16px',
        display: 'flex', alignItems: 'center', gap: 4,
      }}>
        <div className="dot-bounce" aria-label="Thinking">
          <span /><span /><span />
        </div>
      </div>
    </div>
  );
}

// ---------- Agent bubble --------------------------------------------------
function AgentBubble({ msg, onChipClick }: { msg: ChatMessage; onChipClick?: (text: string) => void }) {
  const r = msg.response;

  if (msg.pending) return <TypingIndicator />;

  const isPendingConfirm = r?.requires_confirmation === true;
  const isError = r?.success === false && !isPendingConfirm;

  return (
    <div className="flex gap-3 items-start msg-enter">
      <img
        src={botAvatar} alt="Bot"
        style={{
          width: 34, height: 34, borderRadius: 11, objectFit: 'cover',
          flexShrink: 0, marginTop: 2,
          boxShadow: '0 2px 8px rgba(124,58,237,0.25)',
          border: '1.5px solid rgba(124,58,237,0.3)',
        }}
      />

      <div style={{
        background: isError ? 'rgba(220,38,38,0.05)' : isPendingConfirm ? 'rgba(217,119,6,0.05)' : 'var(--bot-bubble)',
        color: 'var(--bot-text)',
        borderRadius: '18px 18px 18px 4px',
        padding: '12px 16px',
        maxWidth: 700,
        width: '100%',
        boxShadow: isError
          ? '0 1px 6px rgba(220,38,38,0.1)'
          : isPendingConfirm
          ? '0 1px 6px rgba(217,119,6,0.1)'
          : '0 1px 6px rgba(0,0,0,0.06)',
        border: isError
          ? '1px solid rgba(220,38,38,0.2)'
          : isPendingConfirm
          ? '1px solid rgba(217,119,6,0.2)'
          : 'none',
      }}>

        {/* Write-safety banner */}
        {isPendingConfirm && <PendingConfirmationBadge />}

        {/* Intent + tool row */}
        {r?.intent && r.intent !== 'unknown' && (
          <div style={{ marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
            <IntentBadge intent={r.intent} />
            {r.tool && (
              <span style={{ fontSize: '0.72rem', color: 'var(--text-muted)', fontFamily: 'monospace' }}>
                → {r.tool}
              </span>
            )}
            {r.success !== undefined && <StatusPill success={r.success} />}
          </div>
        )}

        {/* Main message */}
        <div style={{ color: isError ? '#ef4444' : 'var(--bot-text)' }}>
          <MarkdownRenderer content={msg.content} />
        </div>

        {/* Suggestion chips */}
        {r?.suggestions && r.suggestions.length > 0 && onChipClick && (
          <div style={{ marginTop: 10, display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {r.suggestions.map((sug, i) => (
              <button
                key={i}
                onClick={() => onChipClick(sug)}
                style={{
                  background: 'var(--surface)',
                  border: '1.5px solid var(--border)',
                  color: 'var(--text)',
                  borderRadius: 99,
                  padding: '5px 14px',
                  fontSize: '0.78rem', fontWeight: 600,
                  fontFamily: 'var(--font-body)',
                  cursor: 'pointer',
                  display: 'flex', alignItems: 'center', gap: 6,
                  transition: 'all 0.15s',
                  boxShadow: '0 1px 4px rgba(124,58,237,0.1)',
                }}
                onMouseEnter={e => {
                  e.currentTarget.style.background = 'linear-gradient(135deg,#7c3aed,#ec4899)';
                  e.currentTarget.style.color = '#fff';
                  e.currentTarget.style.borderColor = 'transparent';
                }}
                onMouseLeave={e => {
                  e.currentTarget.style.background = 'var(--surface)';
                  e.currentTarget.style.color = 'var(--text)';
                  e.currentTarget.style.borderColor = 'var(--border)';
                }}
              >
                <Lightbulb size={13} style={{ flexShrink: 0 }} />
                <span>{sug}</span>
              </button>
            ))}
          </div>
        )}

        {/* Result table */}
        {r?.result !== undefined && r?.result !== null && (
          <EmployeeTable data={r.result} />
        )}

        {/* Execution plan accordion (Phase 7) */}
        {r?.execution_plan && <ExecutionPlanPanel plan={r.execution_plan} />}

        {/* Step trace accordion */}
        {r?.steps && r.steps.length > 0 && <StepTracePanel steps={r.steps} />}

        {/* Footer row: timestamp + correlation ID */}
        <div style={{
          marginTop: 6, display: 'flex', alignItems: 'center', gap: 8,
          fontSize: '0.65rem', color: 'var(--text-muted)',
        }}>
          <span>{msg.timestamp.toLocaleTimeString()}</span>
          {r?.correlation_id && (
            <span style={{ fontFamily: 'monospace', opacity: 0.6 }} title="Correlation ID">
              corr:{r.correlation_id.slice(0, 12)}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

// ---------- User bubble ---------------------------------------------------
function UserBubble({ msg }: { msg: ChatMessage }) {
  return (
    <div className="flex gap-3 items-end justify-end msg-enter">
      <div style={{
        background: 'linear-gradient(135deg, #7c3aed, #ec4899)',
        color: '#fff',
        borderRadius: '18px 18px 4px 18px',
        padding: '10px 16px',
        maxWidth: 520,
        fontSize: '0.875rem',
        boxShadow: '0 2px 12px rgba(124,58,237,0.3)',
        lineHeight: 1.5,
      }}>
        {msg.content}
        <div style={{ marginTop: 4, fontSize: '0.65rem', color: 'rgba(255,255,255,0.65)', textAlign: 'right' }}>
          {msg.timestamp.toLocaleTimeString()}
        </div>
      </div>
      <div style={{
        width: 32, height: 32, borderRadius: 10, flexShrink: 0,
        background: 'rgba(124,58,237,0.15)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 16,
      }}>🧑‍💻</div>
    </div>
  );
}

// ---------- Suggestion chips (welcome screen) -----------------------------
const CHIPS = [
  { label: '👥 All employees',   text: 'Show all employees' },
  { label: '🔍 Find #1001',      text: 'Find employee 1001' },
  { label: '🏢 IT department',   text: 'Search employees in IT' },
  { label: '📊 All departments', text: 'Show all departments' },
];

function WelcomeScreen({ onChip }: { onChip: (text: string) => void }) {
  return (
    <div style={{
      flex: 1, display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center',
      padding: '24px 16px', userSelect: 'none',
      gap: 20,
    }}>
      {/* Avatar */}
      <div style={{ position: 'relative' }}>
        <img
          src={botAvatar}
          alt="Employee Agent Logo"
          style={{
            width: 88, height: 88, borderRadius: 28, objectFit: 'cover',
            boxShadow: '0 12px 36px rgba(124,58,237,0.35)',
            border: '3px solid rgba(255,255,255,0.8)',
            display: 'block',
          }}
        />
        <div style={{
          position: 'absolute', bottom: 2, right: 2,
          width: 18, height: 18, borderRadius: '50%',
          background: '#10b981', border: '3px solid var(--surface)',
        }} />
      </div>

      {/* Greeting */}
      <div style={{ textAlign: 'center' }}>
        <h2 className="grad-text" style={{ fontSize: '1.4rem', fontWeight: 800, margin: '0 0 6px' }}>
          Employee Management Agent
        </h2>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem', margin: 0 }}>
          Manage your organization's employees, departments, and payroll.
        </p>
      </div>

      {/* Chips */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10, justifyContent: 'center', maxWidth: 480 }}>
        {CHIPS.map(chip => (
          <button
            key={chip.text}
            onClick={() => onChip(chip.text)}
            style={{
              background: 'var(--surface)',
              border: '1.5px solid var(--border)',
              color: 'var(--text)',
              borderRadius: 99, padding: '8px 18px',
              fontSize: '0.82rem', fontWeight: 600,
              fontFamily: 'var(--font-body)',
              cursor: 'pointer', transition: 'all 0.18s',
              boxShadow: '0 1px 6px rgba(124,58,237,0.08)',
            }}
            onMouseEnter={e => {
              e.currentTarget.style.background = 'linear-gradient(135deg,#7c3aed,#ec4899)';
              e.currentTarget.style.color = '#fff';
              e.currentTarget.style.borderColor = 'transparent';
              e.currentTarget.style.boxShadow = '0 4px 14px rgba(124,58,237,0.35)';
            }}
            onMouseLeave={e => {
              e.currentTarget.style.background = 'var(--surface)';
              e.currentTarget.style.color = 'var(--text)';
              e.currentTarget.style.borderColor = 'var(--border)';
              e.currentTarget.style.boxShadow = '0 1px 6px rgba(124,58,237,0.08)';
            }}
          >
            {chip.label}
          </button>
        ))}
      </div>

      {/* Hint grid */}
      <div style={{
        display: 'grid', gridTemplateColumns: 'repeat(2,1fr)',
        gap: 8, maxWidth: 480, width: '100%',
      }}>
        {[
          'Add employee Jane Doe email j@co.com dept IT designation Engineer',
          'Update employee 1001 status to Inactive',
          'Update employee 1002 salary to 95000',
          'Count active employees',
        ].map(hint => (
          <div
            key={hint}
            style={{
              background: 'var(--surface-2)',
              border: '1px solid var(--border)',
              borderRadius: 10,
              padding: '7px 10px',
              fontSize: '0.7rem',
              color: 'var(--text-muted)',
              fontFamily: 'monospace',
              cursor: 'pointer',
              transition: 'border-color 0.15s',
            }}
            onClick={() => onChip(hint)}
            onMouseEnter={e => (e.currentTarget.style.borderColor = '#7c3aed')}
            onMouseLeave={e => (e.currentTarget.style.borderColor = 'var(--border)')}
          >
            {hint}
          </div>
        ))}
      </div>
    </div>
  );
}

// ---------- ChatPanel (main export) ---------------------------------------
interface Props {
  messages: ChatMessage[];
  onChipClick: (text: string) => void;
}

export function ChatPanel({ messages, onChipClick }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showScrollBtn, setShowScrollBtn] = useState(false);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  function handleScroll() {
    const el = scrollRef.current;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
    setShowScrollBtn(!nearBottom);
  }

  function scrollToBottom() {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }

  if (messages.length === 0) {
    return <WelcomeScreen onChip={onChipClick} />;
  }

  return (
    <div style={{ flex: 1, position: 'relative', overflow: 'hidden' }}>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        style={{
          height: '100%', overflowY: 'auto',
          padding: '20px 16px',
          display: 'flex', flexDirection: 'column', gap: 14,
        }}
      >
        {messages.map(msg =>
          msg.role === 'user'
            ? <UserBubble key={msg.id} msg={msg} />
            : <AgentBubble key={msg.id} msg={msg} onChipClick={onChipClick} />
        )}
        <div ref={bottomRef} />
      </div>

      {/* Scroll-to-bottom FAB */}
      {showScrollBtn && (
        <button
          onClick={scrollToBottom}
          aria-label="Scroll to latest message"
          style={{
            position: 'absolute', bottom: 16, left: '50%',
            transform: 'translateX(-50%)',
            background: 'linear-gradient(135deg,#7c3aed,#db2777)',
            color: '#fff', border: 'none', borderRadius: 99,
            padding: '6px 16px',
            fontSize: '0.78rem', fontWeight: 700,
            fontFamily: 'var(--font-body)',
            cursor: 'pointer',
            boxShadow: '0 4px 16px rgba(124,58,237,0.4)',
            transition: 'opacity 0.2s',
            display: 'flex', alignItems: 'center', gap: 6,
          }}
        >
          <ArrowDown size={14} />
          <span>Latest</span>
        </button>
      )}
    </div>
  );
}
