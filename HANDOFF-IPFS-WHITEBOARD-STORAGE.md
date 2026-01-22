# Handoff: IPFS Whiteboard Storage & Node Sync

## Summary

This session implemented two major changes:
1. **Synced IPFS features to rhizos-node** - The standalone node repo now has full IPFS support
2. **IPFS-backed whiteboard storage** - Whiteboard data can now be stored in IPFS via workspace nodes

## What Was Built

### 1. rhizos-node Sync (Separate Repo)

**Repository:** https://github.com/Huck-dev/rhizos-node

**Files Added/Updated:**
- `src/ipfs-manager.ts` (NEW) - Full IPFS daemon management
- `src/node-service.ts` (UPDATED) - IPFS integration + message handlers
- `src/main.ts` (UPDATED) - IPC handlers for IPFS and drive selection
- `src/preload.ts` (UPDATED) - Exposed IPFS APIs to renderer
- `src/hardware.ts` (UPDATED) - Added `getDrives()` for storage selection
- `src/index.html` (UPDATED) - UI for IPFS status and drive selection

**New Message Handlers (node-service.ts):**
```typescript
case 'ipfs_store':
  // Store content in IPFS, pin it, return CID
  const cid = await this.ipfsManager.addContent(content, filename);
  await this.ipfsManager.pin(cid);
  ws.send({ type: 'ipfs_store_result', request_id, success: true, cid });

case 'ipfs_retrieve':
  // Retrieve content from IPFS by CID
  await this.ipfsManager.get(cid, tempPath);
  const content = fs.readFileSync(tempPath, 'utf-8');
  ws.send({ type: 'ipfs_retrieve_result', request_id, success: true, content });
```

**IPFS Manager Features:**
- `init()` - Initialize IPFS repo with lowpower profile
- `start()` / `stop()` - Daemon lifecycle management
- `addContent(content, filename)` - Store content, return CID
- `get(cid, outputPath)` - Retrieve content by CID
- `pin(cid)` / `unpin(cid)` - Persistence control
- `setSwarmKey(key)` - Private network isolation
- `connectPeer(multiaddr)` - Connect to workspace peers

### 2. Orchestrator IPFS Integration

**Files Changed:**
- `src/orchestrator/src/services/node-manager.ts`
- `src/orchestrator/src/services/workspace-manager.ts`
- `src/orchestrator/src/index.ts`
- `src/orchestrator/src/types/index.ts`

**New NodeManager Methods:**
```typescript
// Store content in IPFS via a workspace node
async storeInIPFS(workspaceId: string, content: string | object, filename?: string): Promise<string>

// Retrieve content from IPFS via a workspace node
async retrieveFromIPFS(workspaceId: string, cid: string): Promise<string>

// Find a node with IPFS ready for a workspace
findIPFSNodeForWorkspace(workspaceId: string): ConnectedNode | null

// Handle responses from nodes
handleIPFSStoreResult(message): void
handleIPFSRetrieveResult(message): void
```

**Updated WhiteboardData Interface:**
```typescript
export interface WhiteboardData {
  id: string;
  name: string;
  elements: any[];           // In-memory cache (may be empty if in IPFS)
  elementsCid?: string;      // IPFS CID for elements (NEW)
  appState?: any;
  files?: Record<string, any>;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  version: number;
}
```

**Updated API Endpoints:**
- `PATCH /api/v1/workspaces/:id/whiteboards/:whiteboardId`
  - Auto-stores in IPFS if elements > 10KB and IPFS node available
  - Accepts `storeInIPFS: true` to force IPFS storage
  - Clears `elements` array when stored in IPFS (keeps only CID)

- `GET /api/v1/workspaces/:id/whiteboards/:whiteboardId`
  - Retrieves elements from IPFS if `elementsCid` present and `elements` empty

- `GET /api/v1/workspaces/:id/whiteboards/default`
  - Same IPFS retrieval logic

## Architecture

### Data Flow (Save)
```
User saves whiteboard
        ↓
API endpoint receives elements
        ↓
Check: elements > 10KB AND IPFS node available?
        ↓ Yes                    ↓ No
Send ipfs_store to node    Store directly in workspace
        ↓
Node stores in IPFS, pins, returns CID
        ↓
Save CID in workspace, clear elements array
        ↓
Respond with full elements (for client cache)
```

### Data Flow (Load)
```
User requests whiteboard
        ↓
Get whiteboard from workspace-manager
        ↓
Check: elementsCid present AND elements empty?
        ↓ Yes                    ↓ No
Send ipfs_retrieve to node    Return directly
        ↓
Node retrieves from IPFS, returns content
        ↓
Parse JSON, return full whiteboard
```

### Message Protocol
```
Orchestrator → Node:
{
  type: 'ipfs_store',
  request_id: 'ipfs-store-1234-abc',
  content: '{"elements":[...]}',
  filename: 'whiteboard-xyz.json'
}

Node → Orchestrator:
{
  type: 'ipfs_store_result',
  request_id: 'ipfs-store-1234-abc',
  success: true,
  cid: 'QmXyz...'
}

Orchestrator → Node:
{
  type: 'ipfs_retrieve',
  request_id: 'ipfs-retrieve-5678-def',
  cid: 'QmXyz...'
}

Node → Orchestrator:
{
  type: 'ipfs_retrieve_result',
  request_id: 'ipfs-retrieve-5678-def',
  success: true,
  cid: 'QmXyz...',
  content: '{"elements":[...]}'
}
```

## Testing

### Prerequisites
1. Build and run rhizos-node with IPFS binary included
2. Select a storage path in node settings
3. Start IPFS daemon (automatic when joining workspace with swarm key)

### Test Steps
1. Start orchestrator: `cd src/orchestrator && pnpm dev`
2. Start desktop: `cd src/desktop && pnpm dev`
3. Start rhizos-node and add it to a workspace
4. Navigate to workspace → Whiteboard tab
5. Draw something substantial (enough elements to exceed 10KB)
6. Click Save
7. Check orchestrator logs for:
   - `[Whiteboard] Storing elements in IPFS via node xxx`
   - `[Whiteboard] Stored elements with CID: Qm...`
8. Refresh page
9. Check logs for:
   - `[Whiteboard] Retrieving elements from IPFS: Qm...`
   - `[Whiteboard] Retrieved X elements from IPFS`

### Force IPFS Storage (for testing)
```javascript
// In browser console or API call
fetch('/api/v1/workspaces/WORKSPACE_ID/whiteboards/BOARD_ID', {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    elements: [...],
    storeInIPFS: true  // Force IPFS even for small data
  })
});
```

## Known Issues / TODOs

1. **IPFS Binary Distribution** - rhizos-node expects IPFS binary in `resources/ipfs/`. Need to bundle with releases.

2. **Fallback Behavior** - Currently silent fallback to direct storage. May want to inform user.

3. **Large Files (images)** - `files` field in whiteboard (embedded images) not yet stored in IPFS.

4. **Garbage Collection** - Old CIDs not unpinned when whiteboard updated. Could accumulate.

5. **Multi-Node Sync** - If multiple nodes in workspace, only one stores. Others should pin for redundancy.

6. **node-electron Deprecation** - The `src/node-electron` folder in monorepo is now redundant. Consider removing or marking deprecated.

## Commits

**rhizos-node:**
- `308eb8c` - feat: Add IPFS storage and whiteboard support

**rhizos-cloud:**
- `aec550c` - fix: Import Excalidraw CSS for proper toolbar rendering
- `9703aa0` - feat: Add IPFS storage for whiteboard data

## Next Steps

1. Bundle IPFS binary with rhizos-node releases
2. Add IPFS status indicator to desktop UI (shows if whiteboard is IPFS-backed)
3. Implement multi-node pinning for redundancy
4. Add CID garbage collection for old whiteboard versions
5. Store embedded images in IPFS (files field)
6. Remove/deprecate src/node-electron from monorepo
