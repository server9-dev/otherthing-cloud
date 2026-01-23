/**
 * RhizOS Orchestrator Types
 *
 * Core type definitions shared across the orchestrator.
 */

import { z } from 'zod';

// ============ Node Types ============

export const GpuSupportsSchema = z.object({
  cuda: z.boolean(),
  rocm: z.boolean(),
  vulkan: z.boolean(),
  metal: z.boolean(),
  opencl: z.boolean(),
});

export const GpuCapabilitySchema = z.object({
  vendor: z.string(),
  model: z.string(),
  vram_mb: z.number(),
  compute_capability: z.string().optional(),
  driver_version: z.string(),
  supports: GpuSupportsSchema,
});

export const CpuArchitectureSchema = z.enum(['X86_64', 'Aarch64', 'Arm', 'Unknown']);

export const CpuCapabilitySchema = z.object({
  vendor: z.string(),
  model: z.string(),
  cores: z.number(),
  threads: z.number(),
  frequency_mhz: z.number(),
  architecture: CpuArchitectureSchema,
  features: z.array(z.string()),
});

export const MemoryCapabilitySchema = z.object({
  total_mb: z.number(),
  available_mb: z.number(),
});

export const StorageTypeSchema = z.enum(['Ssd', 'Hdd', 'Nvme', 'Unknown']);

export const StorageCapabilitySchema = z.object({
  total_gb: z.number(),
  available_gb: z.number(),
  storage_type: StorageTypeSchema.optional(),
  path: z.string().optional(),
});

export const OllamaModelSchema = z.object({
  name: z.string(),
  size: z.number(), // bytes
  quantization: z.string().optional(),
  family: z.string().optional(),
  parameterSize: z.string().optional(),
});

export const OllamaCapabilitySchema = z.object({
  installed: z.boolean(),
  version: z.string().optional(),
  models: z.array(OllamaModelSchema),
  endpoint: z.string().optional(), // Usually http://localhost:11434
});

export type OllamaCapability = z.infer<typeof OllamaCapabilitySchema>;
export type OllamaModel = z.infer<typeof OllamaModelSchema>;

export const NodeCapabilitiesSchema = z.object({
  node_id: z.string(),
  node_version: z.string(),
  gpus: z.array(GpuCapabilitySchema),
  cpu: CpuCapabilitySchema,
  memory: MemoryCapabilitySchema,
  storage: StorageCapabilitySchema,
  docker_version: z.string().optional(),
  container_runtimes: z.array(z.string()),
  mcp_adapters: z.array(z.string()),
  ollama: OllamaCapabilitySchema.optional(),
});

export type NodeCapabilities = z.infer<typeof NodeCapabilitiesSchema>;
export type GpuCapability = z.infer<typeof GpuCapabilitySchema>;
export type CpuCapability = z.infer<typeof CpuCapabilitySchema>;

// ============ Job Types ============

export const JobStatusSchema = z.enum([
  'pending',
  'assigned',
  'running',
  'completed',
  'failed',
  'cancelled',
  'timeout',
]);

export type JobStatus = z.infer<typeof JobStatusSchema>;

export const JobRequirementsSchema = z.object({
  gpu: z.object({
    count: z.number(),
    min_vram_mb: z.number(),
    requires: z.array(z.string()).optional(),
    preferred_vendor: z.string().optional(),
  }).optional(),
  cpu: z.object({
    min_cores: z.number(),
    min_threads: z.number().optional(),
    architecture: z.string().optional(),
    required_features: z.array(z.string()).optional(),
  }).optional(),
  memory: z.object({
    min_mb: z.number(),
  }).optional(),
  storage: z.object({
    min_gb: z.number(),
    type: StorageTypeSchema.optional(),
  }).optional(),
  mcp_adapter: z.string(),
  container_runtime: z.string().optional(),
  max_cost_cents: z.number(),
  currency: z.string(),
});

export type JobRequirements = z.infer<typeof JobRequirementsSchema>;

export const JobPayloadSchema = z.object({
  type: z.string(),
}).passthrough(); // Allow additional fields

export type JobPayload = z.infer<typeof JobPayloadSchema>;

export const CreateJobRequestSchema = z.object({
  requirements: JobRequirementsSchema,
  payload: JobPayloadSchema,
  timeout_seconds: z.number().default(3600),
});

export type CreateJobRequest = z.infer<typeof CreateJobRequestSchema>;

export interface Job {
  id: string;
  client_id: string;
  workspace_id?: string; // Optional workspace for routing to workspace nodes
  requirements: JobRequirements;
  payload: JobPayload;
  status: JobStatus;
  assigned_node?: string;
  created_at: Date;
  started_at?: Date;
  completed_at?: Date;
  result?: JobResult;
}

export interface JobResult {
  success: boolean;
  outputs?: Array<{
    type: 'inline' | 'url' | 'cid';
    data: string;
    mime_type?: string;
  }>;
  error?: string;
  execution_time_ms: number;
  actual_cost_cents: number;
}

// ============ Protocol Messages ============

export const NodeMessageSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('register'),
    capabilities: NodeCapabilitiesSchema,
    auth_token: z.string().optional(),
  }),
  z.object({
    type: z.literal('heartbeat'),
    available: z.boolean(),
    current_jobs: z.number(),
  }),
  z.object({
    type: z.literal('job_status'),
    job_id: z.string(),
    status: z.enum(['accepted', 'preparing', 'running', 'completed', 'failed']),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('job_result'),
    job_id: z.string(),
    result: z.object({
      success: z.boolean(),
      outputs: z.array(z.any()).optional(),
      error: z.string().optional(),
      execution_time_ms: z.number(),
      actual_cost_cents: z.number(),
    }),
  }),
  z.object({
    type: z.literal('ipfs_ready'),
    peer_id: z.string(),
    addresses: z.array(z.string()),
  }),
  z.object({
    type: z.literal('ipfs_store_result'),
    request_id: z.string(),
    success: z.boolean(),
    cid: z.string().optional(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('ipfs_retrieve_result'),
    request_id: z.string(),
    success: z.boolean(),
    cid: z.string().optional(),
    content: z.string().optional(),
    error: z.string().optional(),
  }),
  // Sandbox operation results
  z.object({
    type: z.literal('sandbox_write_file_result'),
    request_id: z.string(),
    success: z.boolean(),
    path: z.string().optional(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_read_file_result'),
    request_id: z.string(),
    success: z.boolean(),
    content: z.string().optional(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_list_files_result'),
    request_id: z.string(),
    success: z.boolean(),
    files: z.array(z.any()).optional(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_delete_file_result'),
    request_id: z.string(),
    success: z.boolean(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_execute_result'),
    request_id: z.string(),
    success: z.boolean(),
    stdout: z.string(),
    stderr: z.string(),
    exitCode: z.number(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_sync_ipfs_result'),
    request_id: z.string(),
    success: z.boolean(),
    cid: z.string().optional(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('sandbox_restore_ipfs_result'),
    request_id: z.string(),
    success: z.boolean(),
    error: z.string().optional(),
  }),
  z.object({
    type: z.literal('pull_model_result'),
    request_id: z.string(),
    success: z.boolean(),
    model: z.string().optional(),
    error: z.string().optional(),
  }),
  // LLM inference result (node -> orchestrator)
  z.object({
    type: z.literal('llm_inference_result'),
    request_id: z.string(),
    success: z.boolean(),
    response: z.object({
      content: z.string(),
      model: z.string(),
      tokens_used: z.number().optional(),
      finish_reason: z.string().optional(),
    }).optional(),
    error: z.string().optional(),
  }),
]);

export type NodeMessage = z.infer<typeof NodeMessageSchema>;

export interface OrchestratorMessage {
  type: 'registered' | 'job_assignment' | 'cancel_job' | 'config_update' | 'error' | 'workspace_joined' | 'ollama_pull' | 'ollama_pull_status' | 'llm_inference';
  [key: string]: unknown;
}

// LLM inference request (orchestrator -> node)
export interface LlmInferenceRequest {
  type: 'llm_inference';
  request_id: string;
  model: string;
  messages: Array<{
    role: 'system' | 'user' | 'assistant';
    content: string;
  }>;
  max_tokens?: number;
  temperature?: number;
  stream?: boolean; // Future: support streaming
}

// LLM inference response (node -> orchestrator)
export interface LlmInferenceResponse {
  type: 'llm_inference_result';
  request_id: string;
  success: boolean;
  response?: {
    content: string;
    model: string;
    tokens_used?: number;
    finish_reason?: string;
  };
  error?: string;
}

// Ollama pull request (orchestrator -> node)
export interface OllamaPullMessage extends OrchestratorMessage {
  type: 'ollama_pull';
  model: string;
  requestId: string;
}

// Ollama pull status (node -> orchestrator)
export interface OllamaPullStatusMessage {
  type: 'ollama_pull_status';
  requestId: string;
  model: string;
  status: 'pulling' | 'completed' | 'failed';
  progress?: number; // 0-100
  error?: string;
}

// IPFS message types
export interface WorkspaceJoinedMessage extends OrchestratorMessage {
  type: 'workspace_joined';
  workspace_id: string;
  ipfs_swarm_key: string;
  bootstrap_peers: string[];
}

export interface IPFSReadyMessage {
  type: 'ipfs_ready';
  peer_id: string;
  addresses: string[];
}

// ============ Connected Node ============

export interface ResourceLimits {
  cpuCores?: number;
  ramPercent?: number;
  storageGb?: number;
  gpuVramPercent?: number[];
}

export interface ConnectedNode {
  id: string;
  capabilities: NodeCapabilities;
  ws: import('ws').WebSocket;
  available: boolean;
  current_jobs: number;
  last_heartbeat: Date;
  reputation: number;
  resourceLimits?: ResourceLimits;
  remoteControlEnabled?: boolean;
  // IPFS integration
  ipfsPeerId?: string;
  ipfsAddresses?: string[];
  ipfsReady?: boolean;
}

// ============ Registration ============

export const RegisterNodeRequestSchema = z.object({
  wallet_address: z.string(),
  capabilities: NodeCapabilitiesSchema,
});

export type RegisterNodeRequest = z.infer<typeof RegisterNodeRequestSchema>;

export interface RegisterNodeResponse {
  node_id: string;
  auth_token: string;
}

// ============ Flow Deployment Types ============

export const FlowNodeSchema = z.object({
  id: z.string(),
  moduleId: z.string(),
  moduleName: z.string(),
  moduleVersion: z.string().optional(),
  position: z.object({
    x: z.number(),
    y: z.number(),
  }),
  config: z.record(z.string(), z.unknown()).default({}),
  credentialRefs: z.record(z.string(), z.object({
    credentialId: z.string(),
    type: z.string(),
  })).optional(),
});

export const FlowConnectionSchema = z.object({
  id: z.string(),
  sourceNodeId: z.string(),
  sourcePort: z.string().default('output'),
  targetNodeId: z.string(),
  targetPort: z.string().default('input'),
});

export const DeployFlowRequestSchema = z.object({
  flowId: z.string(),
  name: z.string(),
  workspaceId: z.string().optional(), // Optional workspace for routing to workspace nodes
  nodes: z.array(FlowNodeSchema),
  connections: z.array(FlowConnectionSchema),
  resolvedCredentials: z.record(z.string(), z.record(z.string(), z.string())),
  options: z.object({
    dryRun: z.boolean().default(false),
    priority: z.enum(['low', 'normal', 'high']).default('normal'),
    maxCostCents: z.number().positive().optional(),
  }).optional(),
});

export type DeployFlowRequest = z.infer<typeof DeployFlowRequestSchema>;
export type FlowNode = z.infer<typeof FlowNodeSchema>;
export type FlowConnection = z.infer<typeof FlowConnectionSchema>;

export const FlowDeploymentStatusSchema = z.enum([
  'pending',
  'deploying',
  'running',
  'completed',
  'failed',
  'cancelled',
]);

export type FlowDeploymentStatus = z.infer<typeof FlowDeploymentStatusSchema>;

export interface FlowDeployment {
  id: string;
  flowId: string;
  name: string;
  clientId: string;
  workspaceId?: string; // Optional workspace for routing to workspace nodes
  status: FlowDeploymentStatus;
  nodes: FlowNode[];
  connections: FlowConnection[];
  nodeJobs: Record<string, string>; // nodeId -> jobId mapping
  nodeStatuses: Record<string, {
    status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
    jobId?: string;
    startedAt?: Date;
    completedAt?: Date;
    error?: string;
    output?: unknown;
  }>;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
  totalCostCents: number;
  error?: string;
}
