/**
 * Agent Adapter
 *
 * Autonomous AI agent execution with tool use, multi-step reasoning,
 * and security scanning. Supports multiple agent architectures.
 */

import { z } from 'zod';
import {
  AdapterInfo,
  AdapterMethod,
  ExecutionContext,
} from '../types/index.js';
import { BaseAdapter } from './base.js';
import { LlmInferenceAdapter } from './llm-inference.js';
import { SecurityScanner, RiskLevel, scanForThreats } from '../security/index.js';

// Agent architectures
type AgentArchitecture = 'react' | 'plan-execute' | 'simple';

// Tool definition
interface ToolDef {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  execute: (params: Record<string, unknown>) => Promise<string>;
}

// Agent request schema
export const AgentRunRequestSchema = z.object({
  goal: z.string().min(1),
  agent_type: z.enum(['react', 'plan-execute', 'simple']).optional().default('react'),
  provider: z.enum(['ollama', 'openai', 'anthropic', 'azure', 'bedrock']).optional(),
  model: z.string().optional(),
  api_key: z.string().optional(),
  base_url: z.string().optional(), // Ollama endpoint URL for remote nodes
  tools: z.array(z.string()).optional(),
  max_iterations: z.number().optional().default(10),
  max_tokens: z.number().optional().default(4096),
  temperature: z.number().optional().default(0.7),
  verbose: z.boolean().optional().default(false),
  security_enabled: z.boolean().optional().default(true),
});

export const AgentRunResponseSchema = z.object({
  result: z.string(),
  iterations: z.number(),
  actions: z.array(z.object({
    thought: z.string(),
    tool: z.string().optional(),
    input: z.string().optional(),
    output: z.string().optional(),
  })),
  status: z.enum(['completed', 'max_iterations', 'error', 'blocked']),
  tokens_used: z.number(),
  security_alerts: z.array(z.string()).optional(),
});

export type AgentRunRequest = z.infer<typeof AgentRunRequestSchema>;
export type AgentRunResponse = z.infer<typeof AgentRunResponseSchema>;

// Action from LLM
interface AgentAction {
  thought: string;
  tool?: string;
  input?: string;
  output?: string;
}

export class AgentAdapter extends BaseAdapter {
  readonly info: AdapterInfo = {
    name: 'agent',
    version: '0.2.0',
    description: 'Autonomous AI agent with tool use and security scanning',
    capabilities: ['agent-execution', 'multi-step-reasoning', 'tool-orchestration', 'security-scanning'],
    requirements: {
      memory: {
        min_mb: 4096,
      },
    },
  };

  readonly methods: Map<string, AdapterMethod> = new Map([
    [
      'run',
      {
        name: 'run',
        description: 'Run an autonomous agent to complete a goal',
        parameters: AgentRunRequestSchema,
        returns: AgentRunResponseSchema,
      },
    ],
    [
      'list_architectures',
      {
        name: 'list_architectures',
        description: 'List available agent architectures',
        parameters: z.object({}),
        returns: z.array(z.object({
          name: z.string(),
          description: z.string(),
        })),
      },
    ],
    [
      'list_tools',
      {
        name: 'list_tools',
        description: 'List available tools for agents',
        parameters: z.object({}),
        returns: z.array(z.object({
          name: z.string(),
          description: z.string(),
        })),
      },
    ],
  ]);

  private llmAdapter: LlmInferenceAdapter;
  private securityScanner: SecurityScanner;
  private tools: Map<string, ToolDef> = new Map();

  constructor() {
    super();
    this.llmAdapter = new LlmInferenceAdapter();
    this.securityScanner = new SecurityScanner();
    this.registerBuiltinTools();
  }

  async initialize(): Promise<void> {
    await super.initialize();
    await this.llmAdapter.initialize();
  }

  async execute(
    method: string,
    params: unknown,
    context: ExecutionContext
  ): Promise<unknown> {
    switch (method) {
      case 'run':
        return this.runAgent(params, context);
      case 'list_architectures':
        return this.listArchitectures();
      case 'list_tools':
        return this.listTools();
      default:
        throw new Error(`Unknown method: ${method}`);
    }
  }

  // ============ Agent Execution ============

  private async runAgent(
    params: unknown,
    context: ExecutionContext
  ): Promise<AgentRunResponse> {
    const request = AgentRunRequestSchema.parse(params);
    const { goal, agent_type, max_iterations, verbose, security_enabled } = request;

    console.log(`[agent] Starting ${agent_type} agent for goal: ${goal.slice(0, 50)}...`);

    // Security check on the goal itself
    if (security_enabled) {
      const goalScan = this.securityScanner.scan(goal);
      if (!goalScan.safe && (goalScan.riskLevel === RiskLevel.Critical || goalScan.riskLevel === RiskLevel.High)) {
        return {
          result: `Blocked: Security threat detected in goal - ${goalScan.summary}`,
          iterations: 0,
          actions: [],
          status: 'blocked',
          tokens_used: 0,
          security_alerts: goalScan.threats.map(t => t.pattern.description),
        };
      }
    }

    // Run appropriate agent architecture
    switch (agent_type) {
      case 'react':
        return this.runReactAgent(request, context);
      case 'plan-execute':
        return this.runPlanExecuteAgent(request, context);
      case 'simple':
        return this.runSimpleAgent(request, context);
      default:
        throw new Error(`Unknown agent type: ${agent_type}`);
    }
  }

  /**
   * ReAct Agent: Reason and Act in interleaved steps
   */
  private async runReactAgent(
    request: AgentRunRequest,
    context: ExecutionContext
  ): Promise<AgentRunResponse> {
    const { goal, max_iterations, verbose, security_enabled } = request;

    const actions: AgentAction[] = [];
    const securityAlerts: string[] = [];
    let totalTokens = 0;
    let iteration = 0;
    let finalResult = '';

    // Build tool descriptions for system prompt
    const toolDescriptions = this.buildToolDescriptions(request.tools);

    // Conversation history
    const messages: Array<{ role: 'system' | 'user' | 'assistant'; content: string }> = [
      {
        role: 'system',
        content: this.buildReactSystemPrompt(toolDescriptions),
      },
      {
        role: 'user',
        content: `Goal: ${goal}\n\nBegin working on this goal. Use the Thought/Action/Action Input/Observation format.`,
      },
    ];

    while (iteration < max_iterations) {
      iteration++;
      context.on_progress?.(
        (iteration / max_iterations) * 100,
        `Iteration ${iteration}/${max_iterations}`
      );

      // Get LLM response
      const response = await this.llmAdapter.execute('chat', {
        model: request.model || 'gpt-4o',
        provider: request.provider,
        api_key: request.api_key,
        base_url: request.base_url,
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
      }, context) as { text: string; tokens_generated: number };

      totalTokens += response.tokens_generated;

      // Parse the response
      const parsed = this.parseReactResponse(response.text);

      if (verbose) {
        console.log(`[agent] Iteration ${iteration}:`, parsed);
      }

      actions.push(parsed);

      // Check if agent wants to finish
      if (parsed.tool === 'finish' || parsed.tool === 'final_answer') {
        finalResult = parsed.input || response.text;
        break;
      }

      // Security check on action
      if (security_enabled && parsed.input) {
        const actionScan = this.securityScanner.scan(parsed.input);
        if (!actionScan.safe) {
          securityAlerts.push(...actionScan.threats.map(t => t.pattern.description));

          if (actionScan.riskLevel === RiskLevel.Critical || actionScan.riskLevel === RiskLevel.High) {
            parsed.output = `[BLOCKED] Security threat detected: ${actionScan.summary}`;
            actions[actions.length - 1] = parsed;

            // Add observation to history
            messages.push({ role: 'assistant', content: response.text });
            messages.push({
              role: 'user',
              content: `Observation: ${parsed.output}\n\nThe action was blocked for security reasons. Please try a different approach.`,
            });
            continue;
          }
        }
      }

      // Execute tool if specified
      if (parsed.tool && parsed.input) {
        const toolResult = await this.executeTool(parsed.tool, parsed.input);
        parsed.output = toolResult;
        actions[actions.length - 1] = parsed;

        // Add to conversation
        messages.push({ role: 'assistant', content: response.text });
        messages.push({ role: 'user', content: `Observation: ${toolResult}` });
      } else {
        // No tool, just add the response
        messages.push({ role: 'assistant', content: response.text });

        // Check if this looks like a final answer
        if (response.text.toLowerCase().includes('final answer') ||
            response.text.toLowerCase().includes('goal achieved') ||
            response.text.toLowerCase().includes('task complete')) {
          finalResult = response.text;
          break;
        }
      }
    }

    const status = iteration >= max_iterations ? 'max_iterations' :
                   finalResult ? 'completed' : 'error';

    return {
      result: finalResult || 'Agent did not produce a final result',
      iterations: iteration,
      actions,
      status,
      tokens_used: totalTokens,
      security_alerts: securityAlerts.length > 0 ? securityAlerts : undefined,
    };
  }

  /**
   * Plan-Execute Agent: Create a plan first, then execute steps
   */
  private async runPlanExecuteAgent(
    request: AgentRunRequest,
    context: ExecutionContext
  ): Promise<AgentRunResponse> {
    const { goal, max_iterations, verbose, security_enabled } = request;

    const actions: AgentAction[] = [];
    const securityAlerts: string[] = [];
    let totalTokens = 0;

    // Step 1: Generate plan
    const planPrompt = `Create a step-by-step plan to achieve this goal:

Goal: ${goal}

Create a numbered list of specific steps. Be concise but thorough.
Format:
1. Step one
2. Step two
...

Plan:`;

    const planResponse = await this.llmAdapter.execute('generate', {
      model: request.model || 'gpt-4o',
      provider: request.provider,
      api_key: request.api_key,
      base_url: request.base_url,
      prompt: planPrompt,
      max_tokens: 1024,
      temperature: 0.3,
    }, context) as { text: string; tokens_generated: number };

    totalTokens += planResponse.tokens_generated;

    // Parse steps from plan
    const steps = this.parsePlanSteps(planResponse.text);

    actions.push({
      thought: `Created plan with ${steps.length} steps`,
      tool: 'plan',
      input: planResponse.text,
    });

    if (verbose) {
      console.log(`[agent] Plan created with ${steps.length} steps`);
    }

    // Step 2: Execute each step
    let currentContext = '';
    let iteration = 0;

    for (const step of steps.slice(0, max_iterations)) {
      iteration++;
      context.on_progress?.(
        (iteration / Math.min(steps.length, max_iterations)) * 100,
        `Step ${iteration}/${steps.length}: ${step.slice(0, 30)}...`
      );

      // Security check
      if (security_enabled) {
        const stepScan = this.securityScanner.scan(step);
        if (!stepScan.safe && (stepScan.riskLevel === RiskLevel.Critical || stepScan.riskLevel === RiskLevel.High)) {
          securityAlerts.push(...stepScan.threats.map(t => t.pattern.description));
          actions.push({
            thought: `Step blocked: ${step}`,
            tool: 'security_block',
            output: stepScan.summary,
          });
          continue;
        }
      }

      // Execute step
      const executePrompt = `You are executing a plan step by step.

Previous context: ${currentContext || 'None'}

Current step: ${step}

Execute this step and provide the result. Be specific about what was done.

Result:`;

      const stepResponse = await this.llmAdapter.execute('generate', {
        model: request.model || 'gpt-4o',
        provider: request.provider,
        api_key: request.api_key,
        base_url: request.base_url,
        prompt: executePrompt,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
      }, context) as { text: string; tokens_generated: number };

      totalTokens += stepResponse.tokens_generated;

      actions.push({
        thought: `Executing: ${step}`,
        tool: 'execute_step',
        input: step,
        output: stepResponse.text,
      });

      currentContext += `\n- ${step}: ${stepResponse.text.slice(0, 200)}`;
    }

    // Final summary
    const summaryPrompt = `Summarize the results of completing this goal:

Goal: ${goal}

Steps completed:
${actions.filter(a => a.tool === 'execute_step').map(a => `- ${a.input}: ${a.output?.slice(0, 100)}`).join('\n')}

Provide a final answer summarizing what was accomplished:`;

    const summaryResponse = await this.llmAdapter.execute('generate', {
      model: request.model || 'gpt-4o',
      provider: request.provider,
      api_key: request.api_key,
      base_url: request.base_url,
      prompt: summaryPrompt,
      max_tokens: 1024,
      temperature: 0.3,
    }, context) as { text: string; tokens_generated: number };

    totalTokens += summaryResponse.tokens_generated;

    return {
      result: summaryResponse.text,
      iterations: iteration,
      actions,
      status: 'completed',
      tokens_used: totalTokens,
      security_alerts: securityAlerts.length > 0 ? securityAlerts : undefined,
    };
  }

  /**
   * Simple Agent: Single LLM call with tool descriptions
   */
  private async runSimpleAgent(
    request: AgentRunRequest,
    context: ExecutionContext
  ): Promise<AgentRunResponse> {
    const { goal, security_enabled } = request;

    const toolDescriptions = this.buildToolDescriptions(request.tools);

    const prompt = `You are a helpful AI assistant with access to tools.

Available tools:
${toolDescriptions}

Goal: ${goal}

Provide a direct answer to achieve this goal. If you need to use a tool, explain what you would do.

Response:`;

    const response = await this.llmAdapter.execute('generate', {
      model: request.model || 'gpt-4o',
      provider: request.provider,
      api_key: request.api_key,
      base_url: request.base_url,
      prompt,
      max_tokens: request.max_tokens,
      temperature: request.temperature,
    }, context) as { text: string; tokens_generated: number };

    const securityAlerts: string[] = [];
    if (security_enabled) {
      const scan = this.securityScanner.scan(response.text);
      if (!scan.safe) {
        securityAlerts.push(...scan.threats.map(t => t.pattern.description));
      }
    }

    return {
      result: response.text,
      iterations: 1,
      actions: [{
        thought: 'Direct response',
        output: response.text,
      }],
      status: 'completed',
      tokens_used: response.tokens_generated,
      security_alerts: securityAlerts.length > 0 ? securityAlerts : undefined,
    };
  }

  // ============ Helper Methods ============

  private buildReactSystemPrompt(toolDescriptions: string): string {
    return `You are an autonomous AI agent that reasons step by step to accomplish goals.

You have access to the following tools:
${toolDescriptions}

Use this format:

Thought: Consider what to do next
Action: the tool to use (one of the available tools, or "finish" when done)
Action Input: the input to the tool
Observation: the result (this will be provided to you)

When you have completed the goal, use:
Action: finish
Action Input: your final answer

Begin!`;
  }

  private buildToolDescriptions(allowedTools?: string[]): string {
    const tools = allowedTools
      ? Array.from(this.tools.entries()).filter(([name]) => allowedTools.includes(name))
      : Array.from(this.tools.entries());

    if (tools.length === 0) {
      return '- think: reason about the problem\n- finish: provide final answer';
    }

    return tools
      .map(([name, tool]) => `- ${name}: ${tool.description}`)
      .join('\n') + '\n- finish: provide final answer';
  }

  private parseReactResponse(text: string): AgentAction {
    const thoughtMatch = text.match(/Thought:\s*(.+?)(?=\n|Action:|$)/si);
    const actionMatch = text.match(/Action:\s*(.+?)(?=\n|Action Input:|$)/si);
    const inputMatch = text.match(/Action Input:\s*(.+?)(?=\n|Observation:|$)/si);

    return {
      thought: thoughtMatch?.[1]?.trim() || text.slice(0, 200),
      tool: actionMatch?.[1]?.trim().toLowerCase(),
      input: inputMatch?.[1]?.trim(),
    };
  }

  private parsePlanSteps(planText: string): string[] {
    const lines = planText.split('\n');
    const steps: string[] = [];

    for (const line of lines) {
      const match = line.match(/^\d+\.\s*(.+)/);
      if (match) {
        steps.push(match[1].trim());
      }
    }

    return steps.length > 0 ? steps : [planText];
  }

  private async executeTool(toolName: string, input: string): Promise<string> {
    const tool = this.tools.get(toolName.toLowerCase());

    if (!tool) {
      return `Tool '${toolName}' not found. Available tools: ${Array.from(this.tools.keys()).join(', ')}`;
    }

    try {
      return await tool.execute({ input });
    } catch (error) {
      return `Error executing ${toolName}: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  // ============ Tool Management ============

  private registerBuiltinTools(): void {
    // Think tool (always available)
    this.tools.set('think', {
      name: 'think',
      description: 'Reason about the problem without taking action',
      parameters: { input: 'string' },
      execute: async (params) => `Thought recorded: ${params.input}`,
    });

    // Search tool (simulated)
    this.tools.set('search', {
      name: 'search',
      description: 'Search for information on the web',
      parameters: { query: 'string' },
      execute: async (params) => `Search results for "${params.input}": [Simulated search - integrate real search API]`,
    });

    // Calculate tool
    this.tools.set('calculate', {
      name: 'calculate',
      description: 'Perform mathematical calculations',
      parameters: { expression: 'string' },
      execute: async (params) => {
        try {
          // Simple eval for math (in production, use a proper math parser)
          const expr = String(params.input).replace(/[^0-9+\-*/().%\s]/g, '');
          const result = Function(`"use strict"; return (${expr})`)();
          return `Result: ${result}`;
        } catch {
          return `Error: Could not evaluate expression`;
        }
      },
    });
  }

  /**
   * Register a custom tool
   */
  registerTool(tool: ToolDef): void {
    this.tools.set(tool.name.toLowerCase(), tool);
  }

  // ============ Info Methods ============

  private listArchitectures(): Array<{ name: string; description: string }> {
    return [
      {
        name: 'react',
        description: 'ReAct agent - Reason and Act in interleaved steps. Best for complex multi-step tasks.',
      },
      {
        name: 'plan-execute',
        description: 'Plan then execute - Creates a complete plan first, then executes each step. Best for well-defined goals.',
      },
      {
        name: 'simple',
        description: 'Simple single-turn agent - One LLM call with tool awareness. Best for quick tasks.',
      },
    ];
  }

  private listTools(): Array<{ name: string; description: string }> {
    return Array.from(this.tools.entries()).map(([name, tool]) => ({
      name,
      description: tool.description,
    }));
  }
}
