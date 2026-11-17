// =============================================================================
// components/EmployeeTable.tsx — Sortable table with status badges (redesign)
// =============================================================================

import { useState } from 'react';

type Row = Record<string, unknown>;

function statusStyle(val: string): React.CSSProperties {
  switch (val) {
    case 'Active':    return { background: 'rgba(16,185,129,0.15)', color: '#059669', border: '1px solid rgba(16,185,129,0.3)' };
    case 'Inactive':  return { background: 'rgba(239,68,68,0.12)',  color: '#dc2626', border: '1px solid rgba(239,68,68,0.25)' };
    case 'On Leave':  return { background: 'rgba(245,158,11,0.15)', color: '#d97706', border: '1px solid rgba(245,158,11,0.3)' };
    default:          return { background: 'rgba(124,58,237,0.1)',  color: '#7c3aed', border: '1px solid rgba(124,58,237,0.2)' };
  }
}

interface Props {
  data: unknown;
}

export function EmployeeTable({ data }: Props) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [asc, setAsc] = useState(true);
  const [copied, setCopied] = useState(false);

  const rows: Row[] = Array.isArray(data)
    ? (data as Row[])
    : data && typeof data === 'object'
    ? [data as Row]
    : [];

  if (rows.length === 0) return null;

  const keys = Object.keys(rows[0]);

  function copyAsTsv() {
    const header = keys.join('\t');
    const body = rows.map(r => keys.map(k => String(r[k] ?? '')).join('\t')).join('\n');
    navigator.clipboard.writeText(`${header}\n${body}`).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  function downloadCsv() {
    const escapeCsv = (val: unknown) => {
      const s = String(val ?? '').replace(/"/g, '""');
      return `"${s}"`;
    };
    const header = keys.map(escapeCsv).join(',');
    const body = rows.map(r => keys.map(k => escapeCsv(r[k])).join(',')).join('\n');
    const blob = new Blob([`${header}\n${body}`], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `employees_export_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  const sorted = sortKey
    ? [...rows].sort((a, b) => {
        const av = String(a[sortKey] ?? '');
        const bv = String(b[sortKey] ?? '');
        return asc ? av.localeCompare(bv) : bv.localeCompare(av);
      })
    : rows;

  function handleSort(key: string) {
    if (key === sortKey) setAsc(a => !a);
    else { setSortKey(key); setAsc(true); }
  }

  function renderCell(key: string, val: unknown) {
    if (key === 'status' && typeof val === 'string') {
      return (
        <span style={{
          display: 'inline-block',
          fontSize: '0.68rem',
          fontWeight: 700,
          padding: '2px 8px',
          borderRadius: 99,
          ...statusStyle(val),
        }}>
          {val}
        </span>
      );
    }
    if (val === null || val === undefined) {
      return <span style={{ color: 'var(--text-muted)' }}>—</span>;
    }
    return <span>{String(val)}</span>;
  }

  return (
    <div style={{
      marginTop: 10,
      borderRadius: 12,
      border: '1px solid var(--border)',
      boxShadow: '0 2px 10px rgba(0,0,0,0.05)',
      overflow: 'hidden',
    }}>
      {/* Action Bar for Export (CSV / TSV) */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '6px 12px',
        background: 'var(--surface-2)',
        borderBottom: '1px solid var(--border)',
        fontSize: '0.72rem',
      }}>
        <span style={{ color: 'var(--text-muted)', fontWeight: 600 }}>
          {rows.length} record{rows.length !== 1 ? 's' : ''}
        </span>
        <div style={{ display: 'flex', gap: 6 }}>
          <button
            onClick={copyAsTsv}
            title="Copy as TSV (paste into Excel/Sheets)"
            style={{
              background: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 6,
              padding: '3px 8px',
              fontSize: '0.7rem',
              color: 'var(--text)',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 4,
            }}
          >
            📋 {copied ? 'Copied TSV!' : 'Copy TSV'}
          </button>
          <button
            onClick={downloadCsv}
            title="Download CSV spreadsheet"
            style={{
              background: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 6,
              padding: '3px 8px',
              fontSize: '0.7rem',
              color: 'var(--text)',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 4,
            }}
          >
            📥 Export CSV
          </button>
        </div>
      </div>
      <div style={{ overflowX: 'auto' }}>
      <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.78rem' }}>
        <thead>
          <tr style={{
            background: 'linear-gradient(90deg, rgba(124,58,237,0.12), rgba(219,39,119,0.08))',
          }}>
            {keys.map(k => (
              <th
                key={k}
                onClick={() => handleSort(k)}
                style={{
                  padding: '8px 12px',
                  textAlign: 'left',
                  cursor: 'pointer',
                  whiteSpace: 'nowrap',
                  fontWeight: 700,
                  color: sortKey === k ? '#7c3aed' : 'var(--text-muted)',
                  userSelect: 'none',
                  transition: 'color 0.15s',
                }}
              >
                {k}
                {sortKey === k && (
                  <span style={{ marginLeft: 4 }}>{asc ? '↑' : '↓'}</span>
                )}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => (
            <tr
              key={i}
              style={{
                borderTop: '1px solid var(--border)',
                transition: 'background 0.12s',
              }}
              onMouseEnter={e => (e.currentTarget.style.background = 'rgba(124,58,237,0.04)')}
              onMouseLeave={e => (e.currentTarget.style.background = 'transparent')}
            >
              {keys.map(k => (
                <td key={k} style={{ padding: '7px 12px', whiteSpace: 'nowrap', color: 'var(--text)' }}>
                  {renderCell(k, row[k])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      </div>
      <div style={{
        padding: '5px 12px',
        fontSize: '0.68rem',
        color: 'var(--text-muted)',
        borderTop: '1px solid var(--border)',
      }}>
        {rows.length} row{rows.length !== 1 ? 's' : ''}
      </div>
    </div>
  );
}
