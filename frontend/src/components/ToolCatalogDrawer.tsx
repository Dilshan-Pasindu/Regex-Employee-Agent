// =============================================================================
// components/ToolCatalogDrawer.tsx — MCP tool sidebar (redesign)
// =============================================================================

import { useEffect, useState } from 'react';
import { Wrench, X } from 'lucide-react';
import { fetchTools } from '../api/client';
import type { McpTool } from '../types';

interface Props {
  open: boolean;
  onClose: () => void;
}

export function ToolCatalogDrawer({ open, onClose }: Props) {
  const [tools, setTools] = useState<McpTool[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<string | null>(null);

  useEffect(() => {
    if (!open || tools.length > 0) return;
    setLoading(true);
    fetchTools()
      .then(d => setTools(d.tools))
      .catch(e => setError(e.message))
      .finally(() => setLoading(false));
  }, [open]);

  if (!open) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        style={{ position: 'fixed', inset: 0, zIndex: 30, background: 'rgba(0,0,0,0.4)', backdropFilter: 'blur(4px)' }}
        onClick={onClose}
      />

      {/* Drawer */}
      <aside
        style={{
          position: 'fixed', right: 0, top: 0, zIndex: 40,
          height: '100%', width: 300,
          background: 'var(--surface)',
          borderLeft: '1px solid var(--border)',
          display: 'flex', flexDirection: 'column',
          boxShadow: '-8px 0 32px rgba(0,0,0,0.15)',
          overflowY: 'auto',
        }}
        role="complementary"
        aria-label="MCP Tool Catalogue"
      >
        {/* Header */}
        <div style={{
          padding: '14px 16px',
          borderBottom: '1px solid var(--border)',
          position: 'sticky', top: 0,
          background: 'var(--surface)',
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{
              width: 30, height: 30, borderRadius: 8,
              background: 'linear-gradient(135deg,#7c3aed,#db2777)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <Wrench size={16} color="#fff" />
            </span>
            <h2 style={{ margin: 0, fontSize: '0.9rem', fontWeight: 800, color: 'var(--text)' }}>
              MCP Tool Catalogue
            </h2>
          </div>
          <button
            onClick={onClose}
            aria-label="Close drawer"
            style={{
              background: 'none', border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: '4px', borderRadius: 6,
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              transition: 'color 0.15s',
            }}
            onMouseEnter={e => (e.currentTarget.style.color = 'var(--text)')}
            onMouseLeave={e => (e.currentTarget.style.color = 'var(--text-muted)')}
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: 8 }}>
          {loading && (
            <div style={{ textAlign: 'center', padding: '32px 0', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
              <div className="dot-bounce" style={{ justifyContent: 'center', display: 'flex', gap: 4 }}>
                <span /><span /><span />
              </div>
              <p style={{ marginTop: 8 }}>Loading tools…</p>
            </div>
          )}
          {error && (
            <p style={{ color: '#ef4444', fontSize: '0.78rem', padding: '8px 4px' }}>
              ⚠ {error}
            </p>
          )}

          {tools.map(tool => (
            <div
              key={tool.name}
              style={{
                borderRadius: 12,
                border: '1px solid var(--border)',
                overflow: 'hidden',
                transition: 'border-color 0.15s',
              }}
              onMouseEnter={e => ((e.currentTarget as HTMLDivElement).style.borderColor = '#7c3aed')}
              onMouseLeave={e => ((e.currentTarget as HTMLDivElement).style.borderColor = 'var(--border)')}
            >
              {/* Tool header button */}
              <button
                style={{
                  width: '100%',
                  display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                  padding: '10px 12px',
                  background: 'none', border: 'none',
                  cursor: 'pointer',
                  textAlign: 'left',
                  transition: 'background 0.12s',
                }}
                onMouseEnter={e => (e.currentTarget.style.background = 'var(--surface-2)')}
                onMouseLeave={e => (e.currentTarget.style.background = 'transparent')}
                onClick={() => setExpanded(e => e === tool.name ? null : tool.name)}
                aria-expanded={expanded === tool.name}
              >
                {/* Left: coloured bar + name */}
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <div style={{
                    width: 3, height: 20, borderRadius: 2,
                    background: 'linear-gradient(180deg,#7c3aed,#ec4899)',
                    flexShrink: 0,
                  }} />
                  <span style={{
                    fontFamily: 'monospace', fontSize: '0.78rem',
                    fontWeight: 700, color: '#7c3aed',
                  }}>
                    {tool.name}
                  </span>
                </div>
                <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem', transition: 'transform 0.15s',
                  transform: expanded === tool.name ? 'rotate(180deg)' : 'rotate(0deg)',
                  display: 'inline-block',
                }}>▼</span>
              </button>

              {/* Expanded content */}
              {expanded === tool.name && (
                <div style={{
                  padding: '10px 12px 12px',
                  background: 'var(--surface-2)',
                  borderTop: '1px solid var(--border)',
                  fontSize: '0.76rem',
                  display: 'flex', flexDirection: 'column', gap: 6,
                }}>
                  {tool.description && (
                    <p style={{ margin: 0, color: 'var(--text)', lineHeight: 1.5 }}>
                      {tool.description}
                    </p>
                  )}
                  {tool.inputSchema?.properties && (
                    <div>
                      <p style={{ margin: '0 0 4px', color: 'var(--text-muted)', fontWeight: 700, textTransform: 'uppercase', fontSize: '0.65rem', letterSpacing: 1 }}>
                        Parameters
                      </p>
                      {Object.entries(tool.inputSchema.properties).map(([pname, pschema]) => (
                        <div key={pname} style={{ display: 'flex', gap: 4, alignItems: 'baseline', marginBottom: 2 }}>
                          <span style={{ color: '#7c3aed', fontFamily: 'monospace', fontWeight: 700 }}>{pname}</span>
                          <span style={{ color: 'var(--text-muted)' }}>({pschema.type ?? 'any'})</span>
                          {tool.inputSchema?.required?.includes(pname) && (
                            <span style={{ color: '#ef4444', fontWeight: 700 }}>*</span>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </aside>
    </>
  );
}
