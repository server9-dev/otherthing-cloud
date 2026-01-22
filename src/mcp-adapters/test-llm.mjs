/**
 * Test LLM Inference Adapter
 */

import { LlmInferenceAdapter } from './dist/index.js';

const llm = new LlmInferenceAdapter();
await llm.initialize();

const context = {
  job_id: 'llm-test',
  timeout_seconds: 120,
  hardware: { gpus: [], cpu_cores: 4, memory_mb: 8192 },
  on_progress: (pct, msg) => console.log(`[${pct.toFixed(0)}%] ${msg}`)
};

console.log('=== Testing LLM Inference Adapter ===\n');

// Test 1: List models
console.log('Test 1: List available models');
try {
  const models = await llm.execute('list_models', {}, context);
  console.log(`   Found ${models.length} models`);
  if (models.length > 0) {
    console.log(`   First 5: ${models.slice(0, 5).map(m => m.name).join(', ')}`);
  }
  console.log('   Status: PASS\n');
} catch (error) {
  console.log(`   Error: ${error.message}`);
  console.log('   Status: FAIL\n');
}

// Test 2: Generate text
console.log('Test 2: Generate text');
try {
  const result = await llm.execute('generate', {
    model: 'qwen2.5-coder:7b',
    provider: 'ollama',
    prompt: 'What is 2 + 2? Reply with just the number.',
    max_tokens: 50,
    temperature: 0.1,
  }, context);
  console.log(`   Response: ${result.text.trim()}`);
  console.log(`   Tokens: ${result.tokens_generated}`);
  console.log('   Status: PASS\n');
} catch (error) {
  console.log(`   Error: ${error.message}`);
  console.log('   Status: FAIL\n');
}

// Test 3: Chat completion
console.log('Test 3: Chat completion');
try {
  const result = await llm.execute('chat', {
    model: 'qwen2.5-coder:7b',
    provider: 'ollama',
    messages: [
      { role: 'system', content: 'You are a helpful assistant. Be brief.' },
      { role: 'user', content: 'Name 3 programming languages.' },
    ],
    max_tokens: 100,
    temperature: 0.5,
  }, context);
  console.log(`   Response: ${result.text.slice(0, 150)}...`);
  console.log(`   Tokens: ${result.tokens_generated}`);
  console.log('   Status: PASS\n');
} catch (error) {
  console.log(`   Error: ${error.message}`);
  console.log('   Status: FAIL\n');
}

// Test 4: Embeddings
console.log('Test 4: Generate embeddings');
try {
  const result = await llm.execute('embed', {
    model: 'qwen2.5-coder:7b',
    provider: 'ollama',
    text: 'Hello world',
  }, context);
  console.log(`   Embedding dimensions: ${result.embedding.length}`);
  console.log(`   First 5 values: [${result.embedding.slice(0, 5).map(v => v.toFixed(4)).join(', ')}...]`);
  console.log('   Status: PASS\n');
} catch (error) {
  console.log(`   Error: ${error.message}`);
  console.log('   Status: FAIL (embeddings may not be supported)\n');
}

console.log('=== LLM Tests Complete ===');
