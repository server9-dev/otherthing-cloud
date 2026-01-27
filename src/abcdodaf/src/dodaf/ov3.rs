//! DoDAF 2.02 OV-3: Operational Information Exchange Matrix
//!
//! Describes information exchanged between nodes/activities with attributes
//! such as media, quality, quantity, and interoperability requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OV-3 Information Exchange Matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationExchangeMatrix {
    /// Unique identifier
    pub id: String,
    /// Name of the matrix
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    // Matrix data
    /// Information elements being exchanged
    pub exchanges: Vec<InformationElement>,
    /// Exchange pairs defining source-target relationships
    pub exchange_pairs: Vec<ExchangePair>,

    // Aggregations
    /// Source nodes/activities (row headers)
    pub row_headers: Vec<String>,
    /// Target nodes/activities (column headers)
    pub column_headers: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information element - a distinct piece of information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationElement {
    /// Element ID
    pub id: String,
    /// Element name
    pub name: String,
    /// Element description
    pub description: Option<String>,
    /// Type of information
    pub information_type: InformationType,
    /// Creator of the information
    pub creator: Option<String>,
    /// Consumer(s) of the information
    pub consumer: Option<String>,
}

/// Type of information element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InformationType {
    /// Raw data
    Data,
    /// Signal or notification
    Signal,
    /// Document or report
    Document,
    /// Message or communication
    Message,
    /// Command or instruction
    Command,
    /// Report or summary
    Report,
    /// Custom information type
    Custom(String),
}

/// Exchange pair - describes a source-target information exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePair {
    /// Pair ID
    pub id: String,
    /// Source node/activity ID
    pub source: String,
    /// Target node/activity ID
    pub target: String,
    /// Information element IDs exchanged
    pub information_elements: Vec<String>,

    // OV-3 Attributes
    /// Exchange attributes and requirements
    pub exchange_attributes: ExchangeAttributes,
    /// Frequency of exchange
    pub frequency: Option<crate::dodaf::ov5::Frequency>,
    /// Criticality of this exchange
    pub criticality: Option<Criticality>,
}

/// Criticality level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Criticality {
    /// Mission-critical
    Critical,
    /// Essential
    Essential,
    /// Important
    Important,
    /// Desirable
    Desirable,
}

/// Exchange attributes - detailed requirements for information exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAttributes {
    /// When resource must be available (e.g., "Real-time", "Within 24 hours")
    pub timeliness: Option<String>,
    /// Quality characteristics
    /// Accuracy as a percentage (0-100)
    pub accuracy: Option<f64>,
    /// Completeness as a percentage (0-100)
    pub completeness: Option<f64>,
    /// Consistency as a percentage (0-100)
    pub consistency: Option<f64>,
    /// Availability requirement as a percentage (0-100)
    pub availability: Option<f64>,
    /// Security classification
    pub protective_marking: Option<crate::dodaf::ov5::SecurityClassification>,
    /// Volume or quantity of information
    pub quantity: Option<crate::dodaf::ov5::Quantity>,
    /// Transmission medium
    pub media: Option<ExchangeMedia>,
    /// Interoperability level required
    pub interoperability_level: Option<InteroperabilityLevel>,
    /// Custom attributes
    pub custom_attributes: HashMap<String, String>,
}

/// Medium of exchange
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExchangeMedia {
    /// Electronic/digital exchange
    Electronic,
    /// Voice/verbal exchange
    Voice,
    /// Physical/paper exchange
    Physical,
    /// Face-to-face/visual contact
    VisualContact,
    /// Automated system exchange
    Automated,
    /// Custom medium
    Custom(String),
}

/// Interoperability level of the exchange
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InteroperabilityLevel {
    /// Level 0 - Incompatible systems
    Level0,
    /// Level 1 - Manual/Off-line exchange
    Level1,
    /// Level 2 - Supported data elements
    Level2,
    /// Level 3 - Automated data exchange
    Level3,
    /// Level 4 - Full interoperability
    Level4,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl InformationExchangeMatrix {
    /// Create a new information exchange matrix
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            exchanges: Vec::new(),
            exchange_pairs: Vec::new(),
            row_headers: Vec::new(),
            column_headers: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add an information element
    pub fn add_information_element(mut self, element: InformationElement) -> Self {
        self.exchanges.push(element);
        self
    }

    /// Add an exchange pair
    pub fn add_exchange_pair(mut self, pair: ExchangePair) -> Self {
        // Update row and column headers
        if !self.row_headers.contains(&pair.source) {
            self.row_headers.push(pair.source.clone());
        }
        if !self.column_headers.contains(&pair.target) {
            self.column_headers.push(pair.target.clone());
        }
        self.exchange_pairs.push(pair);
        self
    }

    /// Get exchanges between source and target
    pub fn get_exchanges(&self, source: &str, target: &str) -> Vec<&ExchangePair> {
        self.exchange_pairs
            .iter()
            .filter(|p| p.source == source && p.target == target)
            .collect()
    }

    /// Get all exchanges from a source
    pub fn get_outbound_exchanges(&self, source: &str) -> Vec<&ExchangePair> {
        self.exchange_pairs.iter().filter(|p| p.source == source).collect()
    }

    /// Get all exchanges to a target
    pub fn get_inbound_exchanges(&self, target: &str) -> Vec<&ExchangePair> {
        self.exchange_pairs.iter().filter(|p| p.target == target).collect()
    }
}

impl InformationElement {
    /// Create a new information element
    pub fn new(id: impl Into<String>, name: impl Into<String>, info_type: InformationType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            information_type: info_type,
            creator: None,
            consumer: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set creator
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// Set consumer
    pub fn with_consumer(mut self, consumer: impl Into<String>) -> Self {
        self.consumer = Some(consumer.into());
        self
    }
}

impl ExchangePair {
    /// Create a new exchange pair
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            information_elements: Vec::new(),
            exchange_attributes: ExchangeAttributes::default(),
            frequency: None,
            criticality: None,
        }
    }

    /// Add an information element
    pub fn add_information_element(mut self, element_id: impl Into<String>) -> Self {
        self.information_elements.push(element_id.into());
        self
    }

    /// Set frequency
    pub fn with_frequency(mut self, freq: crate::dodaf::ov5::Frequency) -> Self {
        self.frequency = Some(freq);
        self
    }

    /// Set criticality
    pub fn with_criticality(mut self, crit: Criticality) -> Self {
        self.criticality = Some(crit);
        self
    }

    /// Set exchange attributes
    pub fn with_attributes(mut self, attrs: ExchangeAttributes) -> Self {
        self.exchange_attributes = attrs;
        self
    }
}

impl Default for ExchangeAttributes {
    fn default() -> Self {
        Self {
            timeliness: None,
            accuracy: None,
            completeness: None,
            consistency: None,
            availability: None,
            protective_marking: None,
            quantity: None,
            media: None,
            interoperability_level: None,
            custom_attributes: HashMap::new(),
        }
    }
}

impl ExchangeAttributes {
    /// Create a new exchange attributes with quality metrics
    pub fn with_quality(accuracy: f64, completeness: f64, consistency: f64) -> Self {
        Self {
            accuracy: Some(accuracy),
            completeness: Some(completeness),
            consistency: Some(consistency),
            ..Default::default()
        }
    }

    /// Set timeliness requirement
    pub fn with_timeliness(mut self, timeliness: impl Into<String>) -> Self {
        self.timeliness = Some(timeliness.into());
        self
    }

    /// Set media type
    pub fn with_media(mut self, media: ExchangeMedia) -> Self {
        self.media = Some(media);
        self
    }

    /// Set interoperability level
    pub fn with_interoperability(mut self, level: InteroperabilityLevel) -> Self {
        self.interoperability_level = Some(level);
        self
    }

    /// Add custom attribute
    pub fn add_custom_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.custom_attributes.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_exchange_matrix() {
        let matrix = InformationExchangeMatrix::new("ov3_1", "Exchange Matrix")
            .with_description("Information exchange requirements");

        assert_eq!(matrix.name, "Exchange Matrix");
        assert_eq!(matrix.exchanges.len(), 0);
        assert_eq!(matrix.exchange_pairs.len(), 0);
    }

    #[test]
    fn test_information_elements() {
        let elem = InformationElement::new("ie_1", "Tactical Order", InformationType::Command)
            .with_description("Operational command")
            .with_creator("Command Center");

        assert_eq!(elem.information_type, InformationType::Command);
        assert!(elem.creator.is_some());
    }

    #[test]
    fn test_exchange_pair_with_attributes() {
        let attrs = ExchangeAttributes::with_quality(95.0, 100.0, 99.0)
            .with_timeliness("Real-time")
            .with_media(ExchangeMedia::Electronic)
            .with_interoperability(InteroperabilityLevel::Level4);

        let pair = ExchangePair::new("ep_1", "node1", "node2")
            .add_information_element("ie_1")
            .with_attributes(attrs)
            .with_criticality(Criticality::Critical);

        assert_eq!(pair.information_elements.len(), 1);
        assert!(pair.criticality.is_some());
        assert_eq!(pair.criticality.unwrap(), Criticality::Critical);
    }

    #[test]
    fn test_matrix_queries() {
        let pair = ExchangePair::new("ep_1", "node1", "node2").add_information_element("ie_1");

        let matrix = InformationExchangeMatrix::new("ov3_1", "Test").add_exchange_pair(pair);

        assert_eq!(matrix.row_headers.len(), 1);
        assert_eq!(matrix.column_headers.len(), 1);
        assert_eq!(matrix.get_outbound_exchanges("node1").len(), 1);
        assert_eq!(matrix.get_inbound_exchanges("node2").len(), 1);
    }

    #[test]
    fn test_exchange_media_types() {
        assert_eq!(ExchangeMedia::Electronic, ExchangeMedia::Electronic);
        assert_ne!(ExchangeMedia::Voice, ExchangeMedia::Physical);
    }

    #[test]
    fn test_interoperability_levels() {
        assert!(InteroperabilityLevel::Level4 > InteroperabilityLevel::Level0);
    }
}
