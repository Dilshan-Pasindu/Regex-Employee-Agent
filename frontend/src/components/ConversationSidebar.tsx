// =============================================================================
// ConversationSidebar.tsx — ChatGPT-style conversation sidebar
// Shows conversation list, pin/unpin (max 3), search, rename, delete, new chat
// =============================================================================

import { useState, useEffect, useRef } from 'react';
import { 
  SquarePen, 
  Search, 
  MessageSquare, 
  MoreVertical, 
  Pencil, 
  Pin,
  PinOff,
  Trash2, 
  MessagesSquare 
} from 'lucide-react';
import type { ConversationSummary } from '../types';
import {
  listConversations,
  searchConversations,
  renameConversation,
  pinConversation,
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
  const [isPinning, setIsPinning] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  const renameInputRef = useRef<HTMLInputElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

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

  // Click outside to close context menu
  useEffect(() => {
    if (!menuOpenId) return;
    function handleDocumentClick(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpenId(null);
      }
    }
    document.addEventListener('mousedown', handleDocumentClick);
    return () => document.removeEventListener('mousedown', handleDocumentClick);
  }, [menuOpenId]);

  // Toast timer
  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 3500);
    return () => clearTimeout(timer);
  }, [toast]);

  async function handlePin(id: string, currentlyPinned: boolean) {
    if (isPinning) return;

    // Check max 3 pins on frontend
    const currentPinnedCount = conversations.filter(c => c.is_pinned && c.conversation_id !== id).length;
    if (!currentlyPinned && currentPinnedCount >= 3) {
      setToast('You can pin up to 3 chats. Unpin one to pin another.');
      setMenuOpenId(null);
      return;
    }

    setIsPinning(true);
    const targetPinned = !currentlyPinned;
    const nowIso = new Date().toISOString();

    // Optimistic update
    setConversations(prev =>
      prev.map(c =>
        c.conversation_id === id
          ? { ...c, is_pinned: targetPinned, pinned_at: targetPinned ? nowIso : null }
          : c
      )
    );
    setMenuOpenId(null);

    try {
      const updated = await pinConversation(id, targetPinned);
      setConversations(prev =>
        prev.map(c => (c.conversation_id === id ? { ...c, ...updated } : c))
      );
    } catch (e: any) {
      // Rollback
      setConversations(prev =>
        prev.map(c =>
          c.conversation_id === id
            ? { ...c, is_pinned: currentlyPinned, pinned_at: currentlyPinned ? c.pinned_at : null }
            : c
        )
      );
      if (e.message?.includes('max_pins') || e.message?.includes('400')) {
        setToast('You can pin up to 3 chats. Unpin one to pin another.');
      } else {
        setToast('Failed to update pin. Please try again.');
      }
    } finally {
      setIsPinning(false);
    }
  }

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

  // Sorted conversations: pinned first (by pinned_at DESC), unpinned next (by last_message_at/updated_at DESC)
  const sortedConversations = [...conversations].sort((a, b) => {
    if (a.is_pinned && !b.is_pinned) return -1;
    if (!a.is_pinned && b.is_pinned) return 1;
    if (a.is_pinned && b.is_pinned) {
      const timeA = a.pinned_at ? new Date(a.pinned_at).getTime() : 0;
      const timeB = b.pinned_at ? new Date(b.pinned_at).getTime() : 0;
      return timeB - timeA;
    }
    const timeA = new Date(a.last_message_at || a.updated_at || a.created_at).getTime();
    const timeB = new Date(b.last_message_at || b.updated_at || b.created_at).getTime();
    return timeB - timeA;
  });

  const hasPinned = sortedConversations.some(c => c.is_pinned);
  const firstUnpinnedIndex = sortedConversations.findIndex(c => !c.is_pinned);

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
        position: 'relative',
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
        {loading && sortedConversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: 16 }}>
            Loading…
          </div>
        )}
        {!loading && sortedConversations.length === 0 && (
          <div style={{ textAlign: 'center', color: textMuted, fontSize: '0.75rem', padding: '24px 16px', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8 }}>
            <MessagesSquare size={26} style={{ color: textMuted, opacity: 0.6 }} />
            <span>No conversations yet.<br />Start a new chat!</span>
          </div>
        )}

        {sortedConversations.map((conv, idx) => {
          const isFirstPinned = hasPinned && conv.is_pinned && idx === 0;
          const isFirstUnpinned = hasPinned && !conv.is_pinned && idx === firstUnpinnedIndex;

          return (
            <div key={conv.conversation_id}>
              {/* Pinned divider */}
              {isFirstPinned && (
                <div style={{
                  padding: '8px 14px 4px',
                  fontSize: '0.65rem',
                  color: textMuted,
                  fontWeight: 700,
                  letterSpacing: '0.06em',
                  textTransform: 'uppercase',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                }}>
                  <Pin size={10} style={{ color: '#a855f7' }} />
                  <span>Pinned</span>
                </div>
              )}

              {/* All / Unpinned Chats divider */}
              {isFirstUnpinned && (
                <div style={{
                  padding: '10px 14px 4px',
                  fontSize: '0.65rem',
                  color: textMuted,
                  fontWeight: 700,
                  letterSpacing: '0.06em',
                  textTransform: 'uppercase',
                  borderTop: `1px solid ${border}`,
                  marginTop: 6,
                }}>
                  Conversations
                </div>
              )}

              <div
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
                        display: 'flex',
                        alignItems: 'center',
                        gap: 5,
                        minWidth: 0,
                      }}>
                        {conv.is_pinned && (
                          <span title="Pinned" style={{ display: 'inline-flex', alignItems: 'center' }}>
                            <Pin
                              size={14}
                              style={{
                                color: '#a855f7',
                                fill: '#a855f7',
                                flexShrink: 0,
                              }}
                            />
                          </span>
                        )}
                        <span style={{
                          whiteSpace: 'nowrap',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          minWidth: 0,
                          flex: 1,
                        }}>
                          {conv.title}
                        </span>
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
                    ref={menuRef}
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
                    {/* Pin / Unpin */}
                    <button
                      onClick={() => handlePin(conv.conversation_id, conv.is_pinned)}
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
                      {conv.is_pinned ? (
                        <PinOff size={13} style={{ color: '#a855f7' }} />
                      ) : (
                        <Pin size={13} style={{ color: '#a855f7' }} />
                      )}
                      <span>{conv.is_pinned ? 'Unpin' : 'Pin'}</span>
                    </button>

                    {/* Rename */}
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

                    {/* Delete */}
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
            </div>
          );
        })}
      </div>

      {/* Toast Alert */}
      {toast && (
        <div
          style={{
            position: 'fixed',
            bottom: 24,
            left: '50%',
            transform: 'translateX(-50%)',
            background: isDark ? 'rgba(30, 16, 64, 0.95)' : 'rgba(255, 255, 255, 0.95)',
            color: textMain,
            border: '1px solid #7c3aed',
            borderRadius: 10,
            padding: '10px 18px',
            fontSize: '0.82rem',
            fontWeight: 600,
            boxShadow: '0 8px 30px rgba(124, 58, 237, 0.25)',
            zIndex: 9999,
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            pointerEvents: 'none',
            backdropFilter: 'blur(8px)',
          }}
        >
          <Pin size={14} style={{ color: '#a855f7' }} />
          <span>{toast}</span>
        </div>
      )}
    </aside>
  );
}
