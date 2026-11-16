// =============================================================================
// components/ConfirmationModal.tsx
// Shown for add_employee / update_employee before the action executes.
// =============================================================================

interface Props {
  intent: string;
  params: Record<string, unknown>;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationModal({ intent, params, onConfirm, onCancel }: Props) {
  const label = intent === 'add_employee' ? 'Add Employee' : 'Update Employee';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-slate-800 border border-slate-600 rounded-xl shadow-2xl p-6 w-full max-w-md mx-4">
        <div className="flex items-center gap-2 mb-4">
          <span className="text-2xl">⚠️</span>
          <h2 className="text-lg font-semibold text-slate-100">{label} — Confirm</h2>
        </div>

        <p className="text-sm text-slate-400 mb-4">
          Review the extracted parameters below. Click <strong className="text-slate-200">Confirm</strong> to execute
          this write operation on the database.
        </p>

        <div className="bg-slate-900 rounded-lg p-4 mb-5 space-y-1.5">
          {Object.entries(params).map(([k, v]) => (
            <div key={k} className="flex justify-between text-sm">
              <span className="text-slate-400 font-medium">{k}</span>
              <span className="text-slate-200 font-mono">{String(v)}</span>
            </div>
          ))}
        </div>

        <div className="flex gap-3">
          <button
            onClick={onConfirm}
            className="flex-1 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold
                       py-2 px-4 rounded-lg transition-colors text-sm"
          >
            ✓ Confirm
          </button>
          <button
            onClick={onCancel}
            className="flex-1 bg-slate-700 hover:bg-slate-600 text-slate-200 font-semibold
                       py-2 px-4 rounded-lg transition-colors text-sm"
          >
            ✗ Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
