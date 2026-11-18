// =============================================================================
// ConversationSidebar.tsx — ChatGPT-style conversation sidebar (Phase 1)
// Shows conversation list, search, rename, archive, delete, new chat
// =============================================================================

import { useState, useEffect, useRef } from 'react';
import type { ConversationSummary } from '../types';
import {
  listConversations,
  searchConversations,
  renameConversation,
  archiveConversation,
  deleteConversation,
} from '../api/client';

interface Props {
  activeConversationId: string | null;
  onSelectConversation: (id: string) => void;
  onNewConversation: () => void;
  refreshTrigger: number; // increment to force a refresh
  isDark: boolean;
}

export function ConversationSidebar({
  activeConversationId,
  onSelectConversation,
  onNewConversation,
  refreshTrigger,
  isDark,
}: Props) {
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const renameInputRef = useRef<HTMLInputElement>(null);

  async function loadConversations() {
    setLoading(true);
    try {
      if (searchQuery.trim()) {
        const res = await searchConversations(searchQuery.trim());
        setConversations(res.conversations);
      } else {
        const res = await listConversations();
        setConversations(res.conversations);
      }
    } catch (e) {
      console.error('Failed to load conversations:', e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { loadConversations(); }, [refreshTrigger]);
  useEffect(() => {
    const timer = setTimeout(loadConversations, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  useEffect(() => {
    if (renamingId && renameInputRef.current) {
      renameInputRef.current.focus();
      renameInputRef.current.select();
    }
  }, [renamingId]);

  async function handleRename(id: string) {
    if (!renameValue.trim()) return;
    try {
      await renameConversation(id, renameValue.trim());
      setRenamingId(null);
      await loadConversations();
    } catch (e) {
      console.error('Rename failed:', e);
    }
  }

  async function handleArchive(id: string) {
    try {
      await archiveConversation(id);
      setMenuOpenId(null);
      await loadConversations();
    } catch (e) {
      console.error('Archive failed:', e);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this conversation? This cannot be undone.')) return;
    try {
      await deleteConversation(id);
      setMenuOpenId(null);
      if (activeConversationId === id) onNewConversation();
      await loadConversations();
    } catch (e) {
      console.error('Delete failed:', e);
    }
  }

  const surface = isDark ? '#160d2e' : '#ffffff';
  const border = isDark ? '#2d1b6e' : '#ede9fe';
  const textMain = isDark ? '#ede9fe' : '#1e1040';
  const textMuted = isDark ? '#9b89c4' : '#7c6fa0';
  const activeBg = isDark ? '#2d1b6e' : '#f3eeff';
  const hoverBg = isDark ? '#1f1245' : '#f8f5ff';
  const inputBg = isDark ? '#1f1245' : '#f3eeff';

  function formatDate(iso: string | null): string {
    if (!iso) return '';
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / 86400000);
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays}d ago`;
    return d.toLocaleDateString();
  }

  return (
    <aside
      style={{
        width: 260,
        minWidth: 260,
        maxWidth: 260,
        height: '100%',
        background: surface,
        borderRight: `1px solid ${border}`,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        flexShrink: 0,
      }}
    >
      {/* Header */}
      <div style={{ padding: '12px 12px 8px', borderBottom: `1px solid ${border}` }}>
        <button
          onClick={onNewConversation}
          style={{
            width: '100%',
            background: 'linear-gradient(135deg, #7c3aed, #db2777)',
            color: '#fff',
            border: 'none',
            borderRadius: 10,
            padding: '8px 12px',
            fontWeight: 700,
            fontSize: '0.85rem',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 8,
          }}
        >
          <span style={{ fontSize: '1rem' }}>✏️</span>
          New Chat
        </button>
      </div>

      {/* Search */}
      <div style={{ padding: '8px 12px', borderBottom: `1px solid ${border}` }}>
        <input
          type="text"
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          placeholder="Search conversations…"
          style={{
            width: '100%',
            background: inputBg,
            border: `1px solid ${border}`,
            borderRadius: 8,
            padding: '6px 10px',
            fontSize: '0.78rem',
            color: textMain,
            outline: 'none',
            boxSizing: 'border-box',
          }}
        />
      </div>

      {/* Conversation list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px 0' }}>
        {loading && conversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: 16 }}>
            Loading…
          </div>
        )}
        {!loading && conversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: 16 }}>
            No conversations yet.
            <br />Start a new chat!
          </div>
        )}
        {conversations.map(conv => (
          <div
            key={conv.conversation_id}
            style={{
              position: 'relative',
              background: activeConversationId === conv.conversation_id ? activeBg : 'transparent',
              borderRadius: 8,
              margin: '2px 8px',
            }}
          >
            {renamingId === conv.conversation_id ? (
              <div style={{ padding: '6px 8px' }}>
                <input
                  ref={renameInputRef}
                  value={renameValue}
                  onChange={e => setRenameValue(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') handleRename(conv.conversation_id);
                    if (e.key === 'Escape') setRenamingId(null);
                  }}
                  onBlur={() => setRenamingId(null)}
                  style={{
                    width: '100%',
                    background: inputBg,
                    border: `1.5px solid #7c3aed`,
                    borderRadius: 6,
                    padding: '4px 8px',
                    fontSize: '0.8rem',
                    color: textMain,
                    outline: 'none',
                    boxSizing: 'border-box',
                  }}
                />
              </div>
            ) : (
              <div
                onClick={() => onSelectConversation(conv.conversation_id)}
                style={{
                  padding: '8px 10px',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: 6,
                }}
                onMouseEnter={e => {
                  if (activeConversationId !== conv.conversation_id)
                    (e.currentTarget.parentElement as HTMLElement).style.background = hoverBg;
                }}
                onMouseLeave={e => {
                  if (activeConversationId !== conv.conversation_id)
                    (e.currentTarget.parentElement as HTMLElement).style.background = 'transparent';
                }}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{
                    fontSize: '0.82rem',
                    fontWeight: activeConversationId === conv.conversation_id ? 700 : 500,
                    color: textMain,
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                  }}>
                    {conv.title}
                  </div>
                  <div style={{ fontSize: '0.68rem', color: textMuted, marginTop: 2 }}>
                    {formatDate(conv.last_message_at || conv.updated_at)}
                  </div>
                </div>
                {/* Context menu button */}
                <button
                  onClick={e => {
                    e.stopPropagation();
                    setMenuOpenId(menuOpenId === conv.conversation_id ? null : conv.conversation_id);
                  }}
                  style={{
                    background: 'none',
                    border: 'none',
                    color: textMuted,
                    cursor: 'pointer',
                    padding: '0 2px',
                    fontSize: '0.9rem',
                    lineHeight: 1,
                    flexShrink: 0,
                  }}
                  title="More options"
                >
                  ⋯
                </button>
              </div>
            )}

            {/* Dropdown menu */}
            {menuOpenId === conv.conversation_id && (
              <div
                style={{
                  position: 'absolute',
                  right: 0,
                  top: '100%',
                  zIndex: 100,
                  background: surface,
                  border: `1px solid ${border}`,
                  borderRadius: 8,
                  boxShadow: '0 4px 16px rgba(0,0,0,0.15)',
                  minWidth: 140,
                  overflow: 'hidden',
                }}
              >
                {[
                  {
                    label: '✏️ Rename', action: () => {
                      setRenameValue(conv.title);
                      setRenamingId(conv.conversation_id);
                      setMenuOpenId(null);
                    }
                  },
                  { label: '📦 Archive', action: () => handleArchive(conv.conversation_id) },
                  { label: '🗑️ Delete', action: () => handleDelete(conv.conversation_id), danger: true },
                ].map(item => (
                  <button
                    key={item.label}
                    onClick={item.action}
                    style={{
                      display: 'block',
                      width: '100%',
                      textAlign: 'left',
                      background: 'none',
                      border: 'none',
                      padding: '8px 14px',
                      fontSize: '0.8rem',
                      color: (item as any).danger ? '#ef4444' : textMain,
                      cursor: 'pointer',
                    }}
                    onMouseEnter={e => (e.currentTarget.style.background = hoverBg)}
                    onMouseLeave={e => (e.currentTarget.style.background = 'none')}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </aside>
  );
}
