/**
 * Agent Service
 *
 * Manages agent execution within workspaces using pooled resources.
 * Agents can use workspace API keys and node hardware.
 */

import { AgentAdapter, LlmInferenceAdapter, SecurityScanner, RiskLevel } from '@rhizos-cloud/mcp-adapters';
import { v4 as uuidv4 } from 'uuid';

// Agent execution request
export interface AgentRequest {
  goal: string;
  agentType?: 'react' | 'plan-execute' | 'simple';
  model?: string;
  provider?: 'ollama' | 'openai' | 'anthropic' | 'azure' | 'bedrock';
  maxIterations?: number;
  maxTokens?: number;
  temperature?: number;
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
  status: 'pending' | 'running' | 'completed' | 'failed' | 'blocked';
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

  constructor() {
    this.agentAdapter = new AgentAdapter();
    this.llmAdapter = new LlmInferenceAdapter();
    this.securityScanner = new SecurityScanner();
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;
    await this.agentAdapter.initialize();
    await this.llmAdapter.initialize();
    this.initialized = true;
    console.log('[AgentService] Initialized');
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
   * Run an agent within a workspace context
   */
  async runAgent(
    workspaceId: string,
    userId: string,
    request: AgentRequest,
    apiKey?: string
  ): Promise<AgentExecution> {
    await this.initialize();

    const executionId = uuidv4();
    const agentType = request.agentType || 'simple';
    const model = request.model || 'qwen2.5-coder:7b';
    const provider = request.provider || 'ollama';

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
      progressMessage: 'Starting agent...',
      actions: [],
      tokensUsed: 0,
      iterations: 0,
      createdAt: new Date(),
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
    this.emitProgress(execution.id, 5, 'Agent starting...');

    try {
      const result = await this.agentAdapter.execute('run', {
        goal: request.goal,
        agent_type: request.agentType || 'simple',
        model: request.model || 'qwen2.5-coder:7b',
        provider: request.provider || 'ollama',
        api_key: apiKey,
        max_iterations: request.maxIterations || 10,
        max_tokens: request.maxTokens || 4096,
        temperature: request.temperature || 0.7,
        security_enabled: true,
      }, {
        job_id: execution.id,
        timeout_seconds: 300,
        hardware: { gpus: [], cpu_cores: 4, memory_mb: 8192 },
        on_progress: (pct: number, msg?: string) => {
          execution.progress = pct;
          execution.progressMessage = msg || '';
          this.emitProgress(execution.id, pct, msg || '');
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
      .filter(e => e.status === 'running' || e.status === 'pending');
  }

  /**
   * Cancel an agent execution
   */
  cancelExecution(executionId: string): boolean {
    const execution = this.executions.get(executionId);
    if (!execution || execution.status !== 'running') {
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
   * List available models
   */
  async listModels(): Promise<Array<{ name: string; provider: string }>> {
    await this.initialize();
    return this.llmAdapter.execute('list_models', {}, {
      job_id: 'list-models',
      timeout_seconds: 10,
      hardware: { gpus: [], cpu_cores: 1, memory_mb: 1024 },
    }) as Promise<Array<{ name: string; provider: string }>>;
  }
}

// Singleton instance
export const agentService = new AgentService();
