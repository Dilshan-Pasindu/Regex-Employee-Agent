// =============================================================================
// components/StepTrace.tsx — Execution step accordion + ExecutionPlan panel (Phase 7)
// =============================================================================

import { useState } from 'react';
import { ChevronRight, Zap, Target, Search, CheckCircle, AlertTriangle, XCircle, List, Filter, Columns, RotateCcw, Eye, FlaskConical } from 'lucide-react';
import type { StepTrace, ExecutionPlan } from '../types';

// Extended step label → icon/colour mapping
const STEP_META: Record<string, { icon: React.ReactNode; color: string; bg: string }> = {
  'Intent Recognized':          { icon: <Target size={12} />,       color: '#7c3aed', bg: 'rgba(124,58,237,0.12)' },
  'Parameters Extracted':       { icon: <Search size={12} />,       color: '#2563eb', bg: 'rgba(37,99,235,0.12)'  },
  'Execution Plan Generated':   { icon: <List size={12} />,         color: '#0891b2', bg: 'rgba(8,145,178,0.12)'  },
  'MCP Tool Invoked':           { icon: <Zap size={12} />,          color: '#d97706', bg: 'rgba(217,119,6,0.12)'  },
  'MCP Tool Dispatched':        { icon: <Zap size={12} />,          color: '#d97706', bg: 'rgba(217,119,6,0.12)'  },
  'Result Received':            { icon: <CheckCircle size={12} />,  color: '#059669', bg: 'rgba(5,150,105,0.12)'  },
  'Field Projection':           { icon: <Columns size={12} />,      color: '#7c3aed', bg: 'rgba(124,58,237,0.12)' },
  'In-Memory Projection':       { icon: <Filter size={12} />,       color: '#7c3aed', bg: 'rgba(124,58,237,0.12)' },
  'Context Resolved':           { icon: <RotateCcw size={12} />,    color: '#0891b2', bg: 'rgba(8,145,178,0.12)'  },
  'Context Cleared':            { icon: <RotateCcw size={12} />,    color: '#6b7280', bg: 'rgba(107,114,128,0.12)' },
  'Awaiting Confirmation':      { icon: <AlertTriangle size={12} />, color: '#d97706', bg: 'rgba(217,119,6,0.12)' },
  'Confirmation Verified':      { icon: <CheckCircle size={12} />,  color: '#059669', bg: 'rgba(5,150,105,0.12)'  },
  'Lookup':                     { icon: <Search size={12} />,       color: '#2563eb', bg: 'rgba(37,99,235,0.12)'  },
  'Error':                      { icon: <XCircle size={12} />,      color: '#dc2626', bg: 'rgba(220,38,38,0.12)'  },
};

function getStepMeta(label: string) {
  if (STEP_META[label]) return STEP_META[label];
  // Fuzzy fallback
  if (label.toLowerCase().includes('error')) return STEP_META['Error'];
  if (label.toLowerCase().includes('confirm')) return STEP_META['Awaiting Confirmation'];
  if (label.toLowerCase().includes('mcp') || label.toLowerCase().includes('tool')) return STEP_META['MCP Tool Invoked'];
  if (label.toLowerCase().includes('intent')) return STEP_META['Intent Recognized'];
  if (label.toLowerCase().includes('param')) return STEP_META['Parameters Extracted'];
  if (label.toLowerCase().includes('result')) return STEP_META['Result Received'];
  if (label.toLowerCase().includes('context')) return STEP_META['Context Resolved'];
  return { icon: <FlaskConical size={12} />, color: 'var(--text-muted)', bg: 'var(--surface-2)' };
}

function formatValue(val: unknown): string {
  if (typeof val === 'string') return val;
  if (typeof val === 'object' && val !== null) return JSON.stringify(val, null, 2);
  return String(val);
}

// ---------- Step count pill -----------------------------------------------
function StepCountPill({ count }: { count: number }) {
  return (
    <span style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: 3,
      fontSize: '0.68rem',
      fontWeight: 700,
      padding: '1px 7px',
      borderRadius: 99,
      background: 'rgba(124,58,237,0.1)',
      color: '#7c3aed',
      border: '1px solid rgba(124,58,237,0.2)',
      fontFamily: 'monospace',
    }}>
      {count}
    </span>
  );
}

// ---------- Execution step trace accordion --------------------------------
interface StepProps { steps: StepTrace[] }

export function StepTracePanel({ steps }: StepProps) {
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
        <ChevronRight size={13} style={{ transition: 'transform 0.2s', transform: open ? 'rotate(90deg)' : 'rotate(0deg)' }} />
        {open ? 'Hide' : 'Show'} execution trace
        <StepCountPill count={steps.length} />
      </button>

      {open && (
        <div style={{
          marginTop: 8,
          borderLeft: '2px solid var(--border)',
          paddingLeft: 12,
          display: 'flex', flexDirection: 'column', gap: 6,
        }}>
          {steps.map(step => {
            const meta = getStepMeta(step.label);
            return (
              <div key={step.step} style={{ display: 'flex', gap: 8, alignItems: 'flex-start' }}>
                {/* Icon badge */}
                <div style={{
                  width: 22, height: 22, borderRadius: 6, flexShrink: 0,
                  background: meta.bg,
                  color: meta.color,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
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

// ---------- Execution plan accordion (Phase 7) ----------------------------
interface PlanProps { plan: ExecutionPlan }

export function ExecutionPlanPanel({ plan }: PlanProps) {
  const [open, setOpen] = useState(false);
  if (!plan?.steps?.length) return null;

  return (
    <div style={{ marginTop: 8 }}>
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
        onMouseEnter={e => (e.currentTarget.style.color = '#0891b2')}
        onMouseLeave={e => (e.currentTarget.style.color = 'var(--text-muted)')}
        aria-expanded={open}
      >
        <Eye size={13} />
        {open ? 'Hide' : 'View'} execution plan
        {plan.intent_label && (
          <span style={{ fontFamily: 'monospace', fontSize: '0.68rem', color: '#0891b2' }}>
            [{plan.intent_label}]
          </span>
        )}
        <StepCountPill count={plan.steps.length} />
      </button>

      {open && (
        <div style={{
          marginTop: 8,
          border: '1px solid rgba(8,145,178,0.2)',
          borderRadius: 10,
          overflow: 'hidden',
          background: 'rgba(8,145,178,0.03)',
        }}>
          {plan.steps.map((step, idx) => (
            <div
              key={step.step_number ?? idx}
              style={{
                display: 'flex',
                gap: 10,
                alignItems: 'flex-start',
                padding: '8px 12px',
                borderBottom: idx < plan.steps.length - 1 ? '1px solid rgba(8,145,178,0.1)' : 'none',
              }}
            >
              {/* Step number badge */}
              <div style={{
                width: 20, height: 20, borderRadius: 6, flexShrink: 0,
                background: 'rgba(8,145,178,0.15)',
                color: '#0891b2',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                fontSize: '0.65rem', fontWeight: 800, fontFamily: 'monospace',
              }}>
                {step.step_number ?? idx + 1}
              </div>
              <div style={{ fontSize: '0.74rem', lineHeight: 1.5, minWidth: 0 }}>
                <span style={{ color: 'var(--text)', fontWeight: 600 }}>{step.description}</span>
                {step.tool_name && (
                  <span style={{
                    marginLeft: 6,
                    fontSize: '0.66rem',
                    fontFamily: 'monospace',
                    color: '#d97706',
                    background: 'rgba(217,119,6,0.1)',
                    border: '1px solid rgba(217,119,6,0.2)',
                    borderRadius: 4,
                    padding: '0 5px',
                  }}>
                    {step.tool_name}
                  </span>
                )}
                {step.arguments && Object.keys(step.arguments).length > 0 && (
                  <div style={{
                    marginTop: 3,
                    fontFamily: 'monospace',
                    fontSize: '0.65rem',
                    color: 'var(--text-muted)',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-all',
                  }}>
                    {JSON.stringify(step.arguments, null, 2)}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
