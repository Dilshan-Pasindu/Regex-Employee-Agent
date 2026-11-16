// =============================================================================
// components/StepTrace.tsx — Execution step accordion (redesign)
// =============================================================================

import { useState } from 'react';
import type { StepTrace } from '../types';

const STEP_META: Record<string, { icon: string; color: string; bg: string }> = {
  'Intent Recognized':     { icon: '🎯', color: '#7c3aed', bg: 'rgba(124,58,237,0.1)' },
  'Parameters Extracted':  { icon: '🔍', color: '#2563eb', bg: 'rgba(37,99,235,0.1)'  },
  'MCP Tool Dispatched':   { icon: '⚡', color: '#d97706', bg: 'rgba(217,119,6,0.1)'  },
  'Result Received':       { icon: '✅', color: '#059669', bg: 'rgba(5,150,105,0.1)'  },
  'Awaiting Confirmation': { icon: '⚠️', color: '#d97706', bg: 'rgba(217,119,6,0.1)'  },
  'Error':                 { icon: '❌', color: '#dc2626', bg: 'rgba(220,38,38,0.1)'  },
};

function formatValue(val: unknown): string {
  if (typeof val === 'string') return val;
  if (typeof val === 'object' && val !== null) return JSON.stringify(val, null, 2);
  return String(val);
}

interface Props {
  steps: StepTrace[];
}

export function StepTracePanel({ steps }: Props) {
  const [open, setOpen] = useState(false);

  if (!steps || steps.length === 0) return null;

  return (
    <div style={{ marginTop: 10 }}>
      <button
        onClick={() => setOpen(o => !o)}
        style={{
          display: 'flex', alignItems: 'center', gap: 6,
          background: 'none', border: 'none',
          color: 'var(--text-muted)',
          fontSize: '0.74rem', fontWeight: 600,
          cursor: 'pointer', padding: 0,
          fontFamily: 'var(--font-body)',
          transition: 'color 0.15s',
        }}
        onMouseEnter={e => (e.currentTarget.style.color = '#7c3aed')}
        onMouseLeave={e => (e.currentTarget.style.color = 'var(--text-muted)')}
        aria-expanded={open}
      >
        <span style={{
          display: 'inline-block',
          transition: 'transform 0.2s',
          transform: open ? 'rotate(90deg)' : 'rotate(0deg)',
        }}>▶</span>
        {open ? 'Hide' : 'Show'} execution trace ({steps.length} steps)
      </button>

      {open && (
        <div style={{
          marginTop: 8,
          borderLeft: '2px solid var(--border)',
          paddingLeft: 12,
          display: 'flex', flexDirection: 'column', gap: 6,
        }}>
          {steps.map(step => {
            const meta = STEP_META[step.label] ?? { icon: '→', color: 'var(--text-muted)', bg: 'var(--surface-2)' };
            return (
              <div key={step.step} style={{ display: 'flex', gap: 8, alignItems: 'flex-start' }}>
                {/* Icon badge */}
                <div style={{
                  width: 22, height: 22, borderRadius: 6, flexShrink: 0,
                  background: meta.bg,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  fontSize: 12,
                }}>
                  {meta.icon}
                </div>
                <div style={{ fontSize: '0.74rem', lineHeight: 1.5, minWidth: 0 }}>
                  <span style={{ color: meta.color, fontWeight: 700 }}>{step.label}: </span>
                  <span style={{
                    fontFamily: 'monospace',
                    color: step.label === 'Error' ? '#ef4444' : 'var(--text)',
                    wordBreak: 'break-all',
                    whiteSpace: 'pre-wrap',
                  }}>
                    {formatValue(step.value)}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
