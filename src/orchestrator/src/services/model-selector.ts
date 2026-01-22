/**
 * Smart Model Selector
 *
 * Analyzes tasks and selects the optimal model based on:
 * 1. Task type (coding, general, analysis, creative)
 * 2. Available workspace compute (nodes with Ollama)
 * 3. Workspace API keys (fallback to cloud)
 *
 * Automatically triggers model downloads if needed.
 */

import { ConnectedNode, OllamaModel } from '../types/index.js';

// Task categories
export type TaskCategory = 'coding' | 'general' | 'analysis' | 'creative' | 'math';

// Model recommendation
export interface ModelRecommendation {
  model: string;
  provider: 'ollama' | 'openai' | 'anthropic';
  reason: string;
  estimatedVram?: number; // MB needed
  needsPull?: boolean;
  nodeId?: string; // Which node to use
}

// Compute availability
export interface ComputeAvailability {
  hasLocalCompute: boolean;
  ollamaNodes: Array<{
    nodeId: string;
    models: OllamaModel[];
    vramMb: number;
    endpoint?: string;
  }>;
  apiKeys: {
    openai: boolean;
    anthropic: boolean;
  };
}

// Optimal models by category and size tier
const MODEL_RECOMMENDATIONS: Record<TaskCategory, {
  local: { small: string; medium: string; large: string };
  cloud: { openai: string; anthropic: string };
  keywords: string[];
}> = {
  coding: {
    local: {
      small: 'qwen2.5-coder:7b',
      medium: 'qwen2.5-coder:14b',
      large: 'qwen2.5-coder:32b',
    },
    cloud: {
      openai: 'gpt-4o',
      anthropic: 'claude-sonnet-4-5',
    },
    keywords: ['code', 'program', 'function', 'debug', 'fix', 'implement', 'python', 'javascript', 'typescript', 'rust', 'api', 'class', 'method', 'bug', 'error', 'compile', 'syntax', 'refactor'],
  },
  general: {
    local: {
      small: 'llama3.2:3b',
      medium: 'llama3.2:8b',
      large: 'llama3.1:70b',
    },
    cloud: {
      openai: 'gpt-4o-mini',
      anthropic: 'claude-haiku-4-5',
    },
    keywords: ['explain', 'what', 'how', 'why', 'help', 'tell', 'describe', 'summarize'],
  },
  analysis: {
    local: {
      small: 'llama3.2:8b',
      medium: 'qwen2.5:14b',
      large: 'qwen2.5:32b',
    },
    cloud: {
      openai: 'gpt-4o',
      anthropic: 'claude-sonnet-4-5',
    },
    keywords: ['analyze', 'review', 'evaluate', 'compare', 'assess', 'examine', 'investigate', 'research', 'study', 'data'],
  },
  creative: {
    local: {
      small: 'llama3.2:8b',
      medium: 'mistral:7b',
      large: 'llama3.1:70b',
    },
    cloud: {
      openai: 'gpt-4o',
      anthropic: 'claude-sonnet-4-5',
    },
    keywords: ['write', 'create', 'generate', 'story', 'poem', 'creative', 'imagine', 'design', 'brainstorm'],
  },
  math: {
    local: {
      small: 'qwen2.5-coder:7b',
      medium: 'qwen2.5:14b',
      large: 'qwen2.5:32b',
    },
    cloud: {
      openai: 'gpt-4o',
      anthropic: 'claude-sonnet-4-5',
    },
    keywords: ['calculate', 'math', 'equation', 'solve', 'formula', 'number', 'compute', 'algebra', 'calculus'],
  },
};

// VRAM requirements (approximate, in MB)
const MODEL_VRAM_REQUIREMENTS: Record<string, number> = {
  'llama3.2:3b': 2500,
  'llama3.2:8b': 5500,
  'llama3.1:70b': 45000,
  'qwen2.5-coder:7b': 5000,
  'qwen2.5-coder:14b': 10000,
  'qwen2.5-coder:32b': 22000,
  'qwen2.5:14b': 10000,
  'qwen2.5:32b': 22000,
  'mistral:7b': 5000,
  'deepseek-coder-v2:latest': 9000,
};

export class ModelSelector {
  /**
   * Analyze a goal/task and determine the category
   */
  categorizeTask(goal: string): TaskCategory {
    const lowerGoal = goal.toLowerCase();

    // Score each category
    const scores: Record<TaskCategory, number> = {
      coding: 0,
      general: 0,
      analysis: 0,
      creative: 0,
      math: 0,
    };

    for (const [category, config] of Object.entries(MODEL_RECOMMENDATIONS)) {
      for (const keyword of config.keywords) {
        if (lowerGoal.includes(keyword)) {
          scores[category as TaskCategory] += 1;
        }
      }
    }

    // Find highest scoring category
    let maxCategory: TaskCategory = 'general';
    let maxScore = 0;

    for (const [category, score] of Object.entries(scores)) {
      if (score > maxScore) {
        maxScore = score;
        maxCategory = category as TaskCategory;
      }
    }

    return maxCategory;
  }

  /**
   * Get compute availability from workspace nodes and API keys
   */
  getComputeAvailability(
    nodes: ConnectedNode[],
    apiKeys: Array<{ provider: string }>
  ): ComputeAvailability {
    const ollamaNodes = nodes
      .filter(n => n.capabilities.ollama?.installed)
      .map(n => ({
        nodeId: n.id,
        models: n.capabilities.ollama?.models || [],
        vramMb: n.capabilities.gpus.reduce((sum, g) => sum + g.vram_mb, 0) ||
                n.capabilities.memory.available_mb * 0.7, // Use 70% of RAM if no GPU
        endpoint: n.capabilities.ollama?.endpoint,
      }));

    return {
      hasLocalCompute: ollamaNodes.length > 0,
      ollamaNodes,
      apiKeys: {
        openai: apiKeys.some(k => k.provider === 'openai'),
        anthropic: apiKeys.some(k => k.provider === 'anthropic'),
      },
    };
  }

  /**
   * Select the best model for a task given available compute
   */
  selectModel(
    goal: string,
    compute: ComputeAvailability,
    preferLocal: boolean = true
  ): ModelRecommendation {
    const category = this.categorizeTask(goal);
    const recommendations = MODEL_RECOMMENDATIONS[category];

    // Try local compute first if available and preferred
    if (preferLocal && compute.hasLocalCompute) {
      const localRec = this.selectLocalModel(category, compute.ollamaNodes);
      if (localRec) {
        return localRec;
      }
    }

    // Fall back to cloud
    if (compute.apiKeys.anthropic) {
      return {
        model: recommendations.cloud.anthropic,
        provider: 'anthropic',
        reason: `Using Anthropic API for ${category} task (no local compute available)`,
      };
    }

    if (compute.apiKeys.openai) {
      return {
        model: recommendations.cloud.openai,
        provider: 'openai',
        reason: `Using OpenAI API for ${category} task (no local compute available)`,
      };
    }

    // No compute available - recommend smallest local model with pull
    return {
      model: recommendations.local.small,
      provider: 'ollama',
      reason: `No compute available. Need to add a node with Ollama or API keys.`,
      needsPull: true,
      estimatedVram: MODEL_VRAM_REQUIREMENTS[recommendations.local.small] || 5000,
    };
  }

  /**
   * Select the best local model based on available nodes
   */
  private selectLocalModel(
    category: TaskCategory,
    ollamaNodes: ComputeAvailability['ollamaNodes']
  ): ModelRecommendation | null {
    const recommendations = MODEL_RECOMMENDATIONS[category];
    const modelPriority = [
      recommendations.local.large,
      recommendations.local.medium,
      recommendations.local.small,
    ];

    // Check if any node has a recommended model
    for (const model of modelPriority) {
      for (const node of ollamaNodes) {
        const hasModel = node.models.some(m =>
          m.name === model || m.name.startsWith(model.split(':')[0])
        );
        if (hasModel) {
          return {
            model,
            provider: 'ollama',
            reason: `Using local ${model} on node ${node.nodeId.slice(0, 8)} for ${category} task`,
            nodeId: node.nodeId,
          };
        }
      }
    }

    // No model found, check if we can pull one
    for (const model of modelPriority) {
      const vramNeeded = MODEL_VRAM_REQUIREMENTS[model] || 5000;

      for (const node of ollamaNodes) {
        if (node.vramMb >= vramNeeded) {
          return {
            model,
            provider: 'ollama',
            reason: `Will pull ${model} to node ${node.nodeId.slice(0, 8)} (${(vramNeeded / 1024).toFixed(1)}GB needed, ${(node.vramMb / 1024).toFixed(1)}GB available)`,
            needsPull: true,
            nodeId: node.nodeId,
            estimatedVram: vramNeeded,
          };
        }
      }
    }

    return null;
  }

  /**
   * Get all models that could handle a task category
   */
  getModelsForCategory(category: TaskCategory): {
    local: string[];
    cloud: Array<{ provider: string; model: string }>;
  } {
    const rec = MODEL_RECOMMENDATIONS[category];
    return {
      local: [rec.local.small, rec.local.medium, rec.local.large],
      cloud: [
        { provider: 'openai', model: rec.cloud.openai },
        { provider: 'anthropic', model: rec.cloud.anthropic },
      ],
    };
  }

  /**
   * Estimate VRAM needed for a model
   */
  getVramRequirement(model: string): number {
    return MODEL_VRAM_REQUIREMENTS[model] || 5000;
  }

  /**
   * Check if a node can run a model
   */
  canNodeRunModel(node: ConnectedNode, model: string): boolean {
    const vramNeeded = this.getVramRequirement(model);
    const nodeVram = node.capabilities.gpus.reduce((sum, g) => sum + g.vram_mb, 0) ||
                     node.capabilities.memory.available_mb * 0.7;
    return nodeVram >= vramNeeded;
  }
}

// Singleton instance
export const modelSelector = new ModelSelector();
