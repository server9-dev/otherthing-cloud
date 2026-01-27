//! DODAF Security Markings
//!
//! Security classification and handling instructions for DODAF elements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security marking for workflow elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityMarking {
    pub element_id: String,
    pub element_type: String,
    pub classification: Classification,
    pub caveats: Vec<Caveat>,
    pub handling_instructions: Vec<String>,
    pub declassification: Option<DeclassificationInfo>,
    pub metadata: HashMap<String, String>,
}

/// Security classification levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Classification {
    /// Unclassified
    Unclassified,

    /// Controlled Unclassified Information
    CUI,

    /// For Official Use Only
    FOUO,

    /// Confidential
    Confidential,

    /// Secret
    Secret,

    /// Top Secret
    TopSecret,

    /// Special Access Program
    SAP { program_name: String },
}

impl Classification {
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        match self {
            Classification::Unclassified => "UNCLASSIFIED",
            Classification::CUI => "CUI",
            Classification::FOUO => "FOUO",
            Classification::Confidential => "CONFIDENTIAL",
            Classification::Secret => "SECRET",
            Classification::TopSecret => "TOP SECRET",
            Classification::SAP { .. } => "TOP SECRET//SAP",
        }
    }

    /// Check if classification requires encryption
    pub fn requires_encryption(&self) -> bool {
        !matches!(self, Classification::Unclassified | Classification::FOUO)
    }
}

/// Security caveats and compartments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Caveat {
    /// Not Releasable to Foreign Nationals
    NOFORN,

    /// Releasable to specific countries
    REL(Vec<String>),

    /// For Eyes Only
    EYES,

    /// Originator Controlled
    ORCON,

    /// No Contractor
    NOCON,

    /// Proprietary Information
    PROPIN,

    /// Personally Identifiable Information
    PII,

    /// Special Access Required
    SAR(String),

    /// Sensitive Compartmented Information
    SCI(String),

    /// Custom caveat
    Custom(String),
}

impl Caveat {
    /// Get the marking abbreviation
    pub fn as_str(&self) -> String {
        match self {
            Caveat::NOFORN => "NOFORN".to_string(),
            Caveat::REL(countries) => format!("REL TO {}", countries.join(", ")),
            Caveat::EYES => "EYES ONLY".to_string(),
            Caveat::ORCON => "ORCON".to_string(),
            Caveat::NOCON => "NOCON".to_string(),
            Caveat::PROPIN => "PROPIN".to_string(),
            Caveat::PII => "PII".to_string(),
            Caveat::SAR(name) => format!("SAR-{}", name),
            Caveat::SCI(compartment) => format!("SCI/{}", compartment),
            Caveat::Custom(s) => s.clone(),
        }
    }
}

/// Declassification information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclassificationInfo {
    /// Declassify on date
    pub declassify_on: Option<chrono::NaiveDate>,

    /// Declassification event
    pub declassify_event: Option<String>,

    /// Exemption from automatic declassification
    pub exemption: Option<String>,

    /// Downgrade to classification
    pub downgrade_to: Option<Classification>,

    /// Downgrade on date
    pub downgrade_on: Option<chrono::NaiveDate>,
}

/// Security domain assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDomain {
    pub id: String,
    pub name: String,
    pub classification: Classification,
    pub allowed_caveats: Vec<Caveat>,
    pub encryption_required: bool,
    pub access_control_policy: String,
}

/// Security marking builder
pub struct SecurityMarkingBuilder {
    marking: SecurityMarking,
}

impl SecurityMarkingBuilder {
    /// Create a new security marking for an element
    pub fn new(element_id: impl Into<String>, element_type: impl Into<String>) -> Self {
        Self {
            marking: SecurityMarking {
                element_id: element_id.into(),
                element_type: element_type.into(),
                classification: Classification::Unclassified,
                caveats: Vec::new(),
                handling_instructions: Vec::new(),
                declassification: None,
                metadata: HashMap::new(),
            },
        }
    }

    /// Set classification level
    pub fn classification(mut self, classification: Classification) -> Self {
        self.marking.classification = classification;
        self
    }

    /// Add a caveat
    pub fn caveat(mut self, caveat: Caveat) -> Self {
        self.marking.caveats.push(caveat);
        self
    }

    /// Add handling instruction
    pub fn handling(mut self, instruction: impl Into<String>) -> Self {
        self.marking.handling_instructions.push(instruction.into());
        self
    }

    /// Set declassification info
    pub fn declassification(mut self, info: DeclassificationInfo) -> Self {
        self.marking.declassification = Some(info);
        self
    }

    /// Build the security marking
    pub fn build(self) -> SecurityMarking {
        self.marking
    }
}

impl SecurityMarking {
    /// Generate the full classification marking string
    pub fn marking_string(&self) -> String {
        let mut parts = vec![self.classification.as_str().to_string()];

        if !self.caveats.is_empty() {
            parts.push("//".to_string());
            parts.push(
                self.caveats
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
        }

        parts.join("")
    }

    /// Check if marking allows foreign release
    pub fn allows_foreign_release(&self) -> bool {
        !self.caveats.contains(&Caveat::NOFORN)
    }

    /// Check if marking requires special access
    pub fn requires_special_access(&self) -> bool {
        self.caveats.iter().any(|c| matches!(c, Caveat::SAR(_) | Caveat::SCI(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_ordering() {
        assert!(Classification::Secret > Classification::Confidential);
        assert!(Classification::TopSecret > Classification::Secret);
        assert!(Classification::Unclassified < Classification::CUI);
    }

    #[test]
    fn test_security_marking_string() {
        let marking = SecurityMarkingBuilder::new("task_1", "ServiceTask")
            .classification(Classification::Secret)
            .caveat(Caveat::NOFORN)
            .caveat(Caveat::ORCON)
            .build();

        let marking_str = marking.marking_string();
        assert_eq!(marking_str, "SECRET//NOFORN/ORCON");
    }

    #[test]
    fn test_foreign_release_check() {
        let marking1 = SecurityMarkingBuilder::new("task_1", "ServiceTask")
            .classification(Classification::Secret)
            .caveat(Caveat::NOFORN)
            .build();

        assert!(!marking1.allows_foreign_release());

        let marking2 = SecurityMarkingBuilder::new("task_2", "ServiceTask")
            .classification(Classification::Secret)
            .build();

        assert!(marking2.allows_foreign_release());
    }

    #[test]
    fn test_encryption_requirement() {
        assert!(!Classification::Unclassified.requires_encryption());
        assert!(!Classification::FOUO.requires_encryption());
        assert!(Classification::Confidential.requires_encryption());
        assert!(Classification::Secret.requires_encryption());
    }
}
