// =============================================================================
// ConversationSidebar.tsx — ChatGPT-style conversation sidebar (Phase 1)
// Shows conversation list, search, rename, archive, delete, new chat
// =============================================================================

import { useState, useEffect, useRef } from 'react';
import { 
  SquarePen, 
  Search, 
  MessageSquare, 
  MoreVertical, 
  Pencil, 
  Archive, 
  Trash2, 
  MessagesSquare 
} from 'lucide-react';
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
            justifyContent: 'center',
            gap: 8,
            transition: 'opacity 0.2s',
          }}
        >
          <SquarePen size={16} />
          <span>New Chat</span>
        </button>
      </div>

      {/* Search */}
      <div style={{ padding: '8px 12px', borderBottom: `1px solid ${border}` }}>
        <div style={{ position: 'relative', display: 'flex', alignItems: 'center' }}>
          <Search size={14} style={{ position: 'absolute', left: 10, color: textMuted, pointerEvents: 'none' }} />
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
              padding: '6px 10px 6px 30px',
              fontSize: '0.78rem',
              color: textMain,
              outline: 'none',
              boxSizing: 'border-box',
            }}
          />
        </div>
      </div>

      {/* Conversation list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px 0' }}>
        {loading && conversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: 16 }}>
            Loading…
          </div>
        )}
        {!loading && conversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: '24px 16px', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8 }}>
            <MessagesSquare size={26} style={{ color: textMuted, opacity: 0.6 }} />
            <span>No conversations yet.<br />Start a new chat!</span>
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
                  gap: 8,
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
                <MessageSquare 
                  size={14} 
                  style={{ 
                    color: activeConversationId === conv.conversation_id ? '#7c3aed' : textMuted, 
                    marginTop: 3, 
                    flexShrink: 0 
                  }} 
                />
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
                    padding: '2px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    borderRadius: 4,
                    flexShrink: 0,
                  }}
                  title="More options"
                >
                  <MoreVertical size={14} />
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
                  minWidth: 130,
                  overflow: 'hidden',
                  padding: '4px 0',
                }}
              >
                <button
                  onClick={() => {
                    setRenameValue(conv.title);
                    setRenamingId(conv.conversation_id);
                    setMenuOpenId(null);
                  }}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    width: '100%',
                    textAlign: 'left',
                    background: 'none',
                    border: 'none',
                    padding: '7px 12px',
                    fontSize: '0.78rem',
                    color: textMain,
                    cursor: 'pointer',
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = hoverBg)}
                  onMouseLeave={e => (e.currentTarget.style.background = 'none')}
                >
                  <Pencil size={13} style={{ color: textMuted }} />
                  <span>Rename</span>
                </button>
                <button
                  onClick={() => handleArchive(conv.conversation_id)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    width: '100%',
                    textAlign: 'left',
                    background: 'none',
                    border: 'none',
                    padding: '7px 12px',
                    fontSize: '0.78rem',
                    color: textMain,
                    cursor: 'pointer',
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = hoverBg)}
                  onMouseLeave={e => (e.currentTarget.style.background = 'none')}
                >
                  <Archive size={13} style={{ color: textMuted }} />
                  <span>Archive</span>
                </button>
                <button
                  onClick={() => handleDelete(conv.conversation_id)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    width: '100%',
                    textAlign: 'left',
                    background: 'none',
                    border: 'none',
                    padding: '7px 12px',
                    fontSize: '0.78rem',
                    color: '#ef4444',
                    cursor: 'pointer',
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = hoverBg)}
                  onMouseLeave={e => (e.currentTarget.style.background = 'none')}
                >
                  <Trash2 size={13} style={{ color: '#ef4444' }} />
                  <span>Delete</span>
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
    </aside>
  );
}
