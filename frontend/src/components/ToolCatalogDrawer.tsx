// =============================================================================
// components/ToolCatalogDrawer.tsx — Sidebar showing discovered MCP tools
// =============================================================================

import { useEffect, useState } from 'react';
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
        className="fixed inset-0 z-30 bg-black/40"
        onClick={onClose}
      />
      {/* Drawer */}
      <aside className="fixed right-0 top-0 z-40 h-full w-80 bg-slate-900 border-l border-slate-700
                        flex flex-col shadow-2xl overflow-y-auto">
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700 sticky top-0 bg-slate-900">
          <h2 className="text-sm font-semibold text-slate-100">🔧 MCP Tool Catalogue</h2>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-100 text-lg leading-none"
          >
            ✕
          </button>
        </div>

        <div className="flex-1 p-3 space-y-2">
          {loading && (
            <p className="text-slate-400 text-xs text-center py-8">Loading tools…</p>
          )}
          {error && (
            <p className="text-red-400 text-xs">{error}</p>
          )}
          {tools.map(tool => (
            <div key={tool.name} className="rounded-lg border border-slate-700 overflow-hidden">
              <button
                className="w-full flex justify-between items-center px-3 py-2 text-left
                           text-xs hover:bg-slate-800 transition-colors"
                onClick={() => setExpanded(e => e === tool.name ? null : tool.name)}
              >
                <span className="font-mono text-sky-400 font-medium">{tool.name}</span>
                <span className="text-slate-500">{expanded === tool.name ? '▲' : '▼'}</span>
              </button>

              {expanded === tool.name && (
                <div className="px-3 pb-3 bg-slate-800/50 text-xs space-y-2">
                  {tool.description && (
                    <p className="text-slate-300 mt-1">{tool.description}</p>
                  )}
                  {tool.inputSchema?.properties && (
                    <div>
                      <p className="text-slate-500 font-medium mb-1">Parameters:</p>
                      {Object.entries(tool.inputSchema.properties).map(([pname, pschema]) => (
                        <div key={pname} className="flex gap-1 text-slate-400">
                          <span className="text-sky-500 font-mono">{pname}</span>
                          <span className="text-slate-600">({pschema.type ?? 'any'})</span>
                          {tool.inputSchema?.required?.includes(pname) && (
                            <span className="text-red-400">*</span>
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
