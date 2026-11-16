// =============================================================================
// components/StepTrace.tsx — Animated accordion showing execution steps
// =============================================================================

import { useState } from 'react';
import type { StepTrace } from '../types';

const STEP_ICONS: Record<string, string> = {
  'Intent Recognized':     '🎯',
  'Parameters Extracted':  '🔍',
  'MCP Tool Dispatched':   '⚡',
  'Result Received':       '✅',
  'Awaiting Confirmation': '⚠️',
  'Error':                 '❌',
};

function formatValue(val: unknown): string {
  if (typeof val === 'string') return val;
  if (typeof val === 'object' && val !== null) {
    return JSON.stringify(val, null, 2);
  }
  return String(val);
}

interface Props {
  steps: StepTrace[];
}

export function StepTracePanel({ steps }: Props) {
  const [open, setOpen] = useState(false);

  if (!steps || steps.length === 0) return null;

  return (
    <div className="mt-2">
      <button
        onClick={() => setOpen(o => !o)}
        className="flex items-center gap-1 text-xs text-slate-400 hover:text-slate-200 transition-colors"
      >
        <span className={`transform transition-transform ${open ? 'rotate-90' : ''}`}>▶</span>
        {open ? 'Hide' : 'Show'} execution trace ({steps.length} steps)
      </button>

      {open && (
        <div className="mt-2 space-y-1 border-l-2 border-slate-600 pl-3">
          {steps.map((step) => (
            <div key={step.step} className="flex gap-2 items-start text-xs">
              <span className="shrink-0 w-5 text-center">
                {STEP_ICONS[step.label] ?? '→'}
              </span>
              <div className="min-w-0">
                <span className="text-slate-400 font-medium">{step.label}: </span>
                <span className={`font-mono break-all ${
                  step.label === 'Error' ? 'text-red-400' : 'text-emerald-300'
                }`}>
                  {formatValue(step.value)}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
