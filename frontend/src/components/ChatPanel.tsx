// =============================================================================
// components/ChatPanel.tsx — Main chat message timeline
// =============================================================================

import { useRef, useEffect } from 'react';
import type { ChatMessage } from '../types';
import { StepTracePanel } from './StepTrace';
import { EmployeeTable } from './EmployeeTable';

function IntentBadge({ intent }: { intent: string }) {
  const colour = intent.startsWith('add') || intent.startsWith('update')
    ? 'bg-amber-900 text-amber-300 border border-amber-700'
    : 'bg-sky-900 text-sky-300 border border-sky-700';
  return (
    <span className={`inline-block text-xs px-2 py-0.5 rounded font-mono ${colour}`}>
      {intent}
    </span>
  );
}

function AgentBubble({ msg }: { msg: ChatMessage }) {
  const r = msg.response;

  if (msg.pending) {
    return (
      <div className="flex gap-3 items-start">
        <span className="mt-1 shrink-0 text-lg">🤖</span>
        <div className="bg-slate-800 rounded-xl rounded-tl-sm px-4 py-3 text-slate-300 text-sm max-w-2xl">
          <span className="animate-pulse">Thinking…</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex gap-3 items-start">
      <span className="mt-1 shrink-0 text-lg">🤖</span>
      <div className="bg-slate-800 border border-slate-700 rounded-xl rounded-tl-sm
                      px-4 py-3 text-slate-200 text-sm max-w-2xl w-full">

        {/* Intent badge */}
        {r?.intent && r.intent !== 'unknown' && (
          <div className="mb-2">
            <IntentBadge intent={r.intent} />
            {r.tool && (
              <span className="ml-2 text-xs text-slate-500">→ {r.tool}</span>
            )}
          </div>
        )}

        {/* Main message */}
        <p className={`mb-1 ${r?.success === false ? 'text-red-400' : 'text-slate-100'}`}>
          {msg.content}
        </p>

        {/* Result table / JSON */}
        {r?.result !== undefined && r?.result !== null && (
          <EmployeeTable data={r.result} />
        )}

        {/* Step trace accordion */}
        {r?.steps && r.steps.length > 0 && (
          <StepTracePanel steps={r.steps} />
        )}

        <div className="mt-2 text-xs text-slate-600">
          {msg.timestamp.toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}

function UserBubble({ msg }: { msg: ChatMessage }) {
  return (
    <div className="flex gap-3 items-start justify-end">
      <div className="bg-sky-700 text-white rounded-xl rounded-tr-sm px-4 py-3
                      text-sm max-w-xl">
        {msg.content}
        <div className="mt-1 text-xs text-sky-300/70">
          {msg.timestamp.toLocaleTimeString()}
        </div>
      </div>
      <span className="mt-1 shrink-0 text-lg">🧑‍💻</span>
    </div>
  );
}

interface Props {
  messages: ChatMessage[];
}

export function ChatPanel({ messages }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  if (messages.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-4 text-slate-500 select-none">
        <span className="text-5xl">🤖</span>
        <div className="text-center">
          <p className="font-semibold text-slate-300 text-lg">Employee Management Agent</p>
          <p className="text-sm mt-1">Ask me anything about your employees</p>
          <div className="mt-4 grid grid-cols-2 gap-2 text-xs text-left max-w-sm">
            {[
              'Show all employees',
              'Find employee 1001',
              'Search employees in IT',
              'Count active employees',
              'Show all departments',
              'Add employee Jane Doe email j.doe@co.com dept IT designation Engineer',
              'Update employee 1001 status to Inactive',
              'Update employee 1002 salary to 95000',
            ].map(hint => (
              <div key={hint} className="bg-slate-800 rounded px-2 py-1.5 text-slate-400 font-mono">
                {hint}
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
      {messages.map(msg =>
        msg.role === 'user'
          ? <UserBubble key={msg.id} msg={msg} />
          : <AgentBubble key={msg.id} msg={msg} />
      )}
      <div ref={bottomRef} />
    </div>
  );
}
