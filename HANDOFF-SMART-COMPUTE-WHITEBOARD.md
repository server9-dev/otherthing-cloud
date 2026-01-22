# Handoff: Smart Compute Orchestration & Whiteboard Integration

## Summary

This session implemented two major features:
1. **Smart Compute Orchestration** - Agents now automatically select the best model and prioritize local compute (Ollama) over cloud APIs
2. **Whiteboard Integration** - Interactive drawing canvas using Excalidraw with pop-out and resize capabilities

## What Was Built

### 1. Smart Compute Orchestration

**Files Changed/Created:**
- `src/orchestrator/src/services/model-selector.ts` (NEW)
- `src/orchestrator/src/services/agent-service.ts` (MODIFIED)
- `src/orchestrator/src/services/node-manager.ts` (MODIFIED)
- `src/orchestrator/src/types/index.ts` (MODIFIED)
- `src/orchestrator/src/index.ts` (MODIFIED)
- `src/desktop/src/pages/WorkspaceDetail.tsx` (MODIFIED)

**Key Features:**

1. **Task Categorization** (`model-selector.ts:128-160`)
   - Analyzes goal text for keywords
   - Categories: `coding`, `general`, `analysis`, `creative`, `math`
   - Each category has optimal model recommendations

2. **Model Recommendations** (`model-selector.ts:43-108`)
   ```
   coding → qwen2.5-coder (7b/14b/32b local) or gpt-4o/claude-sonnet-4-5 (cloud)
   general → llama3.2 (local) or gpt-4o-mini/claude-haiku-4-5 (cloud)
   analysis → qwen2.5 (local) or gpt-4o/claude-sonnet-4-5 (cloud)
   creative → mistral/llama3.1 (local) or gpt-4o/claude-sonnet-4-5 (cloud)
   math → qwen2.5 (local) or gpt-4o/claude-sonnet-4-5 (cloud)
   ```

3. **Local-First Compute** (`agent-service.ts:166-273`)
   - Checks workspace nodes for Ollama first
   - Selects largest model that fits in available VRAM
   - Auto-pulls model if needed (with progress tracking)
   - Falls back to cloud only if no local compute

4. **Ollama Model Pull** (`node-manager.ts`)
   - `pullModel(nodeId, model)` - Sends pull request to node
   - `handlePullStatus(message)` - Handles progress/completion
   - `getOllamaNodesForWorkspace(workspaceId)` - Lists Ollama-capable nodes
   - `findBestNodeForModel(workspaceId, model, vramNeeded)` - Finds optimal node

5. **New API Endpoints** (`index.ts`)
   - `GET /api/v1/workspaces/:id/compute` - Returns compute summary
   - `POST /api/v1/workspaces/:id/agents/analyze` - Analyzes task and returns recommendation

6. **UI Updates** (`WorkspaceDetail.tsx`)
   - Compute summary bar showing local nodes and cloud keys
   - Task analysis showing recommended model/provider
   - "Auto" option for provider/model (uses smart selection)
   - Visual indicators: ⚡ Local / ☁️ Cloud
   - `pulling_model` status with download progress

### 2. Whiteboard Integration

**Files Changed/Created:**
- `src/desktop/src/components/Whiteboard.tsx` (NEW)
- `src/desktop/src/components/index.ts` (MODIFIED)
- `src/desktop/src/pages/WorkspaceDetail.tsx` (MODIFIED)
- `src/desktop/package.json` (MODIFIED - added @excalidraw/excalidraw)

**Key Features:**

1. **Whiteboard Component** (`Whiteboard.tsx`)
   - Uses `@excalidraw/excalidraw` for drawing
   - Dark theme by default
   - Collapsible header
   - Resizable height (drag handle)
   - Pop-out to full-screen overlay
   - Save/Export/Clear functionality
   - Dirty state tracking

2. **Pop-Out Mode**
   - Full-screen overlay (not a new browser window)
   - Same Excalidraw instance
   - Changes sync back to main component

3. **Workspace Integration**
   - New "Whiteboard" tab in workspace detail
   - Board data stored in component state
   - Ready for storage integration (save to workspace files)

## Architecture Notes

### Smart Compute Flow
```
User enters goal → scanGoal (security) + analyzeTask (recommendation)
                            ↓
          Display: task category, recommended model, compute source
                            ↓
User clicks "Run Agent" (model/provider can be Auto or explicit)
                            ↓
         AgentService.runAgent() with preferLocal=true
                            ↓
    Check workspace nodes for Ollama → Found? → Has model? → Run locally
                            ↓ No                   ↓ No
                    Has VRAM?  →  Yes  →  Pull model → Run locally
                            ↓ No
                    Has cloud keys? → Yes → Use cloud API
                            ↓ No
                    Error: No compute available
```

### Whiteboard State
```
WhiteboardData {
  elements: ExcalidrawElement[]  // Drawing elements
  appState?: AppState            // View state (zoom, scroll, etc.)
  files?: BinaryFiles            // Embedded images
}
```

## Known Issues / TODOs

1. **Whiteboard Persistence** - Currently only stored in React state. Need to:
   - Save to workspace IPFS storage
   - Load on workspace open
   - Handle multiple boards per workspace

2. **Excalidraw Bundle Size** - Adds ~1.8MB to bundle due to font subsetting
   - Consider lazy loading for production

3. **Node Model Pull Progress** - The node-to-orchestrator message flow for pull progress needs testing with real nodes

4. **React 19 Peer Deps** - Excalidraw has peer dependency warnings for React 18 vs 19, but works

## Testing

To test smart compute:
1. Start orchestrator: `cd src/orchestrator && pnpm dev`
2. Start desktop: `cd src/desktop && pnpm dev`
3. Navigate to a workspace → Agents tab
4. Enter a goal (e.g., "Write a Python function to sort a list")
5. Observe task analysis showing category and recommendation
6. Run with "Auto" to see smart selection in action

To test whiteboard:
1. Navigate to workspace → Whiteboard tab
2. Draw something
3. Try resize handle at bottom
4. Click "Pop Out" for full-screen
5. Export as PNG

## Commits

- `e52b788` - feat: Smart compute orchestration for agents
- (pending) - feat: Add whiteboard with Excalidraw integration

## Next Steps

1. Wire up whiteboard save/load to workspace storage
2. Add collaboration features (share whiteboard state)
3. Test model pull flow with real nodes
4. Add model download progress UI in node manager
5. Consider code splitting for Excalidraw
