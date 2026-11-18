// =============================================================================
// components/ConfirmationModal.tsx — Write-op confirmation (redesign)
// =============================================================================

import { UserPlus, Pencil, Check, X } from 'lucide-react';

interface Props {
  intent: string;
  params: Record<string, unknown>;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationModal({ intent, params, onConfirm, onCancel }: Props) {
  const label = intent === 'add_employee' ? 'Add Employee' : 'Update Employee';
  const isAdd = intent === 'add_employee';

  return (
    <div
      style={{
        position: 'fixed', inset: 0, zIndex: 50,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        background: 'rgba(0,0,0,0.55)',
        backdropFilter: 'blur(8px)',
        WebkitBackdropFilter: 'blur(8px)',
        padding: 16,
      }}
      role="dialog"
      aria-modal="true"
      aria-label={`${label} confirmation`}
    >
      <div style={{
        background: 'var(--surface)',
        border: '1px solid var(--border)',
        borderRadius: 20,
        boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
        width: '100%',
        maxWidth: 440,
        overflow: 'hidden',
      }}>
        {/* Gradient top bar */}
        <div style={{
          height: 4,
          background: isAdd
            ? 'linear-gradient(90deg,#7c3aed,#ec4899)'
            : 'linear-gradient(90deg,#f97316,#db2777)',
        }} />

        <div style={{ padding: '24px 24px 20px' }}>
          {/* Title */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 12 }}>
            <div style={{
              width: 40, height: 40, borderRadius: 12,
              background: isAdd ? 'rgba(124,58,237,0.12)' : 'rgba(249,115,22,0.12)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              {isAdd ? <UserPlus size={20} color="#7c3aed" /> : <Pencil size={20} color="#f97316" />}
            </div>
            <div>
              <h2 style={{ margin: 0, fontSize: '1rem', fontWeight: 800, color: 'var(--text)' }}>
                {label}
              </h2>
              <p style={{ margin: 0, fontSize: '0.72rem', color: 'var(--text-muted)' }}>
                Confirm before writing to database
              </p>
            </div>
          </div>

          {/* Body text */}
          <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', margin: '0 0 16px' }}>
            Review the extracted parameters below. Click{' '}
            <strong style={{ color: 'var(--text)' }}>Confirm</strong> to execute
            this write operation.
          </p>

          {/* Params card */}
          <div style={{
            background: 'var(--surface-2)',
            border: '1px solid var(--border)',
            borderRadius: 12,
            padding: '12px 16px',
            marginBottom: 20,
            display: 'flex',
            flexDirection: 'column',
            gap: 8,
          }}>
            {Object.entries(params).map(([k, v]) => (
              <div key={k} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '0.82rem' }}>
                <span style={{ color: 'var(--text-muted)', fontWeight: 600 }}>{k}</span>
                <span style={{ color: 'var(--text)', fontFamily: 'monospace', fontSize: '0.8rem' }}>
                  {String(v)}
                </span>
              </div>
            ))}
          </div>

          {/* Action buttons */}
          <div style={{ display: 'flex', gap: 10 }}>
            <button
              onClick={onConfirm}
              style={{
                flex: 1,
                background: 'linear-gradient(135deg, #7c3aed, #ec4899)',
                border: 'none',
                color: '#fff',
                borderRadius: 12,
                padding: '10px',
                fontWeight: 700,
                fontSize: '0.85rem',
                fontFamily: 'var(--font-body)',
                cursor: 'pointer',
                transition: 'opacity 0.15s',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: 6,
              }}
              onMouseEnter={e => (e.currentTarget.style.opacity = '0.88')}
              onMouseLeave={e => (e.currentTarget.style.opacity = '1')}
            >
              <Check size={15} /> Confirm
            </button>
            <button
              onClick={onCancel}
              style={{
                flex: 1,
                background: 'var(--surface-2)',
                border: '1.5px solid var(--border)',
                color: 'var(--text)',
                borderRadius: 12,
                padding: '10px',
                fontWeight: 700,
                fontSize: '0.85rem',
                fontFamily: 'var(--font-body)',
                cursor: 'pointer',
                transition: 'background 0.15s',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: 6,
              }}
              onMouseEnter={e => (e.currentTarget.style.background = 'var(--border)')}
              onMouseLeave={e => (e.currentTarget.style.background = 'var(--surface-2)')}
            >
              <X size={15} /> Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
