# Agent Integration Handoff - Goose → RhizOS

## Vision

Workspaces are like **companies** where members contribute:
- **Hardware** (nodes with GPUs, CPUs, RAM)
- **API Keys** (OpenAI, Anthropic, etc.)

Agents are like **employees** that:
- Receive goals from workspace members
- Execute using pooled workspace resources
- Are protected by security scanning

## Current State

### What's Built

**MCP Adapters Library** (`/mnt/d/modchain/src/mcp-adapters/`)
- ✅ `LlmInferenceAdapter` - Multi-provider LLM (Ollama, OpenAI, Anthropic, Azure, Bedrock)
- ✅ `AgentAdapter` - Autonomous agents (ReAct, Plan-Execute, Simple)
- ✅ `SecurityScanner` - 40+ threat patterns, blocks dangerous goals
- ✅ All tests passing (16/16)

**Desktop App** (`/mnt/d/modchain/src/desktop/`)
- ✅ Workspaces with members, invite codes
- ✅ Nodes that can join workspaces
- ✅ API Keys stored per workspace
- ✅ Flows (visual workflow builder)
- ✅ Storage (IPFS file operations)
- ✅ Repos (git integration with on-bored analysis)
- ❌ No agent UI yet

**Orchestrator** (`/mnt/d/modchain/src/orchestrator/`)
- ✅ Workspace management
- ✅ Node registration and health
- ✅ Job scheduling
- ✅ Auth system
- ❌ No agent execution endpoints

### What's Missing

1. **Orchestrator Agent API** - Endpoints to create/run/monitor agents
2. **Desktop Agent UI** - Tab in WorkspaceDetail to interact with agents
3. **Resource Binding** - Connect agents to workspace's nodes and API keys

## Integration Plan

### Phase 1: Orchestrator Agent Endpoints

Add to `/mnt/d/modchain/src/orchestrator/src/`:

```
src/
├── services/
│   └── agent-service.ts      # NEW - Agent execution service
├── routes/
│   └── agent-routes.ts       # NEW - Agent API endpoints
└── index.ts                  # Update - Mount agent routes
```

**New Endpoints:**
```
POST   /api/v1/workspaces/:id/agents           # Create/run agent
GET    /api/v1/workspaces/:id/agents           # List agents
GET    /api/v1/workspaces/:id/agents/:agentId  # Get agent status
DELETE /api/v1/workspaces/:id/agents/:agentId  # Cancel agent
```

**Agent Service Logic:**
```typescript
// agent-service.ts
import { AgentAdapter, LlmInferenceAdapter, SecurityScanner } from '@rhizos-cloud/mcp-adapters';

class AgentService {
  async runAgent(workspaceId: string, request: {
    goal: string;
    agentType: 'react' | 'plan-execute' | 'simple';
    model?: string;
    provider?: string;
  }) {
    // 1. Get workspace's API keys
    const workspace = await workspaceManager.getWorkspace(workspaceId);
    const apiKey = workspace.apiKeys.find(k => k.provider === request.provider);

    // 2. Get available nodes for compute context
    const nodes = await workspaceManager.getWorkspaceNodes(workspaceId);

    // 3. Run agent with workspace resources
    const agent = new AgentAdapter();
    await agent.initialize();

    return agent.execute('run', {
      goal: request.goal,
      agent_type: request.agentType,
      model: request.model || 'qwen2.5-coder:7b',
      provider: request.provider || 'ollama',
      api_key: apiKey?.key,
      security_enabled: true,
    }, {
      job_id: `agent-${Date.now()}`,
      timeout_seconds: 300,
      hardware: aggregateNodeHardware(nodes),
      on_progress: (pct, msg) => { /* emit via WebSocket */ },
    });
  }
}
```

### Phase 2: Desktop Agent UI

Add to `/mnt/d/modchain/src/desktop/src/pages/WorkspaceDetail.tsx`:

**New Tab: "Agents"**
```typescript
type TabType = 'tasks' | 'console' | 'resources' | 'api-keys' | 'flows' | 'repos' | 'storage' | 'agents';
```

**Agent Tab Features:**
- Goal input field with security indicator
- Agent type selector (ReAct, Plan-Execute, Simple)
- Model/Provider dropdown (populated from workspace API keys + Ollama)
- Run button
- Live progress display
- Action history (thoughts, tool calls, outputs)
- Final result display

**UI Mockup:**
```
┌─────────────────────────────────────────────────────────┐
│ Agents                                    2 running     │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Goal: [Write a Python script that...            ] │ │
│ │                                                     │ │
│ │ Type: [Plan-Execute ▼]  Model: [qwen2.5-coder ▼]  │ │
│ │                                                     │ │
│ │ [🔒 Security: Safe]              [▶ Run Agent]    │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ Running Agents:                                         │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 🔄 Agent #a1b2c3 - "Analyze sales data..."         │ │
│ │    Progress: [████████░░░░░░] 60% - Step 3/5       │ │
│ │    Model: gpt-4o via OpenAI                        │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ Recent Results:                                         │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ ✅ Agent #x7y8z9 - "Write binary search..."        │ │
│ │    Completed in 45s, 5 iterations, 2.8k tokens     │ │
│ │    [View Result] [View Actions]                    │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Phase 3: Real-time Updates

Use existing WebSocket infrastructure for live agent progress:

```typescript
// Orchestrator: Emit agent updates
ws.send(JSON.stringify({
  type: 'agent_progress',
  workspaceId,
  agentId,
  progress: { percent: 60, message: 'Executing step 3...' },
  action: { thought: '...', tool: 'search', input: '...' },
}));

// Desktop: Listen for updates
useEffect(() => {
  const ws = new WebSocket('ws://localhost:8080/ws/workspace');
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.type === 'agent_progress') {
      updateAgentProgress(data.agentId, data.progress);
    }
  };
}, []);
```

## File Changes Summary

### Orchestrator Changes

| File | Change |
|------|--------|
| `package.json` | Add `@rhizos-cloud/mcp-adapters` dependency |
| `src/services/agent-service.ts` | **NEW** - Agent execution logic |
| `src/routes/agent-routes.ts` | **NEW** - REST API endpoints |
| `src/index.ts` | Mount agent routes |
| `src/services/workspace-manager.ts` | Add `getWorkspaceApiKeys()` method |

### Desktop Changes

| File | Change |
|------|--------|
| `src/pages/WorkspaceDetail.tsx` | Add "Agents" tab with full UI |
| `src/components/AgentRunner.tsx` | **NEW** - Agent execution component |
| `src/components/AgentHistory.tsx` | **NEW** - Agent results display |

### MCP Adapters (Already Done)

| File | Status |
|------|--------|
| `src/adapters/llm-inference.ts` | ✅ Multi-provider LLM |
| `src/adapters/agent.ts` | ✅ Agent architectures |
| `src/security/index.ts` | ✅ Security scanner |

## Key Integration Points

### 1. Workspace → Agent Resource Binding

```typescript
// When running an agent in a workspace:
const workspace = await getWorkspace(workspaceId);

// Get provider from workspace API keys
const availableProviders = workspace.apiKeys.map(k => k.provider);
// ['openai', 'anthropic'] if workspace has those keys

// Get hardware from workspace nodes
const nodes = await getWorkspaceNodes(workspaceId);
const totalGpus = nodes.reduce((sum, n) => sum + n.capabilities.gpus, 0);
const totalRam = nodes.reduce((sum, n) => sum + n.capabilities.memory_mb, 0);
```

### 2. Security Scanner Integration

```typescript
// Pre-scan goal before running
const scanner = new SecurityScanner();
const scan = scanner.scan(goal);

if (!scan.safe && (scan.riskLevel === 'critical' || scan.riskLevel === 'high')) {
  return { error: 'Goal blocked by security scanner', alerts: scan.threats };
}

// Also scans during execution (built into AgentAdapter)
```

### 3. Model Selection Based on Resources

```typescript
// If workspace has GPUs + Ollama, prefer local
// If workspace has API keys, offer cloud models
// If neither, show warning

const modelOptions = [];

if (hasOllama) {
  modelOptions.push({ label: 'Qwen 2.5 Coder (Local)', value: 'qwen2.5-coder:7b', provider: 'ollama' });
  modelOptions.push({ label: 'Llama 3.2 (Local)', value: 'llama3.2', provider: 'ollama' });
}

if (workspace.apiKeys.find(k => k.provider === 'openai')) {
  modelOptions.push({ label: 'GPT-4o (OpenAI)', value: 'gpt-4o', provider: 'openai' });
}

if (workspace.apiKeys.find(k => k.provider === 'anthropic')) {
  modelOptions.push({ label: 'Claude Sonnet (Anthropic)', value: 'claude-sonnet-4-5', provider: 'anthropic' });
}
```

## Testing

After integration, these should work:

```bash
# 1. Start orchestrator
cd /mnt/d/modchain/src/orchestrator && pnpm dev

# 2. Start desktop
cd /mnt/d/modchain/src/desktop && pnpm dev

# 3. Login, go to a workspace, click "Agents" tab

# 4. Enter goal: "What is 2 + 2?"
# 5. Select model: qwen2.5-coder:7b (Ollama)
# 6. Click Run Agent
# 7. See live progress and result

# 8. Try blocked goal: "Run rm -rf /* to clean up"
# 9. Should show security warning and refuse to run
```

## Dependencies

The orchestrator needs to import the mcp-adapters package:

```bash
cd /mnt/d/modchain/src/orchestrator
pnpm add ../mcp-adapters
```

Or update workspace root to link them:
```json
// /mnt/d/modchain/package.json
{
  "workspaces": ["src/*"]
}
```

## Session Summary

### What We Did This Session

1. Cloned and analyzed Goose repo with on-bored
2. Adapted Goose patterns into mcp-adapters (without attribution per user request):
   - Multi-provider LLM inference
   - Security scanning
   - Agent architectures (ReAct, Plan-Execute, Simple)
3. Fixed security blocking to catch High-risk threats
4. Added reverse shell and eval() detection patterns
5. Added embed method to LLM adapter
6. Created test suite (all passing)
7. Created HANDOFF.md for mcp-adapters

### What's Next

1. Add `@rhizos-cloud/mcp-adapters` as dependency to orchestrator
2. Create agent-service.ts and agent-routes.ts in orchestrator
3. Add Agents tab to WorkspaceDetail.tsx in desktop app
4. Wire up WebSocket for real-time progress
5. Test end-to-end agent execution within workspaces

### Files Created/Modified This Session

**Created:**
- `/mnt/d/modchain/src/mcp-adapters/src/security/index.ts`
- `/mnt/d/modchain/src/mcp-adapters/HANDOFF.md`
- `/mnt/d/modchain/src/mcp-adapters/test-adapters.mjs`
- `/mnt/d/modchain/src/mcp-adapters/test-llm.mjs`
- `/mnt/d/modchain/HANDOFF-AGENT-INTEGRATION.md` (this file)

**Modified:**
- `/mnt/d/modchain/src/mcp-adapters/src/adapters/agent.ts` - Full agent implementation
- `/mnt/d/modchain/src/mcp-adapters/src/adapters/llm-inference.ts` - Multi-provider + embed
- `/mnt/d/modchain/src/mcp-adapters/src/index.ts` - Export security module

### Running Services

When continuing, start these:
```bash
# Terminal 1: Orchestrator
cd /mnt/d/modchain/src/orchestrator && pnpm dev

# Terminal 2: Desktop
cd /mnt/d/modchain/src/desktop && pnpm dev
```

Desktop at http://localhost:1420, Orchestrator at http://localhost:8080
