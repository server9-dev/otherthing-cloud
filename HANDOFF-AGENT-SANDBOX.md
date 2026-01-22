# Agent Sandbox Implementation Handoff

## Overview

Implement a real execution environment for agents that allows them to write files, execute code, and store outputs using IPFS on workspace nodes. Currently agents only generate text descriptions - they need actual tools to build and run code.

## Current State

### What Exists

**Agent Adapter** (`src/mcp-adapters/src/adapters/agent.ts`):
- Three architectures: `react`, `plan-execute`, `simple`
- Only simulated tools: `think`, `search` (fake), `calculate`
- Security scanner for input/output validation
- LLM inference integration working with Ollama

**Node Service** (`/mnt/d/github/node/src/node-service.ts`):
- WebSocket connection to orchestrator
- Job execution: `shell` and `docker` types
- IPFS integration via `ipfs_store` and `ipfs_retrieve` messages
- Storage path configuration per node

**IPFS Manager** (`/mnt/d/github/node/src/ipfs-manager.ts`):
- Full Kubo integration (add, get, pin, unpin)
- Workspace isolation via swarm keys
- Storage at `{storagePath}/otherthing-storage/ipfs`

**Orchestrator** (`src/orchestrator/`):
- Agent service coordinates execution
- Node manager tracks connected nodes with IPFS/Ollama capabilities
- Workspace manager stores workspace metadata

### What's Missing

1. **No workspace sandbox directory** on nodes
2. **No real file tools** for agents (read_file, write_file, list_files)
3. **No sandboxed shell execution** for agents
4. **No code execution environment** (no way to run Python, Node, etc.)
5. **Agent outputs not persisted** to IPFS

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Orchestrator                             │
│  ┌─────────────────┐    ┌─────────────────┐                     │
│  │  Agent Service  │───▶│  Node Manager   │                     │
│  │  (coordinates)  │    │  (routes to     │                     │
│  │                 │    │   best node)    │                     │
│  └─────────────────┘    └────────┬────────┘                     │
└──────────────────────────────────┼──────────────────────────────┘
                                   │ WebSocket
                    ┌──────────────┴──────────────┐
                    ▼                              ▼
            ┌───────────────┐              ┌───────────────┐
            │    Node A     │              │    Node B     │
            │  ┌─────────┐  │              │  ┌─────────┐  │
            │  │  IPFS   │  │              │  │  IPFS   │  │
            │  │ Storage │  │◀────────────▶│  │ Storage │  │
            │  └─────────┘  │   (swarm)    │  └─────────┘  │
            │  ┌─────────┐  │              │  ┌─────────┐  │
            │  │ Sandbox │  │              │  │ Sandbox │  │
            │  │ Workdir │  │              │  │ Workdir │  │
            │  └─────────┘  │              │  └─────────┘  │
            │  ┌─────────┐  │              │  ┌─────────┐  │
            │  │ Ollama  │  │              │  │ Ollama  │  │
            │  └─────────┘  │              │  └─────────┘  │
            └───────────────┘              └───────────────┘
```

## Implementation Plan

### Phase 1: Workspace Sandbox Directory on Nodes

**Goal**: Create isolated workspace directories on nodes for agent file operations.

**Node Changes** (`/mnt/d/github/node/src/`):

1. Create `sandbox-manager.ts`:
```typescript
interface SandboxConfig {
  workspaceId: string;
  basePath: string;  // e.g., {storagePath}/otherthing-storage/workspaces/{workspaceId}
  maxSizeBytes: number;
  allowedExtensions: string[];
}

class SandboxManager {
  // Create workspace sandbox directory
  async createSandbox(workspaceId: string): Promise<string>

  // File operations (all paths relative to sandbox root)
  async writeFile(workspaceId: string, relativePath: string, content: string): Promise<void>
  async readFile(workspaceId: string, relativePath: string): Promise<string>
  async listFiles(workspaceId: string, relativePath?: string): Promise<FileInfo[]>
  async deleteFile(workspaceId: string, relativePath: string): Promise<void>

  // Sync to/from IPFS
  async syncToIPFS(workspaceId: string): Promise<string>  // Returns root CID
  async syncFromIPFS(workspaceId: string, cid: string): Promise<void>

  // Cleanup
  async deleteSandbox(workspaceId: string): Promise<void>
}
```

2. Directory structure on node:
```
{storagePath}/otherthing-storage/
├── ipfs/                          # IPFS repo (existing)
└── workspaces/
    └── {workspace-id}/
        ├── sandbox/               # Agent working directory
        │   ├── code/              # Source files
        │   ├── output/            # Execution outputs
        │   └── data/              # Data files
        └── .sandbox-meta.json     # Metadata (size, last sync CID)
```

3. Add WebSocket handlers in `node-service.ts`:
```typescript
case 'sandbox_write_file':
  // Write file to workspace sandbox
  // Returns: { success, path }

case 'sandbox_read_file':
  // Read file from workspace sandbox
  // Returns: { success, content }

case 'sandbox_list_files':
  // List files in sandbox directory
  // Returns: { success, files: FileInfo[] }

case 'sandbox_execute':
  // Execute command in sandbox (sandboxed)
  // Returns: { success, stdout, stderr, exitCode }

case 'sandbox_sync_ipfs':
  // Sync sandbox to IPFS
  // Returns: { success, cid }
```

### Phase 2: Agent Tools (Orchestrator Side)

**Goal**: Add real tools to the agent that route operations to workspace nodes.

**Changes to Agent Adapter** (`src/mcp-adapters/src/adapters/agent.ts`):

1. Add tool execution context:
```typescript
interface AgentToolContext {
  workspaceId: string;
  nodeId: string;
  sendToNode: (message: any) => Promise<any>;
}
```

2. Register real tools:
```typescript
// File Operations
this.tools.set('write_file', {
  name: 'write_file',
  description: 'Write content to a file in the workspace sandbox',
  parameters: { path: 'string', content: 'string' },
  execute: async (params, ctx) => {
    const result = await ctx.sendToNode({
      type: 'sandbox_write_file',
      workspace_id: ctx.workspaceId,
      path: params.path,
      content: params.content,
    });
    return result.success ? `File written: ${params.path}` : `Error: ${result.error}`;
  },
});

this.tools.set('read_file', {
  name: 'read_file',
  description: 'Read content from a file in the workspace sandbox',
  parameters: { path: 'string' },
  execute: async (params, ctx) => {
    const result = await ctx.sendToNode({
      type: 'sandbox_read_file',
      workspace_id: ctx.workspaceId,
      path: params.path,
    });
    return result.success ? result.content : `Error: ${result.error}`;
  },
});

this.tools.set('list_files', {
  name: 'list_files',
  description: 'List files in the workspace sandbox directory',
  parameters: { path: 'string (optional)' },
  execute: async (params, ctx) => {
    const result = await ctx.sendToNode({
      type: 'sandbox_list_files',
      workspace_id: ctx.workspaceId,
      path: params.path || '.',
    });
    return result.success
      ? result.files.map(f => `${f.name} (${f.size} bytes)`).join('\n')
      : `Error: ${result.error}`;
  },
});

// Shell Execution
this.tools.set('shell', {
  name: 'shell',
  description: 'Execute a shell command in the workspace sandbox',
  parameters: { command: 'string' },
  execute: async (params, ctx) => {
    // Security check first
    const scan = this.securityScanner.scan(params.command);
    if (!scan.safe && scan.riskLevel >= RiskLevel.High) {
      return `Command blocked for security: ${scan.summary}`;
    }

    const result = await ctx.sendToNode({
      type: 'sandbox_execute',
      workspace_id: ctx.workspaceId,
      command: params.command,
      timeout: 30000,
    });

    if (!result.success) return `Error: ${result.error}`;
    let output = '';
    if (result.stdout) output += `stdout:\n${result.stdout}\n`;
    if (result.stderr) output += `stderr:\n${result.stderr}\n`;
    output += `exit code: ${result.exitCode}`;
    return output;
  },
});
```

### Phase 3: Orchestrator Integration

**Goal**: Route agent tool calls to appropriate nodes.

**Changes to Agent Service** (`src/orchestrator/src/services/agent-service.ts`):

1. Pass node communication context to agent:
```typescript
// When starting agent execution
const toolContext: AgentToolContext = {
  workspaceId: params.workspace_id,
  nodeId: selectedNode.nodeId,
  sendToNode: async (message) => {
    return this.nodeManager.sendToNode(selectedNode.nodeId, message);
  },
};
```

2. Add to NodeManager:
```typescript
// Send message to node and wait for response
async sendToNode(nodeId: string, message: any): Promise<any> {
  const node = this.nodes.get(nodeId);
  if (!node?.ws) throw new Error('Node not connected');

  const requestId = crypto.randomUUID();
  message.request_id = requestId;

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      this.pendingRequests.delete(requestId);
      reject(new Error('Node request timeout'));
    }, 60000);

    this.pendingRequests.set(requestId, { resolve, reject, timeout });
    node.ws.send(JSON.stringify(message));
  });
}
```

### Phase 4: IPFS Integration for Agent Outputs

**Goal**: Persist agent workspace state to IPFS.

1. After agent completes, sync sandbox to IPFS:
```typescript
// In agent-service.ts after agent execution
if (result.status === 'completed') {
  const syncResult = await this.nodeManager.sendToNode(nodeId, {
    type: 'sandbox_sync_ipfs',
    workspace_id: workspaceId,
  });

  if (syncResult.success) {
    result.workspace_cid = syncResult.cid;
    // Store CID in workspace for later retrieval
    this.workspaceManager.recordAgentOutput(workspaceId, {
      agentId: executionId,
      cid: syncResult.cid,
      timestamp: new Date().toISOString(),
    });
  }
}
```

2. On next agent run, optionally restore from IPFS:
```typescript
// If workspace has previous state
if (params.restore_workspace_cid) {
  await this.nodeManager.sendToNode(nodeId, {
    type: 'sandbox_restore_ipfs',
    workspace_id: workspaceId,
    cid: params.restore_workspace_cid,
  });
}
```

### Phase 5: Security Hardening

**Goal**: Ensure sandbox is secure.

1. **Command allowlist/blocklist**:
```typescript
const BLOCKED_COMMANDS = [
  /rm\s+-rf\s+\//, // rm -rf /
  /sudo/, /su\s/,
  /chmod.*777/,
  /curl.*\|.*sh/, // curl | sh
  /wget.*\|.*sh/,
  /mkfs/, /fdisk/, /dd\s+if=/,
];

const BLOCKED_PATHS = [
  '../',  // No path traversal
  '/etc/', '/usr/', '/bin/', '/sbin/',
  'C:\\Windows', 'C:\\Program Files',
];
```

2. **Resource limits on execution**:
```typescript
interface ExecutionLimits {
  maxCpuPercent: number;
  maxMemoryMb: number;
  maxDiskMb: number;
  maxTimeSeconds: number;
  networkAccess: boolean;
}
```

3. **Sandbox isolation options**:
   - **Basic**: Working directory restriction + command filtering
   - **Docker**: Run in container with limited capabilities
   - **VM**: (Future) Full VM isolation for untrusted code

## File Changes Summary

### Node App (`/mnt/d/github/node/`)

| File | Changes |
|------|---------|
| `src/sandbox-manager.ts` | **NEW** - Sandbox filesystem operations |
| `src/node-service.ts` | Add sandbox WebSocket handlers |
| `src/preload.ts` | Expose sandbox methods to renderer |
| `src/main.ts` | Initialize sandbox manager |

### Orchestrator (`/mnt/d/github/otherthing-cloud/src/orchestrator/`)

| File | Changes |
|------|---------|
| `src/services/node-manager.ts` | Add `sendToNode()` with request/response |
| `src/services/agent-service.ts` | Pass tool context, handle sandbox results |
| `src/services/workspace-manager.ts` | Add agent output tracking |

### MCP Adapters (`/mnt/d/github/otherthing-cloud/src/mcp-adapters/`)

| File | Changes |
|------|---------|
| `src/adapters/agent.ts` | Add real tools: write_file, read_file, list_files, shell |
| `src/security/index.ts` | Add command/path validation |

### Desktop App (`/mnt/d/github/otherthing-cloud/src/desktop/`)

| File | Changes |
|------|---------|
| `src/pages/WorkspaceDetail.tsx` | Show sandbox file browser (optional) |

## Testing Plan

1. **Unit tests for SandboxManager**:
   - Path validation (no traversal)
   - File size limits
   - Extension filtering

2. **Integration tests**:
   - Agent writes file → file exists in sandbox
   - Agent reads file → correct content returned
   - Agent executes command → output captured
   - Sandbox syncs to IPFS → CID returned

3. **Security tests**:
   - Blocked commands rejected
   - Path traversal blocked
   - Resource limits enforced

## Open Questions

1. **Multi-node workspace**: If a workspace has multiple nodes, which one runs the agent sandbox?
   - Option A: Always the node with Ollama (current compute node)
   - Option B: User selects preferred node
   - Option C: Load balance based on availability

2. **Sandbox persistence**: How long to keep sandbox directories?
   - Option A: Delete after agent completes (IPFS is source of truth)
   - Option B: Keep until workspace storage limit reached
   - Option C: User-controlled cleanup

3. **Network access**: Should agents have internet access?
   - For: Can install packages, fetch data
   - Against: Security risk, egress costs
   - Recommendation: Opt-in per workspace, with allowlist

## Implementation Order

1. **Week 1**: SandboxManager + basic file operations on Node
2. **Week 2**: WebSocket handlers + agent tools integration
3. **Week 3**: IPFS sync + orchestrator routing
4. **Week 4**: Security hardening + testing
5. **Week 5**: UI for sandbox file browser (optional)

## Related Files

- Previous handoff: `HANDOFF-AGENT-INTEGRATION.md`
- IPFS storage: `IPFS-STORAGE.md`
- Ollama integration: `HANDOFF-OLLAMA-NODE-INTEGRATION.md`
