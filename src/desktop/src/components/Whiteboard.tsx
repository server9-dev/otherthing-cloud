/**
 * Collaborative Whiteboard Component
 *
 * An interactive whiteboard that supports:
 * - Real-time collaboration via WebSocket
 * - Multiple users drawing simultaneously
 * - Live cursor/pointer tracking
 * - Persistent storage via workspace API
 * - Pop-out to full-screen mode
 * - Resizable height
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { Excalidraw, exportToBlob } from '@excalidraw/excalidraw';
import '@excalidraw/excalidraw/index.css';
import type { ExcalidrawElement } from '@excalidraw/excalidraw/element/types';
import type { AppState, BinaryFiles, Collaborator } from '@excalidraw/excalidraw/types';
import {
  ExternalLink, Save, Download, Trash2,
  ChevronUp, ChevronDown, X, Users, Loader2
} from 'lucide-react';
import { CyberButton } from './CyberButton';
import { authFetch } from '../App';

// Types for whiteboard data
interface WhiteboardData {
  id: string;
  name: string;
  elements: readonly ExcalidrawElement[];
  appState?: Partial<AppState>;
  files?: BinaryFiles;
  version: number;
  updatedAt: string;
}

interface CollabMessage {
  type: string;
  [key: string]: any;
}

interface RemotePointer {
  odilo: string;
  username: string;
  pointer: { x: number; y: number } | null;
}

interface WhiteboardProps {
  workspaceId: string;
  onClose?: () => void;
  minHeight?: number;
  maxHeight?: number;
  defaultHeight?: number;
}

// WebSocket URL - derive from current location
const getCollabWsUrl = () => {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  // Use orchestrator URL
  return `${protocol}//localhost:8080/ws/collab`;
};

// Get token from localStorage
const getToken = () => localStorage.getItem('rhizos_token') || '';

export const Whiteboard = ({
  workspaceId,
  onClose,
  minHeight = 400,
  maxHeight = 1200,
  defaultHeight = 600,
}: WhiteboardProps) => {
  // User info (fetched on mount)
  const [userInfo, setUserInfo] = useState<{ userId: string; username: string } | null>(null);
  const [height, setHeight] = useState(defaultHeight);
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isPoppedOut, setIsPoppedOut] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Whiteboard data
  const [whiteboard, setWhiteboard] = useState<WhiteboardData | null>(null);
  const [localElements, setLocalElements] = useState<readonly ExcalidrawElement[]>([]);

  // Collaboration state
  const [collaborators, setCollaborators] = useState<Array<{ odilo: string; username: string }>>([]);
  const [remotePointers, setRemotePointers] = useState<Map<string, RemotePointer>>(new Map());
  const [isConnected, setIsConnected] = useState(false);

  const excalidrawRef = useRef<any>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const isResizing = useRef(false);
  const lastBroadcast = useRef<number>(0);

  // Load whiteboard from API
  const loadWhiteboard = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const res = await authFetch(`/api/v1/workspaces/${workspaceId}/whiteboards/default`);
      if (!res.ok) {
        throw new Error('Failed to load whiteboard');
      }
      const data = await res.json();
      setWhiteboard(data.whiteboard);
      setLocalElements(data.whiteboard.elements || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load whiteboard');
    } finally {
      setIsLoading(false);
    }
  }, [workspaceId]);

  // Save whiteboard to API
  const saveWhiteboard = useCallback(async () => {
    if (!whiteboard || isSaving) return;

    setIsSaving(true);
    try {
      const res = await authFetch(`/api/v1/workspaces/${workspaceId}/whiteboards/${whiteboard.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          elements: localElements,
          expectedVersion: whiteboard.version,
        }),
      });

      if (!res.ok) {
        const data = await res.json();
        if (data.conflict) {
          // Handle conflict - for now just reload
          console.warn('[Whiteboard] Version conflict, reloading...');
          loadWhiteboard();
          return;
        }
        throw new Error(data.error || 'Failed to save');
      }

      const data = await res.json();
      setWhiteboard(data.whiteboard);
      setIsDirty(false);
    } catch (err) {
      console.error('Failed to save whiteboard:', err);
    } finally {
      setIsSaving(false);
    }
  }, [whiteboard, localElements, workspaceId, isSaving, loadWhiteboard]);

  // Connect to collaboration WebSocket
  const connectCollab = useCallback(() => {
    if (!whiteboard || !userInfo || wsRef.current) return;

    const ws = new WebSocket(getCollabWsUrl());

    ws.onopen = () => {
      console.log('[Whiteboard] Connected to collaboration server');
      setIsConnected(true);

      // Join the whiteboard room
      ws.send(JSON.stringify({
        type: 'join',
        workspaceId,
        whiteboardId: whiteboard.id,
        userId: userInfo.userId,
        username: userInfo.username,
        token: getToken(),
      }));
    };

    ws.onmessage = (event) => {
      try {
        const msg: CollabMessage = JSON.parse(event.data);

        switch (msg.type) {
          case 'joined':
            setCollaborators(msg.collaborators || []);
            break;

          case 'user_joined':
            setCollaborators(prev => [...prev, { odilo: msg.userId, username: msg.username }]);
            break;

          case 'user_left':
            setCollaborators(prev => prev.filter(c => c.odilo !== msg.userId));
            setRemotePointers(prev => {
              const next = new Map(prev);
              next.delete(msg.userId);
              return next;
            });
            break;

          case 'pointer_update':
            setRemotePointers(prev => {
              const next = new Map(prev);
              if (msg.pointer) {
                next.set(msg.userId, {
                  odilo: msg.userId,
                  username: msg.username,
                  pointer: msg.pointer,
                });
              } else {
                next.delete(msg.userId);
              }
              return next;
            });
            break;

          case 'elements_update':
            // Merge remote elements with local
            if (msg.senderId !== userInfo?.userId) {
              setLocalElements(msg.elements);
              // Update excalidraw if ref is available
              if (excalidrawRef.current) {
                excalidrawRef.current.updateScene({ elements: msg.elements });
              }
            }
            break;

          case 'whiteboard_update':
            // Full whiteboard update from another client's save
            if (msg.senderId !== userInfo?.userId) {
              setWhiteboard(msg.whiteboard);
              setLocalElements(msg.whiteboard.elements || []);
              if (excalidrawRef.current) {
                excalidrawRef.current.updateScene({ elements: msg.whiteboard.elements });
              }
            }
            break;
        }
      } catch (err) {
        console.error('[Whiteboard] Error parsing message:', err);
      }
    };

    ws.onclose = () => {
      console.log('[Whiteboard] Disconnected from collaboration server');
      setIsConnected(false);
      wsRef.current = null;

      // Reconnect after delay
      setTimeout(() => {
        if (whiteboard) {
          connectCollab();
        }
      }, 3000);
    };

    ws.onerror = (err) => {
      console.error('[Whiteboard] WebSocket error:', err);
    };

    wsRef.current = ws;
  }, [whiteboard, workspaceId, userInfo]);

  // Broadcast element changes (throttled)
  const broadcastElements = useCallback((elements: readonly ExcalidrawElement[]) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    // Throttle broadcasts to 100ms
    const now = Date.now();
    if (now - lastBroadcast.current < 100) return;
    lastBroadcast.current = now;

    wsRef.current.send(JSON.stringify({
      type: 'elements_update',
      elements,
    }));
  }, []);

  // Broadcast pointer position
  const broadcastPointer = useCallback((pointer: { x: number; y: number } | null) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    wsRef.current.send(JSON.stringify({
      type: 'pointer',
      pointer,
    }));
  }, []);

  // Handle excalidraw changes
  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[], appState: AppState, files: BinaryFiles) => {
      setLocalElements(elements);
      setIsDirty(true);

      // Broadcast to collaborators
      broadcastElements(elements);
    },
    [broadcastElements]
  );

  // Handle pointer move
  const handlePointerUpdate = useCallback(
    (payload: { pointer: { x: number; y: number }; button: string }) => {
      broadcastPointer(payload.pointer);
    },
    [broadcastPointer]
  );

  // Fetch user info on mount
  useEffect(() => {
    const fetchUserInfo = async () => {
      try {
        const res = await authFetch('/api/v1/auth/me');
        if (res.ok) {
          const data = await res.json();
          setUserInfo({
            userId: data.userId || 'anonymous',
            username: data.username || 'Anonymous',
          });
        }
      } catch {
        setUserInfo({ userId: 'anonymous', username: 'Anonymous' });
      }
    };
    fetchUserInfo();
  }, []);

  // Load whiteboard on mount
  useEffect(() => {
    loadWhiteboard();
  }, [loadWhiteboard]);

  // Connect to collab when whiteboard is loaded and user info available
  useEffect(() => {
    if (whiteboard && userInfo && !wsRef.current) {
      connectCollab();
    }

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [whiteboard, userInfo, connectCollab]);

  // Handle resize
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isResizing.current = true;

    const startY = e.clientY;
    const startHeight = height;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isResizing.current) return;
      const deltaY = moveEvent.clientY - startY;
      const newHeight = Math.max(minHeight, Math.min(maxHeight, startHeight + deltaY));
      setHeight(newHeight);
    };

    const handleMouseUp = () => {
      isResizing.current = false;
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [height, minHeight, maxHeight]);

  // Export as PNG
  const handleExport = async () => {
    if (localElements.length === 0) return;

    try {
      const blob = await exportToBlob({
        elements: localElements as ExcalidrawElement[],
        appState: { theme: 'dark' } as AppState,
        files: {},
        mimeType: 'image/png',
      });

      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${whiteboard?.name || 'whiteboard'}.png`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export:', err);
    }
  };

  // Clear board
  const handleClear = () => {
    if (window.confirm('Clear all drawings? This cannot be undone.')) {
      setLocalElements([]);
      setIsDirty(true);
      if (excalidrawRef.current) {
        excalidrawRef.current.updateScene({ elements: [] });
      }
      broadcastElements([]);
    }
  };

  // Convert remote pointers to Excalidraw collaborators format
  const excalidrawCollaborators = new Map<string, Collaborator>();
  remotePointers.forEach((pointer, odilo) => {
    if (pointer.pointer) {
      excalidrawCollaborators.set(odilo, {
        username: pointer.username,
        pointer: pointer.pointer,
        color: { background: '#6366f1', stroke: '#4f46e5' },
      } as Collaborator);
    }
  });

  // Render loading state
  if (isLoading) {
    return (
      <div className="cyber-card" style={{ marginBottom: 'var(--gap-md)' }}>
        <div className="cyber-card-body" style={{ padding: 'var(--gap-xl)', textAlign: 'center' }}>
          <Loader2 size={32} style={{ color: 'var(--accent)', animation: 'spin 1s linear infinite' }} />
          <p style={{ color: 'var(--text-muted)', marginTop: 'var(--gap-md)' }}>Loading whiteboard...</p>
        </div>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className="cyber-card" style={{ marginBottom: 'var(--gap-md)' }}>
        <div className="cyber-card-body" style={{ padding: 'var(--gap-xl)', textAlign: 'center' }}>
          <p style={{ color: 'var(--error)' }}>{error}</p>
          <CyberButton variant="primary" onClick={loadWhiteboard} style={{ marginTop: 'var(--gap-md)' }}>
            Retry
          </CyberButton>
        </div>
      </div>
    );
  }

  // Full-screen pop-out mode
  if (isPoppedOut) {
    return (
      <div style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        zIndex: 9999,
        background: 'var(--bg-primary)',
      }}>
        {/* Header */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: 'var(--gap-sm) var(--gap-md)',
          background: 'var(--bg-elevated)',
          borderBottom: '1px solid var(--border-subtle)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--gap-md)' }}>
            <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>
              {whiteboard?.name || 'Whiteboard'} (Full Screen)
            </span>
            {collaborators.length > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--gap-xs)', color: 'var(--text-muted)', fontSize: '0.85rem' }}>
                <Users size={14} />
                <span>{collaborators.length + 1} online</span>
              </div>
            )}
            <div style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              background: isConnected ? 'var(--success)' : 'var(--error)',
            }} />
          </div>
          <div style={{ display: 'flex', gap: 'var(--gap-xs)' }}>
            <CyberButton variant="ghost" icon={Save} onClick={saveWhiteboard} disabled={!isDirty || isSaving}>
              {isSaving ? 'Saving...' : 'Save'}
            </CyberButton>
            <CyberButton variant="ghost" icon={X} onClick={() => setIsPoppedOut(false)}>
              Close
            </CyberButton>
          </div>
        </div>

        {/* Excalidraw */}
        <div style={{ height: 'calc(100vh - 50px)', width: '100%', position: 'relative' }}>
          <div style={{ width: '100%', height: '100%', position: 'absolute', top: 0, left: 0 }}>
            <Excalidraw
              ref={excalidrawRef}
              initialData={{
                elements: localElements,
                appState: {
                  theme: 'dark',
                  viewBackgroundColor: '#1e1e1e',
                },
              }}
              onChange={handleChange}
              onPointerUpdate={handlePointerUpdate}
              theme="dark"
              isCollaborating={collaborators.length > 0}
              collaborators={excalidrawCollaborators}
              UIOptions={{
                canvasActions: {
                  loadScene: true,
                  export: { saveFileToDisk: true },
                  toggleTheme: true,
                },
                tools: {
                  image: true,
                },
              }}
            />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="cyber-card" style={{ marginBottom: 'var(--gap-md)' }}>
      {/* Header */}
      <div
        className="cyber-card-header"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: 'var(--gap-sm) var(--gap-md)',
          cursor: 'pointer',
        }}
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--gap-sm)' }}>
          {isCollapsed ? <ChevronDown size={16} /> : <ChevronUp size={16} />}
          <span style={{ fontWeight: 500, color: 'var(--text-primary)' }}>
            {whiteboard?.name || 'Whiteboard'}
          </span>
          {isDirty && (
            <span style={{
              fontSize: '0.7rem',
              color: 'var(--warning)',
              background: 'rgba(251, 191, 36, 0.1)',
              padding: '2px 6px',
              borderRadius: '4px',
            }}>
              unsaved
            </span>
          )}
          {collaborators.length > 0 && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--gap-xs)', color: 'var(--success)', fontSize: '0.75rem' }}>
              <Users size={12} />
              <span>{collaborators.length + 1}</span>
            </div>
          )}
          <div style={{
            width: '6px',
            height: '6px',
            borderRadius: '50%',
            background: isConnected ? 'var(--success)' : 'var(--error)',
          }} />
        </div>

        <div
          style={{ display: 'flex', gap: 'var(--gap-xs)' }}
          onClick={(e) => e.stopPropagation()}
        >
          <CyberButton
            variant="ghost"
            icon={Save}
            onClick={saveWhiteboard}
            disabled={!isDirty || isSaving}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          >
            {isSaving ? 'Saving...' : 'Save'}
          </CyberButton>
          <CyberButton
            variant="ghost"
            icon={Download}
            onClick={handleExport}
            disabled={localElements.length === 0}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          >
            Export
          </CyberButton>
          <CyberButton
            variant="ghost"
            icon={Trash2}
            onClick={handleClear}
            disabled={localElements.length === 0}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          />
          <CyberButton
            variant="ghost"
            icon={ExternalLink}
            onClick={() => setIsPoppedOut(true)}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          >
            Pop Out
          </CyberButton>
          {onClose && (
            <CyberButton
              variant="ghost"
              icon={X}
              onClick={onClose}
              style={{ padding: '4px 8px', fontSize: '0.75rem' }}
            />
          )}
        </div>
      </div>

      {/* Body */}
      {!isCollapsed && (
        <>
          <div
            className="cyber-card-body excalidraw-container"
            style={{
              height: `${height}px`,
              width: '100%',
              padding: 0,
              background: '#1e1e1e',
              position: 'relative',
            }}
          >
            <div style={{ width: '100%', height: '100%', position: 'absolute', top: 0, left: 0 }}>
              <Excalidraw
                ref={excalidrawRef}
                initialData={{
                  elements: localElements,
                  appState: {
                    theme: 'dark',
                    viewBackgroundColor: '#1e1e1e',
                  },
                }}
                onChange={handleChange}
                onPointerUpdate={handlePointerUpdate}
                theme="dark"
                isCollaborating={collaborators.length > 0}
                collaborators={excalidrawCollaborators}
                UIOptions={{
                  canvasActions: {
                    loadScene: true,
                    export: { saveFileToDisk: true },
                    toggleTheme: true,
                  },
                  tools: {
                    image: true,
                  },
                }}
              />
            </div>
          </div>

          {/* Resize handle */}
          <div
            onMouseDown={handleMouseDown}
            style={{
              height: '8px',
              background: 'var(--bg-elevated)',
              borderTop: '1px solid var(--border-subtle)',
              cursor: 'ns-resize',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <div style={{
              width: '40px',
              height: '4px',
              background: 'var(--border-subtle)',
              borderRadius: '2px',
            }} />
          </div>
        </>
      )}
    </div>
  );
};

export default Whiteboard;
