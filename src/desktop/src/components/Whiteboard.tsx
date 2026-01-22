/**
 * Whiteboard Component
 *
 * An interactive whiteboard using Excalidraw that can be:
 * - Embedded inline with resizable height
 * - Popped out to a separate window
 * - Saved/loaded from workspace storage
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { Excalidraw, exportToBlob, serializeAsJSON } from '@excalidraw/excalidraw';
import type { ExcalidrawElement } from '@excalidraw/excalidraw/element/types';
import type { AppState, BinaryFiles } from '@excalidraw/excalidraw/types';
import {
  Maximize2, Minimize2, ExternalLink, Save, Download, Trash2,
  ChevronUp, ChevronDown, X
} from 'lucide-react';
import { CyberButton } from './CyberButton';

// Types for whiteboard data
interface WhiteboardData {
  elements: readonly ExcalidrawElement[];
  appState?: Partial<AppState>;
  files?: BinaryFiles;
}

interface WhiteboardProps {
  workspaceId: string;
  boardId?: string;
  boardName?: string;
  initialData?: WhiteboardData;
  onSave?: (data: WhiteboardData) => Promise<void>;
  onClose?: () => void;
  minHeight?: number;
  maxHeight?: number;
  defaultHeight?: number;
}

// Pop-out window component
const PopOutWhiteboard = ({
  data,
  onClose,
  onDataChange,
}: {
  data: WhiteboardData;
  onClose: () => void;
  onDataChange: (data: WhiteboardData) => void;
}) => {
  const excalidrawRef = useRef<any>(null);

  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[], appState: AppState, files: BinaryFiles) => {
      onDataChange({ elements, appState, files });
    },
    [onDataChange]
  );

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
        <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>
          Whiteboard (Full Screen)
        </span>
        <CyberButton variant="ghost" icon={X} onClick={onClose}>
          Close
        </CyberButton>
      </div>

      {/* Excalidraw */}
      <div style={{ height: 'calc(100vh - 50px)' }}>
        <Excalidraw
          ref={excalidrawRef}
          initialData={{
            elements: data.elements,
            appState: {
              ...data.appState,
              theme: 'dark',
            },
            files: data.files,
          }}
          onChange={handleChange}
          theme="dark"
        />
      </div>
    </div>
  );
};

export const Whiteboard = ({
  workspaceId,
  boardId,
  boardName = 'Untitled Board',
  initialData,
  onSave,
  onClose,
  minHeight = 300,
  maxHeight = 800,
  defaultHeight = 500,
}: WhiteboardProps) => {
  const [height, setHeight] = useState(defaultHeight);
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isPoppedOut, setIsPoppedOut] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [data, setData] = useState<WhiteboardData>({
    elements: initialData?.elements || [],
    appState: initialData?.appState,
    files: initialData?.files,
  });

  const excalidrawRef = useRef<any>(null);
  const resizeRef = useRef<HTMLDivElement>(null);
  const isResizing = useRef(false);

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

  // Handle data change
  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[], appState: AppState, files: BinaryFiles) => {
      setData({ elements, appState, files });
      setIsDirty(true);
    },
    []
  );

  // Save handler
  const handleSave = async () => {
    if (!onSave) return;

    setIsSaving(true);
    try {
      await onSave(data);
      setIsDirty(false);
    } catch (err) {
      console.error('Failed to save whiteboard:', err);
    } finally {
      setIsSaving(false);
    }
  };

  // Export as PNG
  const handleExport = async () => {
    if (!excalidrawRef.current || data.elements.length === 0) return;

    try {
      const blob = await exportToBlob({
        elements: data.elements as ExcalidrawElement[],
        appState: data.appState as AppState,
        files: data.files || {},
        mimeType: 'image/png',
      });

      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${boardName.replace(/\s+/g, '-')}.png`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export:', err);
    }
  };

  // Clear board
  const handleClear = () => {
    if (window.confirm('Clear all drawings? This cannot be undone.')) {
      setData({ elements: [], appState: undefined, files: undefined });
      setIsDirty(true);
    }
  };

  // Pop out to new window
  const handlePopOut = () => {
    setIsPoppedOut(true);
  };

  // Close pop-out
  const handlePopOutClose = () => {
    setIsPoppedOut(false);
  };

  // If popped out, render full-screen overlay
  if (isPoppedOut) {
    return (
      <PopOutWhiteboard
        data={data}
        onClose={handlePopOutClose}
        onDataChange={setData}
      />
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
            {boardName}
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
        </div>

        <div
          style={{ display: 'flex', gap: 'var(--gap-xs)' }}
          onClick={(e) => e.stopPropagation()}
        >
          {onSave && (
            <CyberButton
              variant="ghost"
              icon={Save}
              onClick={handleSave}
              disabled={!isDirty || isSaving}
              style={{ padding: '4px 8px', fontSize: '0.75rem' }}
            >
              {isSaving ? 'Saving...' : 'Save'}
            </CyberButton>
          )}
          <CyberButton
            variant="ghost"
            icon={Download}
            onClick={handleExport}
            disabled={data.elements.length === 0}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          >
            Export
          </CyberButton>
          <CyberButton
            variant="ghost"
            icon={Trash2}
            onClick={handleClear}
            disabled={data.elements.length === 0}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          />
          <CyberButton
            variant="ghost"
            icon={ExternalLink}
            onClick={handlePopOut}
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
            className="cyber-card-body"
            style={{
              height: `${height}px`,
              padding: 0,
              overflow: 'hidden',
              background: '#1e1e1e',
            }}
          >
            <Excalidraw
              ref={excalidrawRef}
              initialData={{
                elements: data.elements,
                appState: {
                  ...data.appState,
                  theme: 'dark',
                },
                files: data.files,
              }}
              onChange={handleChange}
              theme="dark"
            />
          </div>

          {/* Resize handle */}
          <div
            ref={resizeRef}
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
