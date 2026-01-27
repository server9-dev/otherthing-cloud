//! DoDAF 2.02 OV-6: Operational Rules, State Transitions, and Event Traces
//!
//! Implements OV-6a (Business Rules), OV-6b (State Transitions), and enhances OV-6c (Event Traces).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// OV-6a: Operational Rules Model
// ============================================================================

/// OV-6a Operational Rules Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRulesModel {
    /// Unique identifier
    pub id: String,
    /// Name of the rules model
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// Operational rules
    pub rules: Vec<OperationalRule>,
    /// Grouped rule sets
    pub rule_sets: Vec<RuleSet>,
    /// Constraints on operations
    pub constraints: Vec<Constraint>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Business rule constraining operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Type of rule
    pub rule_type: RuleType,
    /// Applicability context
    pub applicability: RuleApplicability,

    // Rule Logic
    /// Condition expression (logical statement)
    pub condition: String,
    /// Action/consequence when condition is true
    pub action: String,
    /// Alternative action if primary action fails
    pub alternative_action: Option<String>,

    // Relationships
    /// Activities this rule applies to
    pub applies_to_activities: Vec<String>,
    /// Performers this rule applies to
    pub applies_to_performers: Vec<String>,

    // Context
    /// Priority/precedence of this rule
    pub priority: i32,
    /// Whether this rule is currently enabled
    pub enabled: bool,
    /// How this rule is enforced
    pub enforcement_method: Option<String>,
}

/// Type of operational rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// Policy rule
    Policy,
    /// Regulation rule
    Regulation,
    /// Technical constraint
    Constraint,
    /// Guideline rule
    Guideline,
    /// Procedure rule
    Procedure,
    /// Business logic rule
    BusinessLogic,
}

/// Applicability scope of a rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleApplicability {
    /// Always applies
    Always,
    /// Applies conditionally
    Conditional,
    /// Applies per scenario
    PerScenario,
    /// Applies per mission phase
    PerMissionPhase,
    /// Applies per environment/context
    PerEnvironment,
}

/// Grouped set of rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    /// RuleSet ID
    pub id: String,
    /// RuleSet name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// IDs of rules in this set
    pub rules: Vec<String>,
    /// How to evaluate rules in this set
    pub evaluation_logic: EvaluationLogic,
}

/// Logic for evaluating multiple rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationLogic {
    /// All rules must pass (AND logic)
    All,
    /// At least one rule must pass (OR logic)
    Any,
    /// Rules evaluated in order, stop on first failure
    Ordered,
    /// Rules evaluated with weights
    Weighted,
}

/// Constraint on operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint ID
    pub id: String,
    /// Constraint name
    pub name: String,
    /// Type of constraint
    pub constraint_type: ConstraintType,
    /// Constraint expression
    pub expression: String,
    /// Elements this constraint applies to
    pub applies_to: Vec<String>,
}

/// Type of constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Temporal/timing constraint
    Temporal,
    /// Resource constraint
    Resource,
    /// Sequence constraint
    Sequence,
    /// Exclusion constraint (cannot occur together)
    Exclusion,
    /// Cardinality constraint
    Cardinality,
    /// Custom constraint type
    Custom(String),
}

// ============================================================================
// OV-6b: Operational State Transition Description
// ============================================================================

/// OV-6b State Transition Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionDescription {
    /// Unique identifier
    pub id: String,
    /// Name of the state machine
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// States in the state machine
    pub states: Vec<OperationalState>,
    /// Transitions between states
    pub transitions: Vec<StateTransition>,
    /// Events that trigger transitions
    pub events: Vec<TransitionEvent>,

    // State Machine Properties
    /// Initial state ID
    pub initial_state: String,
    /// Terminal state IDs
    pub terminal_states: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// State in the operational state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalState {
    /// State ID
    pub id: String,
    /// State name
    pub name: String,
    /// State description
    pub description: Option<String>,
    /// Type of state
    pub state_type: StateType,

    // State Behavior
    /// Action executed on entering state
    pub entry_action: Option<String>,
    /// Action executed on exiting state
    pub exit_action: Option<String>,
    /// Activities performed in this state
    pub activities_in_state: Vec<String>,

    // State Properties
    /// Is this the initial state?
    pub is_initial: bool,
    /// Is this a terminal state?
    pub is_terminal: bool,
    /// Is this a composite (nested) state?
    pub is_composite: bool,
    /// Parent state (for composite states)
    pub parent_state: Option<String>,
}

/// Type of operational state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    /// Idle state
    Idle,
    /// Active state
    Active,
    /// Processing state
    Processing,
    /// Waiting state
    Waiting,
    /// Blocked state
    Blocked,
    /// Error state
    Error,
    /// Terminal/final state
    Terminal,
    /// Custom state type
    Custom(String),
}

/// State transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Transition ID
    pub id: String,
    /// Transition name
    pub name: Option<String>,
    /// Source state ID
    pub source_state: String,
    /// Target state ID
    pub target_state: String,

    // Trigger and Guard
    /// Event that triggers this transition
    pub trigger_event: String,
    /// Guard condition (must be true for transition)
    pub guard_condition: Option<String>,
    /// Probability of this transition (0-1.0)
    pub probability: Option<f64>,

    // Transition Actions
    /// Action executed during transition
    pub action: Option<String>,
    /// Sequence of actions
    pub action_sequence: Vec<String>,
}

/// Event that triggers state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionEvent {
    /// Event ID
    pub id: String,
    /// Event name
    pub name: String,
    /// Event description
    pub description: Option<String>,
    /// Type of event
    pub event_type: EventType,
    /// IDs of transitions triggered by this event
    pub triggers_transitions: Vec<String>,
}

/// Type of event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// External event (from outside the system)
    External,
    /// Internal event (generated within system)
    Internal,
    /// Temporal event (time-based)
    Temporal,
    /// Signal event
    Signal,
    /// Condition-based event
    Condition,
    /// Error event
    Error,
    /// Completion event
    Completion,
}

// ============================================================================
// OV-6c Enhancement
// ============================================================================

/// Event relationship type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventRelationshipType {
    /// Events occur in sequence
    Sequence,
    /// One event causes another
    Causality,
    /// Events occur in parallel
    Parallel,
    /// Event occurrence is conditional
    Conditional,
    /// Events occur at same time
    Synchronous,
}

/// Relationship between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRelationship {
    /// Source event ID
    pub source_event_id: String,
    /// Target event ID
    pub target_event_id: String,
    /// Type of relationship
    pub relationship_type: EventRelationshipType,
}

/// Object being traced in event trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceObject {
    /// Object ID
    pub id: String,
    /// Object name
    pub name: String,
    /// Type of object
    pub object_type: String,
    /// Event that created this object
    pub creation_event: String,
    /// Event that terminates/consumes this object
    pub terminal_event: Option<String>,
    /// Object properties
    pub properties: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl OperationalRulesModel {
    /// Create a new operational rules model
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            rules: Vec::new(),
            rule_sets: Vec::new(),
            constraints: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a rule
    pub fn add_rule(mut self, rule: OperationalRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add a rule set
    pub fn add_rule_set(mut self, rule_set: RuleSet) -> Self {
        self.rule_sets.push(rule_set);
        self
    }

    /// Add a constraint
    pub fn add_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
}

impl OperationalRule {
    /// Create a new operational rule
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        rule_type: RuleType,
        condition: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            rule_type,
            applicability: RuleApplicability::Always,
            condition: condition.into(),
            action: action.into(),
            alternative_action: None,
            applies_to_activities: Vec::new(),
            applies_to_performers: Vec::new(),
            priority: 1,
            enabled: true,
            enforcement_method: None,
        }
    }

    /// Set applicability
    pub fn with_applicability(mut self, applicability: RuleApplicability) -> Self {
        self.applicability = applicability;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add applicable activity
    pub fn add_applicable_activity(mut self, activity_id: impl Into<String>) -> Self {
        self.applies_to_activities.push(activity_id.into());
        self
    }

    /// Add applicable performer
    pub fn add_applicable_performer(mut self, performer_id: impl Into<String>) -> Self {
        self.applies_to_performers.push(performer_id.into());
        self
    }
}

impl StateTransitionDescription {
    /// Create a new state transition description
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        initial_state: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            states: Vec::new(),
            transitions: Vec::new(),
            events: Vec::new(),
            initial_state: initial_state.into(),
            terminal_states: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a state
    pub fn add_state(mut self, state: OperationalState) -> Self {
        self.states.push(state);
        self
    }

    /// Add a transition
    pub fn add_transition(mut self, transition: StateTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Add an event
    pub fn add_event(mut self, event: TransitionEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Add a terminal state
    pub fn add_terminal_state(mut self, state_id: impl Into<String>) -> Self {
        self.terminal_states.push(state_id.into());
        self
    }
}

impl OperationalState {
    /// Create a new operational state
    pub fn new(id: impl Into<String>, name: impl Into<String>, state_type: StateType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            state_type,
            entry_action: None,
            exit_action: None,
            activities_in_state: Vec::new(),
            is_initial: false,
            is_terminal: false,
            is_composite: false,
            parent_state: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set as initial state
    pub fn as_initial(mut self) -> Self {
        self.is_initial = true;
        self
    }

    /// Set as terminal state
    pub fn as_terminal(mut self) -> Self {
        self.is_terminal = true;
        self
    }

    /// Add an activity
    pub fn add_activity(mut self, activity_id: impl Into<String>) -> Self {
        self.activities_in_state.push(activity_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rules_model() {
        let model = OperationalRulesModel::new("ov6a_1", "Operational Rules")
            .with_description("Rules constraining operations");

        assert_eq!(model.name, "Operational Rules");
        assert_eq!(model.rules.len(), 0);
    }

    #[test]
    fn test_operational_rule() {
        let rule = OperationalRule::new(
            "rule1",
            "Command Authority",
            "All orders must come from authorized command",
            RuleType::Policy,
            "order.source == AUTHORIZED_COMMAND",
            "execute_order(order)",
        )
        .with_priority(1)
        .add_applicable_activity("activity1");

        assert_eq!(rule.rule_type, RuleType::Policy);
        assert_eq!(rule.priority, 1);
        assert_eq!(rule.applies_to_activities.len(), 1);
    }

    #[test]
    fn test_state_transition_model() {
        let idle = OperationalState::new("state_idle", "Idle", StateType::Idle).as_initial();
        let active = OperationalState::new("state_active", "Active", StateType::Active);

        let event = TransitionEvent {
            id: "ev_start".to_string(),
            name: "Start Signal".to_string(),
            description: None,
            event_type: EventType::Signal,
            triggers_transitions: vec!["trans_1".to_string()],
        };

        let transition = StateTransition {
            id: "trans_1".to_string(),
            name: Some("Start Transition".to_string()),
            source_state: "state_idle".to_string(),
            target_state: "state_active".to_string(),
            trigger_event: "ev_start".to_string(),
            guard_condition: None,
            probability: Some(1.0),
            action: Some("activate_system()".to_string()),
            action_sequence: Vec::new(),
        };

        let model = StateTransitionDescription::new("ov6b_1", "State Machine", "state_idle")
            .add_state(idle)
            .add_state(active)
            .add_event(event)
            .add_transition(transition)
            .add_terminal_state("state_done");

        assert_eq!(model.states.len(), 2);
        assert_eq!(model.transitions.len(), 1);
        assert_eq!(model.events.len(), 1);
    }

    #[test]
    fn test_constraint_types() {
        assert_ne!(ConstraintType::Temporal, ConstraintType::Resource);
        assert_eq!(ConstraintType::Sequence, ConstraintType::Sequence);
    }

    #[test]
    fn test_event_relationships() {
        assert_eq!(
            EventRelationshipType::Sequence,
            EventRelationshipType::Sequence
        );
        assert_ne!(EventRelationshipType::Causality, EventRelationshipType::Parallel);
    }
}
