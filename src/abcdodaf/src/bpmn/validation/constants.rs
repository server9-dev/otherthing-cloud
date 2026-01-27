//! Validation constants
//!
//! Constants used for validating BPMN elements against the BPMN 2.0 specification.

/// Valid BPMN node types as per BPMN 2.0 specification
pub const VALID_NODE_TYPES: &[&str] = &[
    // Events
    "startEvent",
    "endEvent",
    "intermediateCatchEvent",
    "intermediateThrowEvent",
    "boundaryEvent",
    // Tasks
    "task",
    "serviceTask",
    "userTask",
    "manualTask",
    "scriptTask",
    "businessRuleTask",
    "sendTask",
    "receiveTask",
    // Gateways
    "exclusiveGateway",
    "parallelGateway",
    "inclusiveGateway",
    "eventBasedGateway",
    "complexGateway",
    // Subprocesses
    "subProcess",
    "callActivity",
    "transaction",
    "adHocSubProcess",
];

/// Valid event types for events
pub const VALID_EVENT_TYPES: &[&str] = &[
    "none",
    "message",
    "timer",
    "error",
    "escalation",
    "cancel",
    "compensation",
    "conditional",
    "link",
    "signal",
    "terminate",
    "multiple",
    "parallelMultiple",
];

/// Valid task types for service tasks
pub const VALID_TASK_TYPES: &[&str] = &[
    "research",
    "design",
    "code_generation",
    "testing",
    "documentation",
    "integration",
    "debugging",
    "optimization",
    "deployment",
    "monitoring",
];
