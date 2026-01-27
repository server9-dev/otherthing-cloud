# Execution Debugging & Visualization

This document describes the enhanced execution engine with advanced debugging and visualization capabilities for ABCDODAF process execution.

## Overview

The enhanced execution engine provides comprehensive runtime debugging and visualization features that enable developers to:

- Monitor process execution in real-time
- Debug workflows with breakpoints and step-through execution
- Analyze performance bottlenecks
- Track execution history with detailed audit trails
- Visualize token flow through BPMN processes
- Inspect variables during execution
- Manage multiple process instances

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Enhanced Execution Engine                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Runtime    │  │  Debugger    │  │  Visualizer  │      │
│  │   Engine     │◄─┤   Panel      │◄─┤    Panel     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                   │             │
│         │          ┌──────────────┐           │             │
│         └─────────►│   Instance   │◄──────────┘             │
│                    │   Manager    │                         │
│                    └──────────────┘                         │
│                           │                                 │
│                    ┌──────────────┐                         │
│                    │  Audit Trail │                         │
│                    │  & Events    │                         │
│                    └──────────────┘                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Enhanced Runtime (`EnhancedRuntime`)

The core execution engine that supports advanced debugging features.

**Key Features:**
- Event-driven architecture with broadcast channels
- Breakpoint management
- Execution mode control (Continuous, StepByStep, Paused)
- Token-based flow control
- Performance profiling
- Audit trail generation

**Example Usage:**

```rust
use abcdodaf::bpmn::{EnhancedRuntime, ExecutionMode, TaskHandler};
use std::sync::Arc;

// Create runtime with handlers
let runtime = EnhancedRuntime::new()
    .register_handler("user", Arc::new(MyUserTaskHandler))
    .register_handler("service", Arc::new(MyServiceHandler));

// Subscribe to execution events
let mut event_rx = runtime.subscribe_events();

// Start process in step-by-step mode
let instance_id = runtime.start_process(&process, ExecutionMode::StepByStep).await?;

// Listen for events
tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        println!("Event: {:?}", event);
    }
});
```

### 2. Execution Events (`ExecutionEvent`)

Events emitted during process execution for monitoring and debugging.

**Event Types:**
- `ProcessStarted` - Process instance started
- `TokenCreated` - New token created
- `TokenMoved` - Token moved between elements
- `TaskStarted` - Task execution started
- `TaskCompleted` - Task completed successfully
- `TaskFailed` - Task execution failed
- `VariableChanged` - Process variable modified
- `GatewayEvaluated` - Gateway decision made
- `BreakpointHit` - Breakpoint triggered
- `ProcessCompleted` - Process finished successfully
- `ProcessFailed` - Process execution failed

### 3. Breakpoints (`Breakpoint`)

Debug breakpoints that pause execution at specific elements.

**Features:**
- Element-level breakpoints
- Conditional breakpoints (with expressions)
- Hit count tracking
- Enable/disable toggle

**Example:**

```rust
// Add simple breakpoint
runtime.add_breakpoint("task_analyze".to_string()).await;

// Add conditional breakpoint
runtime.add_conditional_breakpoint(
    "gateway_decision".to_string(),
    "total_amount > 1000".to_string()
).await;

// Toggle breakpoint
runtime.toggle_breakpoint("task_analyze").await;

// Remove breakpoint
runtime.remove_breakpoint("task_analyze").await;
```

### 4. Execution Modes (`ExecutionMode`)

Control how processes execute:

- **Continuous** - Normal execution without pauses
- **StepByStep** - Pause after each element for inspection
- **Paused** - Execution is paused (waiting for resume)

**Example:**

```rust
// Pause execution
runtime.pause(instance_id).await?;

// Resume execution
runtime.resume(instance_id).await?;

// Single step
runtime.step(instance_id).await?;

// Cancel execution
runtime.cancel(instance_id).await?;
```

### 5. Token Flow (`ExecutionToken`)

Tokens represent execution flow through the BPMN process.

**Token States:**
- `Active` - Token is moving
- `Waiting` - Token is waiting at an element
- `Blocked` - Token is blocked (breakpoint)
- `Consumed` - Token has been consumed

**Usage:**

```rust
// Get execution context with tokens
let context = runtime.get_context(instance_id).await.unwrap();

for token in &context.tokens {
    println!("Token {} at {} (state: {:?})",
        token.id, token.current_element, token.state);
}
```

### 6. Performance Profiling (`TaskPerformance`)

Track execution performance for each task.

**Metrics:**
- Execution count
- Total duration
- Average duration
- Min/Max duration
- Last execution timestamp

**Example:**

```rust
// Get performance data
let performance = runtime.get_performance(instance_id).await.unwrap();

for (task_id, perf) in &performance {
    println!("{}: avg {}ms ({} executions)",
        task_id, perf.avg_duration_ms, perf.execution_count);
}

// Analyze bottlenecks
let bottlenecks = runtime.analyze_bottlenecks(instance_id).await;
for (task_id, perf) in bottlenecks.iter().take(5) {
    println!("Bottleneck: {} ({}ms)", task_id, perf.avg_duration_ms);
}
```

### 7. Audit Trail

Complete execution history for compliance and debugging.

**Example:**

```rust
// Get full audit trail
let audit = runtime.get_audit_trail().await;

// Get recent events
let recent = runtime.get_recent_audit(100).await;

for event in recent {
    println!("{:?}", event);
}
```

## UI Components

### 1. Debugger Panel (`DebuggerPanel`)

Comprehensive debugging interface with multiple tabs.

**Tabs:**
- **Overview** - Instance status, tokens, current execution point
- **Breakpoints** - Manage breakpoints
- **Variables** - Inspect process variables
- **Events** - View execution event log
- **Performance** - Performance analysis and bottlenecks

**Example Integration:**

```rust
use abcdodaf::ui::{DebuggerPanel, DebuggerAction};

let mut debugger = DebuggerPanel::new();

// Update with context
debugger.update_context(context);

// Render UI
let action = debugger.ui(ui);

// Handle actions
match action {
    DebuggerAction::Pause => runtime.pause(instance_id).await?,
    DebuggerAction::Resume => runtime.resume(instance_id).await?,
    DebuggerAction::Step => runtime.step(instance_id).await?,
    _ => {}
}
```

### 2. Execution Visualizer (`ExecutionVisualizer`)

Visual overlay for BPMN diagrams showing execution state.

**Features:**
- Active element highlighting (pulsing animation)
- Token visualization
- Performance heatmap
- Bottleneck highlighting

**Example:**

```rust
use abcdodaf::ui::ExecutionVisualizer;

let mut visualizer = ExecutionVisualizer::new();

// Update context
visualizer.update_context(context);

// Update element positions from diagram
visualizer.update_positions(element_positions);

// Render overlay
visualizer.render(&painter, viewport);

// Update animation
visualizer.update_animation(delta_time);
```

### 3. Instance Manager (`InstanceManager`)

Manage multiple process instances.

**Features:**
- Instance list with filtering
- Instance state visualization
- Quick actions (pause, resume, cancel, debug)
- Sorting and searching
- Instance statistics

**Example:**

```rust
use abcdodaf::ui::{InstanceManager, InstanceManagerAction};

let mut manager = InstanceManager::new();

// Update instances
let instances = runtime.get_active_instances().await;
manager.update_instances(instances.into_iter().map(|(_, ctx)| ctx.instance).collect());

// Render UI
let action = manager.ui(ui);

// Handle actions
match action {
    InstanceManagerAction::Debug(id) => {
        // Switch to debugger view
    }
    InstanceManagerAction::Pause(id) => {
        runtime.pause(id).await?;
    }
    _ => {}
}
```

## Complete Example

See `examples/execution_debugger_demo.rs` for a complete working example.

```rust
// Create runtime
let runtime = EnhancedRuntime::new()
    .register_handler("user", Arc::new(UserHandler))
    .register_handler("service", Arc::new(ServiceHandler));

// Create process
let process = ProcessBuilder::new("demo", "Demo Process")
    .add_user_task("task1", "User Task")
    .add_service_task("task2", "Service Task")
    .add_flow("flow1", "task1", "task2")
    .build()?;

// Add breakpoint
runtime.add_breakpoint("task2".to_string()).await;

// Subscribe to events
let mut events = runtime.subscribe_events();

// Execute process
let instance = runtime.execute_process(
    &process,
    HashMap::new(),
    ExecutionMode::StepByStep
).await?;

// Process events
while let Ok(event) = events.recv().await {
    match event {
        ExecutionEvent::BreakpointHit { element_id, .. } => {
            println!("Hit breakpoint at {}", element_id);
            // Inspect state, then resume
            runtime.resume(instance.id).await?;
        }
        ExecutionEvent::TaskCompleted { task_id, duration_ms, .. } => {
            println!("Completed {} in {}ms", task_id, duration_ms);
        }
        _ => {}
    }
}
```

## Running the Demo

```bash
# Build and run the demo
cargo run --example execution_debugger_demo --features ui

# The demo shows:
# - Real-time execution visualization
# - Breakpoint management
# - Step-through debugging
# - Performance profiling
# - Event logging
# - Instance management
```

## Best Practices

### 1. Event Handling

```rust
// Use tokio spawn for event processing to avoid blocking
let mut event_rx = runtime.subscribe_events();
tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        // Process event
        handle_event(event).await;
    }
});
```

### 2. Performance Monitoring

```rust
// Regularly check for bottlenecks
let bottlenecks = runtime.analyze_bottlenecks(instance_id).await;
if !bottlenecks.is_empty() {
    warn!("Performance bottlenecks detected:");
    for (task_id, perf) in bottlenecks.iter().take(3) {
        warn!("  {}: {}ms average", task_id, perf.avg_duration_ms);
    }
}
```

### 3. Breakpoint Strategy

```rust
// Use conditional breakpoints for complex scenarios
runtime.add_conditional_breakpoint(
    "gateway".to_string(),
    "error_count > 0".to_string()
).await;

// Remove breakpoints after debugging
runtime.remove_breakpoint("task_id").await;
```

### 4. Audit Trail Management

```rust
// The audit trail is automatically limited to 10,000 events
// Access recent events for better performance
let recent = runtime.get_recent_audit(100).await;

// For long-running processes, consider periodic export
if instance_age > Duration::hours(24) {
    let audit = runtime.get_audit_trail().await;
    export_to_file(&audit).await?;
}
```

## Advanced Features

### Custom Event Handlers

```rust
let mut event_rx = runtime.subscribe_events();

tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        match event {
            ExecutionEvent::TaskFailed { task_id, error, .. } => {
                // Custom error handling
                send_alert(&task_id, &error).await;
            }
            ExecutionEvent::ProcessCompleted { instance_id, .. } => {
                // Custom completion logic
                notify_completion(instance_id).await;
            }
            _ => {}
        }
    }
});
```

### Performance Thresholds

```rust
// Monitor task durations
let mut event_rx = runtime.subscribe_events();

tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        if let ExecutionEvent::TaskCompleted { task_id, duration_ms, .. } = event {
            if duration_ms > 5000 {
                warn!("Task {} took {}ms (threshold: 5000ms)", task_id, duration_ms);
            }
        }
    }
});
```

### Variable Change Tracking

```rust
let mut event_rx = runtime.subscribe_events();

tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        if let ExecutionEvent::VariableChanged { variable_name, old_value, new_value, .. } = event {
            info!("Variable {} changed: {:?} -> {:?}", variable_name, old_value, new_value);
        }
    }
});
```

## Integration with Existing Systems

The enhanced execution engine is backward compatible with the existing `ProcessExecutor`. You can migrate gradually:

```rust
// Old way (still works)
let executor = ProcessExecutor::new()
    .register_handler("user", Arc::new(handler));

// New way (with debugging)
let runtime = EnhancedRuntime::new()
    .register_handler("user", Arc::new(handler));
```

## Future Enhancements

Planned features for future releases:

1. **Distributed Debugging** - Debug processes across multiple nodes
2. **Time-Travel Debugging** - Replay execution from audit trail
3. **Visual Flow Tracing** - Animated token flow on diagrams
4. **AI-Powered Analysis** - Automatic bottleneck detection and suggestions
5. **Cloud Integration** - Store audit trails in cloud storage
6. **Advanced Breakpoints** - Data breakpoints, watchpoints
7. **Live Variable Editing** - Modify variables during execution
8. **Process Mining** - Extract process models from execution logs

## Troubleshooting

### Events Not Received

```rust
// Ensure you subscribe before starting execution
let event_rx = runtime.subscribe_events();
runtime.execute_process(&process, vars, mode).await?;
```

### Breakpoints Not Hit

```rust
// Check breakpoint is enabled
let breakpoints = runtime.get_breakpoints().await;
for bp in breakpoints {
    if !bp.enabled {
        warn!("Breakpoint {} is disabled", bp.element_id);
    }
}
```

### Performance Overhead

```rust
// For production, use Continuous mode
runtime.execute_process(&process, vars, ExecutionMode::Continuous).await?;

// Disable detailed event logging if not needed
// (events are always emitted, but you don't have to subscribe)
```

## License

This code is part of the ABCDODAF project and is licensed under the same terms.
