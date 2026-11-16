// =============================================================================
// components/EmployeeTable.tsx — Sortable table with status badges
// =============================================================================

import { useState } from 'react';

type Row = Record<string, unknown>;

const STATUS_BADGE: Record<string, string> = {
  Active:    'bg-emerald-900 text-emerald-300 border border-emerald-700',
  Inactive:  'bg-red-900 text-red-300 border border-red-700',
  'On Leave': 'bg-yellow-900 text-yellow-300 border border-yellow-700',
};

interface Props {
  data: unknown;
}

export function EmployeeTable({ data }: Props) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [asc, setAsc] = useState(true);

  // Normalise to array
  const rows: Row[] = Array.isArray(data)
    ? (data as Row[])
    : data && typeof data === 'object'
    ? [data as Row]
    : [];

  if (rows.length === 0) return null;

  const keys = Object.keys(rows[0]);

  const sorted = sortKey
    ? [...rows].sort((a, b) => {
        const av = String(a[sortKey] ?? '');
        const bv = String(b[sortKey] ?? '');
        return asc ? av.localeCompare(bv) : bv.localeCompare(av);
      })
    : rows;

  function handleSort(key: string) {
    if (key === sortKey) {
      setAsc(a => !a);
    } else {
      setSortKey(key);
      setAsc(true);
    }
  }

  function renderCell(key: string, val: unknown) {
    if (key === 'status' && typeof val === 'string') {
      const cls = STATUS_BADGE[val] ?? 'bg-slate-700 text-slate-300';
      return (
        <span className={`px-2 py-0.5 rounded text-xs font-semibold ${cls}`}>
          {val}
        </span>
      );
    }
    if (val === null || val === undefined) return <span className="text-slate-500">—</span>;
    return <span>{String(val)}</span>;
  }

  return (
    <div className="overflow-x-auto mt-2 rounded-lg border border-slate-700">
      <table className="text-xs w-full border-collapse">
        <thead>
          <tr className="bg-slate-800 text-slate-400">
            {keys.map(k => (
              <th
                key={k}
                className="px-3 py-2 text-left cursor-pointer select-none hover:text-slate-200 whitespace-nowrap"
                onClick={() => handleSort(k)}
              >
                {k}
                {sortKey === k && (
                  <span className="ml-1">{asc ? '↑' : '↓'}</span>
                )}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => (
            <tr
              key={i}
              className="border-t border-slate-700 hover:bg-slate-800/60 transition-colors"
            >
              {keys.map(k => (
                <td key={k} className="px-3 py-2 text-slate-200 whitespace-nowrap">
                  {renderCell(k, row[k])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="px-3 py-1.5 text-xs text-slate-500 border-t border-slate-700">
        {rows.length} row{rows.length !== 1 ? 's' : ''}
      </div>
    </div>
  );
}
