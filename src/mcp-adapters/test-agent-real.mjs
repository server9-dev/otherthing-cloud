/**
 * Test agent on a real task
 */

import { AgentAdapter } from './dist/index.js';

const agent = new AgentAdapter();
await agent.initialize();

const context = {
  job_id: 'real-test',
  timeout_seconds: 300,
  hardware: { gpus: [], cpu_cores: 4, memory_mb: 8192 },
  on_progress: (pct, msg) => console.log(`[${pct.toFixed(0)}%] ${msg}`)
};

console.log('=== Running Agent on Real Task ===\n');
console.log('Goal: Explain how a binary search algorithm works and provide a simple Python example\n');

try {
  const result = await agent.execute('run', {
    goal: 'Explain how a binary search algorithm works, then write a working Python function that implements binary search with example usage.',
    agent_type: 'plan-execute',
    model: 'qwen2.5-coder:7b',
    provider: 'ollama',
    max_iterations: 5,
    max_tokens: 2048,
    temperature: 0.5,
    verbose: false,
    security_enabled: true,
  }, context);

  console.log('\n=== Agent Result ===');
  console.log('Status:', result.status);
  console.log('Iterations:', result.iterations);
  console.log('Tokens used:', result.tokens_used);

  console.log('\nActions taken:');
  for (const action of result.actions) {
    console.log(`  - Thought: ${action.thought?.slice(0, 100)}...`);
    if (action.tool) console.log(`    Tool: ${action.tool}`);
    if (action.input) console.log(`    Input: ${action.input?.slice(0, 50)}...`);
  }

  console.log('\n=== Final Answer ===\n');
  console.log(result.result);

  // Also show last action's full input if it was finish
  const lastAction = result.actions[result.actions.length - 1];
  if (lastAction?.tool === 'finish' && lastAction?.input) {
    console.log('\n=== Full finish input ===\n');
    console.log(lastAction.input);
  }

  if (result.security_alerts?.length) {
    console.log('\n⚠️ Security alerts:', result.security_alerts);
  }
} catch (error) {
  console.error('Error:', error.message);
  console.error(error.stack);
}
