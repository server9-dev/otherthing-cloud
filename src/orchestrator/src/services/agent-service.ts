/**
 * Agent Service
 *
 * Manages agent execution within workspaces using pooled resources.
 * Smart compute orchestration:
 * 1. Check workspace nodes for Ollama
 * 2. Auto-select best model for the task
 * 3. Pull model if needed
 * 4. Fall back to cloud APIs only if no local compute
 */

import { AgentAdapter, LlmInferenceAdapter, SecurityScanner, RiskLevel } from '@rhizos-cloud/mcp-adapters';
import { v4 as uuidv4 } from 'uuid';
import { modelSelector, ModelRecommendation, ComputeAvailability } from './model-selector.js';
import { ConnectedNode } from '../types/index.js';

// Types for external managers (injected)
interface NodeManagerLike {
  getOllamaNodesForWorkspace(workspaceId: string): ConnectedNode[];
  findBestNodeForModel(workspaceId: string, model: string, vramNeeded: number): ConnectedNode | null;
  pullModel(nodeId: string, model: string): Promise<{ success: boolean; error?: string }>;
  findSandboxNodeForWorkspace(workspaceId: string): ConnectedNode | null;
  // Sandbox operations
  sandboxWriteFile(nodeId: string, workspaceId: string, path: string, content: string): Promise<{ success: boolean; path?: string; error?: string }>;
  sandboxReadFile(nodeId: string, workspaceId: string, path: string): Promise<{ success: boolean; content?: string; error?: string }>;
  sandboxListFiles(nodeId: string, workspaceId: string, path?: string): Promise<{ success: boolean; files?: any[]; error?: string }>;
  sandboxDeleteFile(nodeId: string, workspaceId: string, path: string): Promise<{ success: boolean; error?: string }>;
  sandboxExecute(nodeId: string, workspaceId: string, command: string, timeout?: number): Promise<{ success: boolean; stdout: string; stderr: string; exitCode: number; error?: string }>;
  sandboxSyncToIPFS(nodeId: string, workspaceId: string): Promise<{ success: boolean; cid?: string; error?: string }>;
  sandboxRestoreFromIPFS(nodeId: string, workspaceId: string, cid: string): Promise<{ success: boolean; error?: string }>;
}

// Tool context passed to agents for sandbox operations
export interface AgentToolContext {
  workspaceId: string;
  nodeId: string | null;
  nodeManager: NodeManagerLike | null;
}

interface WorkspaceManagerLike {
  getWorkspace(workspaceId: string): { apiKeys?: Array<{ provider: string; key: string }> } | null;
}

// Agent execution request
export interface AgentRequest {
  goal: string;
  agentType?: 'react' | 'plan-execute' | 'simple';
  model?: string; // If not specified, will auto-select
  provider?: 'ollama' | 'openai' | 'anthropic' | 'azure' | 'bedrock';
  maxIterations?: number;
  maxTokens?: number;
  temperature?: number;
  preferLocal?: boolean; // Default true - prefer local compute over cloud
}

// Agent execution state
export interface AgentExecution {
  id: string;
  workspaceId: string;
  userId: string;
  goal: string;
  agentType: string;
  model: string;
  provider: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'blocked' | 'pulling_model';
  progress: number;
  progressMessage: string;
  actions: Array<{
    thought: string;
    tool?: string;
    input?: string;
    output?: string;
  }>;
  result?: string;
  error?: string;
  securityAlerts?: string[];
  tokensUsed: number;
  iterations: number;
  createdAt: Date;
  completedAt?: Date;
  // Compute info
  computeSource?: 'local' | 'cloud';
  nodeId?: string;
  ollamaEndpoint?: string; // Remote node's Ollama endpoint URL
  modelPulled?: boolean;
  taskCategory?: string;
  // Sandbox info
  sandboxNodeId?: string; // Node used for sandbox operations
  sandboxCid?: string; // IPFS CID of final sandbox state
}

// Progress callback type
type ProgressCallback = (agentId: string, progress: number, message: string, action?: any) => void;

export class AgentService {
  private executions: Map<string, AgentExecution> = new Map();
  private agentAdapter: AgentAdapter;
  private llmAdapter: LlmInferenceAdapter;
  private securityScanner: SecurityScanner;
  private initialized = false;
  private onProgress?: ProgressCallback;

  // External managers (injected)
  private nodeManager?: NodeManagerLike;
  private workspaceManager?: WorkspaceManagerLike;

  constructor() {
    this.agentAdapter = new AgentAdapter();
    this.llmAdapter = new LlmInferenceAdapter();
    this.securityScanner = new SecurityScanner();
  }

  /**
   * Set external managers for compute orchestration
   */
  setManagers(nodeManager: NodeManagerLike, workspaceManager: WorkspaceManagerLike): void {
    this.nodeManager = nodeManager;
    this.workspaceManager = workspaceManager;
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;
    await this.agentAdapter.initialize();
    await this.llmAdapter.initialize();
    this.initialized = true;
    console.log('[AgentService] Initialized with smart compute orchestration');
  }

  /**
   * Set progress callback for real-time updates
   */
  setProgressCallback(callback: ProgressCallback): void {
    this.onProgress = callback;
  }

  /**
   * Pre-scan a goal for security threats (for UI feedback)
   */
  scanGoal(goal: string): { safe: boolean; riskLevel: string | null; alerts: string[] } {
    const result = this.securityScanner.scan(goal);
    return {
      safe: result.safe,
      riskLevel: result.riskLevel,
      alerts: result.threats.map(t => t.pattern.description),
    };
  }

  /**
   * Analyze task and get compute recommendation
   */
  analyzeTask(goal: string, workspaceId: string): {
    category: string;
    recommendation: ModelRecommendation;
    compute: ComputeAvailability;
  } {
    const category = modelSelector.categorizeTask(goal);
    const compute = this.getComputeAvailability(workspaceId);
    const recommendation = modelSelector.selectModel(goal, compute, true);

    return { category, recommendation, compute };
  }

  /**
   * Get compute availability for a workspace
   */
  private getComputeAvailability(workspaceId: string): ComputeAvailability {
    if (!this.nodeManager || !this.workspaceManager) {
      // Fallback if managers not set
      return {
        hasLocalCompute: false,
        ollamaNodes: [],
        apiKeys: { openai: false, anthropic: false },
      };
    }

    const nodes = this.nodeManager.getOllamaNodesForWorkspace(workspaceId);
    const workspace = this.workspaceManager.getWorkspace(workspaceId);
    const apiKeys = workspace?.apiKeys || [];

    return modelSelector.getComputeAvailability(nodes, apiKeys);
  }

  /**
   * Run an agent within a workspace context with smart compute selection
   */
  async runAgent(
    workspaceId: string,
    userId: string,
    request: AgentRequest,
    apiKeyOverride?: string
  ): Promise<AgentExecution> {
    await this.initialize();

    const executionId = uuidv4();
    const agentType = request.agentType || 'simple';
    const preferLocal = request.preferLocal !== false; // Default true

    // Analyze task and get recommendation
    const { category, recommendation, compute } = this.analyzeTask(request.goal, workspaceId);

    // Use explicit model/provider if provided, otherwise use recommendation
    const model = request.model || recommendation.model;
    const provider = request.provider || recommendation.provider;

    console.log(`[AgentService] Task category: ${category}, recommended: ${recommendation.model} via ${recommendation.provider}`);
    console.log(`[AgentService] Using: ${model} via ${provider} (reason: ${recommendation.reason})`);

    // Get the Ollama endpoint from the selected node (if using local compute)
    let ollamaEndpoint: string | undefined;
    if (provider === 'ollama' && recommendation.nodeId) {
      const node = compute.ollamaNodes.find(n => n.nodeId === recommendation.nodeId);
      ollamaEndpoint = node?.endpoint;
      if (ollamaEndpoint) {
        console.log(`[AgentService] Using Ollama endpoint from node: ${ollamaEndpoint}`);
      }
    }

    // Find sandbox node for this workspace
    let sandboxNodeId: string | null = null;
    if (this.nodeManager) {
      const sandboxNode = this.nodeManager.findSandboxNodeForWorkspace(workspaceId);
      sandboxNodeId = sandboxNode?.id || null;
      if (sandboxNodeId) {
        console.log(`[AgentService] Using sandbox node: ${sandboxNodeId}`);
      }
    }

    // Create execution record
    const execution: AgentExecution = {
      id: executionId,
      workspaceId,
      userId,
      goal: request.goal,
      agentType,
      model,
      provider,
      status: 'pending',
      progress: 0,
      progressMessage: 'Analyzing task...',
      actions: [],
      tokensUsed: 0,
      iterations: 0,
      createdAt: new Date(),
      taskCategory: category,
      computeSource: provider === 'ollama' ? 'local' : 'cloud',
      nodeId: recommendation.nodeId,
      ollamaEndpoint,
      sandboxNodeId: sandboxNodeId || undefined,
    };

    this.executions.set(executionId, execution);

    // Pre-scan goal
    const goalScan = this.securityScanner.scan(request.goal);
    if (!goalScan.safe && (goalScan.riskLevel === RiskLevel.Critical || goalScan.riskLevel === RiskLevel.High)) {
      execution.status = 'blocked';
      execution.error = `Goal blocked by security scanner: ${goalScan.summary}`;
      execution.securityAlerts = goalScan.threats.map(t => t.pattern.description);
      execution.completedAt = new Date();
      return execution;
    }

    // Get API key
    let apiKey = apiKeyOverride;
    if (!apiKey && provider !== 'ollama') {
      const workspace = this.workspaceManager?.getWorkspace(workspaceId);
      const wsApiKey = workspace?.apiKeys?.find(k => k.provider === provider);
      apiKey = wsApiKey?.key;

      if (!apiKey) {
        execution.status = 'failed';
        execution.error = `No ${provider} API key configured in workspace`;
        execution.completedAt = new Date();
        return execution;
      }
    }

    // Handle model pull if needed (local compute)
    if (provider === 'ollama' && recommendation.needsPull && recommendation.nodeId && this.nodeManager) {
      execution.status = 'pulling_model';
      execution.progressMessage = `Downloading ${model}...`;
      this.emitProgress(executionId, 5, `Downloading ${model} to node...`);

      const pullResult = await this.nodeManager.pullModel(recommendation.nodeId, model);

      if (!pullResult.success) {
        // Pull failed - try cloud fallback
        console.log(`[AgentService] Model pull failed: ${pullResult.error}, checking cloud fallback`);

        if (compute.apiKeys.anthropic || compute.apiKeys.openai) {
          const cloudProvider = compute.apiKeys.anthropic ? 'anthropic' : 'openai';
          const cloudModel = cloudProvider === 'anthropic' ? 'claude-sonnet-4-5' : 'gpt-4o';

          execution.model = cloudModel;
          execution.provider = cloudProvider;
          execution.computeSource = 'cloud';
          execution.progressMessage = `Falling back to ${cloudProvider}...`;

          const workspace = this.workspaceManager?.getWorkspace(workspaceId);
          apiKey = workspace?.apiKeys?.find(k => k.provider === cloudProvider)?.key;
        } else {
          execution.status = 'failed';
          execution.error = `Failed to pull model: ${pullResult.error}. No cloud API keys available as fallback.`;
          execution.completedAt = new Date();
          return execution;
        }
      } else {
        execution.modelPulled = true;
      }
    }

    // Run agent asynchronously
    this.executeAgent(execution, request, apiKey);

    return execution;
  }

  /**
   * Execute agent (runs in background)
   */
  private async executeAgent(
    execution: AgentExecution,
    request: AgentRequest,
    apiKey?: string
  ): Promise<void> {
    execution.status = 'running';
    this.emitProgress(execution.id, 10, `Running ${execution.agentType} agent with ${execution.model}...`);

    // Build tool context for sandbox operations
    const toolContext: AgentToolContext = {
      workspaceId: execution.workspaceId,
      nodeId: execution.sandboxNodeId || null,
      nodeManager: this.nodeManager || null,
    };

    try {
      const result = await this.agentAdapter.execute('run', {
        goal: request.goal,
        agent_type: execution.agentType,
        model: execution.model,
        provider: execution.provider,
        api_key: apiKey,
        base_url: execution.ollamaEndpoint, // Pass node's Ollama endpoint for remote access
        max_iterations: request.maxIterations || 10,
        max_tokens: request.maxTokens || 4096,
        temperature: request.temperature || 0.7,
        security_enabled: true,
        // Pass tool context for sandbox operations
        tool_context: toolContext,
      }, {
        job_id: execution.id,
        timeout_seconds: 300,
        hardware: { gpus: [], cpu_cores: 4, memory_mb: 8192 },
        on_progress: (pct: number, msg?: string) => {
          // Adjust progress to account for model pull phase (10-100)
          const adjustedPct = 10 + (pct * 0.9);
          execution.progress = adjustedPct;
          execution.progressMessage = msg || '';
          this.emitProgress(execution.id, adjustedPct, msg || '');
        },
      }) as {
        result: string;
        iterations: number;
        actions: any[];
        status: string;
        tokens_used: number;
        security_alerts?: string[];
      };

      execution.result = result.result;
      execution.iterations = result.iterations;
      execution.actions = result.actions;
      execution.tokensUsed = result.tokens_used;
      execution.securityAlerts = result.security_alerts;
      execution.status = result.status === 'blocked' ? 'blocked' :
                         result.status === 'completed' ? 'completed' :
                         result.status === 'error' ? 'failed' : 'completed';
      execution.completedAt = new Date();
      execution.progress = 100;
      execution.progressMessage = 'Complete';

      // Sync sandbox to IPFS after completion if we have a sandbox node
      if (execution.sandboxNodeId && this.nodeManager && execution.status === 'completed') {
        try {
          const syncResult = await this.nodeManager.sandboxSyncToIPFS(
            execution.sandboxNodeId,
            execution.workspaceId
          );
          if (syncResult.success && syncResult.cid) {
            execution.sandboxCid = syncResult.cid;
            console.log(`[AgentService] Sandbox synced to IPFS: ${syncResult.cid}`);
          }
        } catch (syncError) {
          console.warn(`[AgentService] Failed to sync sandbox to IPFS: ${syncError}`);
        }
      }

      const computeInfo = execution.computeSource === 'local'
        ? `Local node ${execution.nodeId?.slice(0, 8) || 'unknown'}`
        : `Cloud (${execution.provider})`;
      console.log(`[AgentService] Agent completed: ${execution.tokensUsed} tokens, ${execution.iterations} iterations, compute: ${computeInfo}`);

      this.emitProgress(execution.id, 100, 'Complete', { final: true, result: execution });

    } catch (error) {
      execution.status = 'failed';
      execution.error = error instanceof Error ? error.message : 'Unknown error';
      execution.completedAt = new Date();
      this.emitProgress(execution.id, 100, `Error: ${execution.error}`);
    }
  }

  /**
   * Emit progress update
   */
  private emitProgress(agentId: string, progress: number, message: string, action?: any): void {
    if (this.onProgress) {
      this.onProgress(agentId, progress, message, action);
    }
  }

  /**
   * Get agent execution by ID
   */
  getExecution(executionId: string): AgentExecution | undefined {
    return this.executions.get(executionId);
  }

  /**
   * Get all executions for a workspace
   */
  getWorkspaceExecutions(workspaceId: string): AgentExecution[] {
    return Array.from(this.executions.values())
      .filter(e => e.workspaceId === workspaceId)
      .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Get running executions for a workspace
   */
  getRunningExecutions(workspaceId: string): AgentExecution[] {
    return this.getWorkspaceExecutions(workspaceId)
      .filter(e => e.status === 'running' || e.status === 'pending' || e.status === 'pulling_model');
  }

  /**
   * Cancel an agent execution
   */
  cancelExecution(executionId: string): boolean {
    const execution = this.executions.get(executionId);
    if (!execution || (execution.status !== 'running' && execution.status !== 'pulling_model')) {
      return false;
    }
    execution.status = 'failed';
    execution.error = 'Cancelled by user';
    execution.completedAt = new Date();
    return true;
  }

  /**
   * List available agent architectures
   */
  async listArchitectures(): Promise<Array<{ name: string; description: string }>> {
    await this.initialize();
    return this.agentAdapter.execute('list_architectures', {}, {
      job_id: 'list-arch',
      timeout_seconds: 10,
      hardware: { gpus: [], cpu_cores: 1, memory_mb: 1024 },
    }) as Promise<Array<{ name: string; description: string }>>;
  }

  /**
   * List available models (local + cloud based on workspace)
   */
  async listModels(): Promise<Array<{ name: string; provider: string }>> {
    await this.initialize();
    return this.llmAdapter.execute('list_models', {}, {
      job_id: 'list-models',
      timeout_seconds: 10,
      hardware: { gpus: [], cpu_cores: 1, memory_mb: 1024 },
    }) as Promise<Array<{ name: string; provider: string }>>;
  }

  /**
   * Get workspace compute summary
   */
  getComputeSummary(workspaceId: string): {
    hasLocalCompute: boolean;
    localNodes: number;
    localModels: string[];
    hasCloudKeys: boolean;
    cloudProviders: string[];
  } {
    const compute = this.getComputeAvailability(workspaceId);

    const localModels = new Set<string>();
    for (const node of compute.ollamaNodes) {
      for (const model of node.models) {
        localModels.add(model.name);
      }
    }

    const cloudProviders: string[] = [];
    if (compute.apiKeys.openai) cloudProviders.push('openai');
    if (compute.apiKeys.anthropic) cloudProviders.push('anthropic');

    return {
      hasLocalCompute: compute.hasLocalCompute,
      localNodes: compute.ollamaNodes.length,
      localModels: Array.from(localModels),
      hasCloudKeys: cloudProviders.length > 0,
      cloudProviders,
    };
  }
}

// Singleton instance
export const agentService = new AgentService();
