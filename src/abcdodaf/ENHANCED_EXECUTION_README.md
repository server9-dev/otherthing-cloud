# Enhanced Execution Engine - Quick Start

## What Was Built

A comprehensive debugging and visualization system for BPMN process execution with:

✅ **Real-time Visualization** - See active nodes as they execute
✅ **Breakpoints** - Pause execution at specific elements
✅ **Step-through Debugging** - Execute one element at a time
✅ **Instance Management** - Control multiple running processes
✅ **Event System** - Subscribe to 12 different execution events
✅ **Token Visualization** - See flow control tokens moving through process
✅ **Variable Inspector** - Watch variables change in real-time
✅ **Audit Trail** - Complete execution history
✅ **Performance Profiling** - Automatic bottleneck detection
✅ **Live UI Integration** - Three integrated UI panels

## Quick Examples

### 1. Basic Enhanced Execution

```rust
use abcdodaf::bpmn::{EnhancedRuntime, ExecutionMode, TaskHandler};
use std::sync::Arc;

// Create runtime
let runtime = EnhancedRuntime::new()
    .register_handler("user", Arc::new(MyHandler));

// Execute process
let instance = runtime.execute_process(
    &process,
    HashMap::new(),
    ExecutionMode::Continuous
).await?;

println!("Process completed: {}", instance.id);
```

### 2. Debugging with Breakpoints

```rust
// Add breakpoint
runtime.add_breakpoint("critical_task".to_string()).await;

// Subscribe to events
let mut events = runtime.subscribe_events();

// Execute
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        if let ExecutionEvent::BreakpointHit { element_id, .. } = event {
            println!("Paused at: {}", element_id);
            // Inspect state, then resume
        }
    }
});
```

### 3. Step-through Execution

```rust
// Start in step mode
let instance_id = runtime.start_process(
    &process,
    ExecutionMode::StepByStep
).await?;

// Step through each element
runtime.step(instance_id).await?;
runtime.step(instance_id).await?;
```

### 4. Performance Analysis

```rust
// Execute process
let instance = runtime.execute_process(&process, vars, mode).await?;

// Get performance data
let performance = runtime.get_performance(instance.id).await.unwrap();

for (task_id, perf) in &performance {
    println!("{}: avg {}ms", task_id, perf.avg_duration_ms);
}

// Find bottlenecks
let bottlenecks = runtime.analyze_bottlenecks(instance.id).await;
for (task_id, perf) in bottlenecks.iter().take(3) {
    println!("Slow task: {} ({}ms)", task_id, perf.avg_duration_ms);
}
```

### 5. Event Monitoring

```rust
let mut events = runtime.subscribe_events();

tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            ExecutionEvent::TaskStarted { task_id, .. } => {
                println!("Started: {}", task_id);
            }
            ExecutionEvent::TaskCompleted { task_id, duration_ms, .. } => {
                println!("Completed: {} in {}ms", task_id, duration_ms);
            }
            ExecutionEvent::TaskFailed { task_id, error, .. } => {
                eprintln!("Failed: {} - {}", task_id, error);
            }
            _ => {}
        }
    }
});
```

## Running the Demo

```bash
# Run the interactive demo
cargo run --example execution_debugger_demo --features ui
```

The demo shows:
- Real-time execution visualization
- Debugger panel with 5 tabs
- Instance management
- Live event logging
- Performance profiling

## UI Components

### Debugger Panel

```rust
use abcdodaf::ui::{DebuggerPanel, DebuggerAction};

let mut debugger = DebuggerPanel::new();
debugger.update_context(context);

let action = debugger.ui(ui);
match action {
    DebuggerAction::Pause => { /* handle pause */ }
    DebuggerAction::Resume => { /* handle resume */ }
    DebuggerAction::Step => { /* handle step */ }
    _ => {}
}
```

### Execution Visualizer

```rust
use abcdodaf::ui::ExecutionVisualizer;

let mut visualizer = ExecutionVisualizer::new();
visualizer.update_context(context);
visualizer.render(&painter, viewport);
```

### Instance Manager

```rust
use abcdodaf::ui::{InstanceManager, InstanceManagerAction};

let mut manager = InstanceManager::new();
manager.update_instances(instances);

let action = manager.ui(ui);
```

## Testing

Run the integration tests:

```bash
cargo test execution_runtime
```

Tests cover:
- Basic execution
- Breakpoint management
- Event system
- Pause/resume
- Performance tracking
- Audit trail
- Bottleneck analysis

## File Structure

```
src/
├── bpmn/
│   ├── runtime.rs           # Enhanced execution engine (758 lines)
│   └── mod.rs               # Updated exports
├── ui/
│   ├── execution_debugger.rs      # Debugger panel (546 lines)
│   ├── execution_visualizer.rs    # Visual overlay (404 lines)
│   ├── instance_manager.rs        # Instance management (305 lines)
│   └── mod.rs                      # Updated exports
examples/
└── execution_debugger_demo.rs     # Complete demo (329 lines)
tests/
└── execution_runtime_tests.rs     # Integration tests
docs/
└── EXECUTION_DEBUGGING.md         # Full documentation (580 lines)
```

## Key Features Explained

### Token-Based Flow Control

Tokens represent execution flow through the BPMN process:

```rust
pub struct ExecutionToken {
    pub id: Uuid,
    pub current_element: String,
    pub state: TokenState,  // Active, Waiting, Blocked, Consumed
    pub variables: HashMap<String, serde_json::Value>,
}
```

### Execution Events

12 event types cover all execution phases:

- **Process Lifecycle**: ProcessStarted, ProcessCompleted, ProcessFailed
- **Token Flow**: TokenCreated, TokenMoved
- **Task Execution**: TaskStarted, TaskCompleted, TaskFailed
- **State Changes**: VariableChanged, GatewayEvaluated, BreakpointHit

### Performance Metrics

Automatic tracking per task:

```rust
pub struct TaskPerformance {
    pub execution_count: u64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    // ... more fields
}
```

### Audit Trail

Complete execution history with configurable size (default: 10,000 events):

```rust
// Get all events
let audit = runtime.get_audit_trail().await;

// Get recent N events
let recent = runtime.get_recent_audit(100).await;
```

## Integration with Existing Code

The enhanced runtime is fully backward compatible:

```rust
// Old API (still works)
let executor = ProcessExecutor::new();

// New API (with debugging)
let runtime = EnhancedRuntime::new();
```

Both implement the same core execution logic but `EnhancedRuntime` adds:
- Event emission
- Breakpoint support
- Performance tracking
- Token management
- Audit trail

## Performance Considerations

The enhanced runtime adds minimal overhead:

- Events are sent via broadcast channel (async, non-blocking)
- Audit trail uses ring buffer (constant memory)
- Performance metrics are updated atomically
- Token tracking is O(n) where n = active tokens (typically < 10)

For production with high throughput:

```rust
// Use Continuous mode (no pauses)
runtime.execute_process(&process, vars, ExecutionMode::Continuous).await?;

// Don't subscribe to events if not needed
// (events are still emitted but won't block)
```

## Next Steps

1. **Read the full docs**: `docs/EXECUTION_DEBUGGING.md`
2. **Run the demo**: `cargo run --example execution_debugger_demo --features ui`
3. **Explore the tests**: `tests/execution_runtime_tests.rs`
4. **Try it in your code**: Use `EnhancedRuntime` instead of `ProcessExecutor`

## Architecture Overview

```
┌──────────────────────────────────────┐
│       Your Application               │
└─────────────┬────────────────────────┘
              │
              ▼
┌──────────────────────────────────────┐
│      EnhancedRuntime                 │
│  ┌────────────┐  ┌────────────┐     │
│  │  Events    │  │ Breakpts   │     │
│  └────────────┘  └────────────┘     │
│  ┌────────────┐  ┌────────────┐     │
│  │   Perf     │  │  Audit     │     │
│  └────────────┘  └────────────┘     │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│         UI Components                │
│  ┌──────────┐ ┌───────────┐         │
│  │Debugger  │ │Visualizer │         │
│  └──────────┘ └───────────┘         │
│  ┌──────────┐                        │
│  │ Manager  │                        │
│  └──────────┘                        │
└──────────────────────────────────────┘
```

## Support

- Full documentation: `docs/EXECUTION_DEBUGGING.md`
- Example code: `examples/execution_debugger_demo.rs`
- Tests: `tests/execution_runtime_tests.rs`
- Summary: `TASK_5_SUMMARY.md`

## License

Same as ABCDODAF project (MIT).
