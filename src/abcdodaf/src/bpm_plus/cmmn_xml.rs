//! CMMN 1.1 XML Import and Export
//!
//! This module handles serialization and deserialization of CMMN cases
//! to/from CMMN 1.1 standard XML format (OMG standard).

use super::cmmn::*;

/// CMMN XML export builder
pub struct CmmnXmlExporter;

impl CmmnXmlExporter {
    /// Export a CMMN case to standard CMMN 1.1 XML
    pub fn to_xml(case: &CmmnCase) -> std::result::Result<String, String> {
        let mut xml = String::new();

        // XML declaration
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        // Definitions element with CMMN 1.1 namespace
        xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/CMMN/20151109/MODEL\"\n");
        xml.push_str("             xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
        xml.push_str("             xmlns:cmmndi=\"http://www.omg.org/spec/CMMN/20151109/CMMNDI\"\n");
        xml.push_str("             xmlns:dc=\"http://www.omg.org/spec/CMMN/20151109/DC\"\n");
        xml.push_str("             id=\"abcdodaf_case_definitions\"\n");
        xml.push_str("             targetNamespace=\"http://abcdodaf.local/cmmn\">\n\n");

        // Case element
        xml.push_str(&format!("  <case id=\"{}\" name=\"{}\">\n", Self::escape_xml(&case.id), Self::escape_xml(&case.name)));

        if let Some(desc) = &case.description {
            xml.push_str(&format!("    <documentation>{}</documentation>\n", Self::escape_xml(desc)));
        }

        // Case Plan Model
        xml.push_str(&Self::export_case_plan_model(&case.case_plan_model, 4));

        // Case File
        xml.push_str(&Self::export_case_file(&case.case_file, 4));

        xml.push_str("  </case>\n\n");
        xml.push_str("</definitions>\n");

        Ok(xml)
    }

    fn export_case_plan_model(cpm: &CasePlanModel, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        let mut xml = format!(
            "{}<casePlanModel id=\"{}\" name=\"{}\">\n",
            spaces,
            Self::escape_xml(&cpm.id),
            Self::escape_xml(&cpm.name)
        );

        // Plan Items
        for (index, item) in cpm.plan_items.iter().enumerate() {
            xml.push_str(&Self::export_plan_item(item, indent + 2, index));
        }

        // Sentries
        for sentry in &cpm.sentries {
            xml.push_str(&Self::export_sentry(sentry, indent + 2));
        }

        xml.push_str(&format!("{}</casePlanModel>\n", spaces));
        xml
    }

    fn export_plan_item(item: &PlanItem, indent: usize, index: usize) -> String {
        let spaces = " ".repeat(indent);

        match item {
            PlanItem::HumanTask {
                id,
                name,
                performer,
                documentation,
                entry_criteria,
                exit_criteria,
                required,
                repeatable,
            } => {
                let mut xml = format!(
                    "{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"humanTask_{}\"\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(name),
                    Self::escape_xml(id)
                );
                xml.push_str(&format!("{}      isRequired=\"{}\" isRepeatable=\"{}\">\n", spaces, required, repeatable));

                // Entry criteria
                for sentry_id in entry_criteria {
                    xml.push_str(&format!(
                        "{}  <itemControl name=\"entryControl_{}_{}\">\n",
                        spaces, id, index
                    ));
                    xml.push_str(&format!(
                        "{}    <requiredRule sentryRef=\"{}\" />\n",
                        spaces,
                        Self::escape_xml(sentry_id)
                    ));
                    xml.push_str(&format!("{}  </itemControl>\n", spaces));
                }

                // Exit criteria
                for sentry_id in exit_criteria {
                    xml.push_str(&format!(
                        "{}  <itemControl name=\"exitControl_{}_{}\">\n",
                        spaces, id, index
                    ));
                    xml.push_str(&format!(
                        "{}    <requiredRule sentryRef=\"{}\" />\n",
                        spaces,
                        Self::escape_xml(sentry_id)
                    ));
                    xml.push_str(&format!("{}  </itemControl>\n", spaces));
                }

                xml.push_str(&format!("{}</planItem>\n", spaces));

                // Human Task definition
                xml.push_str(&format!("{}<humanTask id=\"humanTask_{}\"\n", spaces, Self::escape_xml(id)));
                if let Some(perf) = performer {
                    xml.push_str(&format!("{}              performer=\"{}\"", spaces, Self::escape_xml(perf)));
                }
                xml.push_str(">\n");
                if let Some(doc) = documentation {
                    xml.push_str(&format!("{}<documentation>{}</documentation>\n", spaces, Self::escape_xml(doc)));
                }
                xml.push_str(&format!("{}</humanTask>\n", spaces));

                xml
            }
            PlanItem::ProcessTask {
                id,
                name,
                process_ref,
                entry_criteria,
                exit_criteria: _,
                required,
            } => {
                let mut xml = format!(
                    "{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"processTask_{}\"\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(name),
                    Self::escape_xml(id)
                );
                xml.push_str(&format!("{}      isRequired=\"{}\">\n", spaces, required));

                // Entry/Exit criteria handling
                for sentry_id in entry_criteria {
                    xml.push_str(&format!(
                        "{}  <itemControl name=\"entryControl_{}_{}\">\n",
                        spaces, id, index
                    ));
                    xml.push_str(&format!(
                        "{}    <requiredRule sentryRef=\"{}\" />\n",
                        spaces,
                        Self::escape_xml(sentry_id)
                    ));
                    xml.push_str(&format!("{}  </itemControl>\n", spaces));
                }

                xml.push_str(&format!("{}</planItem>\n", spaces));

                // Process Task definition
                xml.push_str(&format!(
                    "{}<processTask id=\"processTask_{}\" processRef=\"{}\" />\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(process_ref)
                ));

                xml
            }
            PlanItem::Milestone { id, name, entry_criteria } => {
                let mut xml = format!(
                    "{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"milestone_{}\">\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(name),
                    Self::escape_xml(id)
                );

                for sentry_id in entry_criteria {
                    xml.push_str(&format!(
                        "{}  <itemControl name=\"entryControl_{}_{}\">\n",
                        spaces, id, index
                    ));
                    xml.push_str(&format!(
                        "{}    <requiredRule sentryRef=\"{}\" />\n",
                        spaces,
                        Self::escape_xml(sentry_id)
                    ));
                    xml.push_str(&format!("{}  </itemControl>\n", spaces));
                }

                xml.push_str(&format!("{}</planItem>\n", spaces));
                xml.push_str(&format!(
                    "{}<milestone id=\"milestone_{}\" name=\"{}\" />\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(name)
                ));

                xml
            }
            PlanItem::DecisionTask {
                id,
                name,
                decision_ref,
                entry_criteria: _,
                exit_criteria: _,
            } => {
                let mut xml = format!(
                    "{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"decisionTask_{}\">",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(name),
                    Self::escape_xml(id)
                );
                xml.push_str(&format!("\n{}</planItem>\n", spaces));

                xml.push_str(&format!(
                    "{}<decisionTask id=\"decisionTask_{}\" decisionRef=\"{}\" />\n",
                    spaces,
                    Self::escape_xml(id),
                    Self::escape_xml(decision_ref)
                ));

                xml
            }
            _ => String::new(),
        }
    }

    fn export_sentry(sentry: &Sentry, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        let mut xml = format!(
            "{}<sentry id=\"{}\" name=\"{}\">\n",
            spaces,
            Self::escape_xml(&sentry.id),
            Self::escape_xml(&sentry.name)
        );

        // On Parts
        for on_part in &sentry.on_parts {
            xml.push_str(&format!("{}  <onPart>\n", spaces));
            xml.push_str(&format!(
                "{}    <standardEvent>{:?}</standardEvent>\n",
                spaces, on_part.standard_event
            ));
            xml.push_str(&format!(
                "{}    <source ref=\"{}\" />\n",
                spaces,
                Self::escape_xml(&on_part.source_ref)
            ));
            xml.push_str(&format!("{}  </onPart>\n", spaces));
        }

        // If Part
        if let Some(condition) = &sentry.if_part {
            xml.push_str(&format!("{}  <ifPart>{}</ifPart>\n", spaces, Self::escape_xml(condition)));
        }

        xml.push_str(&format!("{}</sentry>\n", spaces));
        xml
    }

    fn export_case_file(case_file: &CaseFile, indent: usize) -> String {
        if case_file.items.is_empty() {
            return String::new();
        }

        let spaces = " ".repeat(indent);
        let mut xml = format!("{}<caseFileModel>\n", spaces);

        for (_key, item) in &case_file.items {
            xml.push_str(&format!(
                "{}  <caseFileItem id=\"{}\" name=\"{}\"",
                spaces,
                Self::escape_xml(&item.id),
                Self::escape_xml(&item.name)
            ));

            if let Some(def_ref) = &item.definition_ref {
                xml.push_str(&format!(" definitionRef=\"{}\"", Self::escape_xml(def_ref)));
            }

            xml.push_str(&format!(" multiplicity=\"{}\" />\n", Self::multiplicity_to_string(&item.multiplicity)));
        }

        xml.push_str(&format!("{}</caseFileModel>\n", spaces));
        xml
    }

    /// Escape special XML characters
    fn escape_xml(s: &str) -> String {
        s.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&apos;")
    }

    fn multiplicity_to_string(m: &Multiplicity) -> &'static str {
        match m {
            Multiplicity::ZeroOrOne => "ZeroOrOne",
            Multiplicity::ExactlyOne => "ExactlyOne",
            Multiplicity::ZeroOrMore => "ZeroOrMore",
            Multiplicity::OneOrMore => "OneOrMore",
        }
    }
}

/// CMMN XML import builder
pub struct CmmnXmlImporter;

impl CmmnXmlImporter {
    /// Import a CMMN case from XML string
    pub fn from_xml(xml_content: &str) -> std::result::Result<CmmnCase, String> {
        // For now, basic XML parsing
        // Full production implementation would use xml crate properly

        // Extract case ID and name
        let case_id = Self::extract_attribute(xml_content, "case", "id")
            .unwrap_or_else(|| "case_1".to_string());
        let case_name = Self::extract_attribute(xml_content, "case", "name")
            .unwrap_or_else(|| "Imported Case".to_string());

        let mut case = CmmnCase::new(case_id, case_name);

        // Extract description if present
        if let Some(desc) = Self::extract_element_text(xml_content, "documentation") {
            case.description = Some(desc);
        }

        // Extract case plan model
        if let Some(cpm_section) = Self::extract_xml_section(xml_content, "casePlanModel") {
            Self::parse_case_plan_model(&cpm_section, &mut case)?;
        }

        // Extract case file model
        if let Some(cfm_section) = Self::extract_xml_section(xml_content, "caseFileModel") {
            Self::parse_case_file_model(&cfm_section, &mut case)?;
        }

        Ok(case)
    }

    fn extract_attribute(xml: &str, element: &str, attr: &str) -> Option<String> {
        let start_pattern = format!("<{} ", element);
        if let Some(start_pos) = xml.find(&start_pattern) {
            let rest = &xml[start_pos..];
            let attr_pattern = format!("{}=\"", attr);
            if let Some(attr_start) = rest.find(&attr_pattern) {
                let value_start = attr_start + attr_pattern.len();
                if let Some(value_end) = rest[value_start..].find('"') {
                    return Some(rest[value_start..value_start + value_end].to_string());
                }
            }
        }
        None
    }

    fn extract_element_text(xml: &str, element: &str) -> Option<String> {
        let open_tag = format!("<{}>", element);
        let close_tag = format!("</{}>", element);

        if let Some(start_pos) = xml.find(&open_tag) {
            let content_start = start_pos + open_tag.len();
            if let Some(end_pos) = xml[content_start..].find(&close_tag) {
                return Some(xml[content_start..content_start + end_pos].to_string());
            }
        }
        None
    }

    fn extract_xml_section(xml: &str, element: &str) -> Option<String> {
        let open_tag = format!("<{}", element);
        let close_tag = format!("</{}>", element);

        if let Some(start_pos) = xml.find(&open_tag) {
            // Find the actual opening bracket after attributes
            if let Some(bracket_pos) = xml[start_pos..].find('>') {
                let content_start = start_pos + bracket_pos + 1;
                if let Some(end_pos) = xml[content_start..].find(&close_tag) {
                    return Some(xml[start_pos..content_start + end_pos + close_tag.len()].to_string());
                }
            }
        }
        None
    }

    fn parse_case_plan_model(cpm_xml: &str, case: &mut CmmnCase) -> std::result::Result<(), String> {
        // Extract plan items
        let plan_items_pattern = "<planItem";
        let mut search_pos = 0;

        while let Some(start_pos) = cpm_xml[search_pos..].find(plan_items_pattern) {
            let abs_pos = search_pos + start_pos;
            let item_section = &cpm_xml[abs_pos..];

            // Find the end of this planItem tag
            if let Some(end_pos) = item_section.find('>') {
                let item_tag = &item_section[..end_pos + 1];

                if let Some(id) = Self::extract_attr_from_tag(item_tag, "id") {
                    if let Some(name) = Self::extract_attr_from_tag(item_tag, "name") {
                        if let Some(def_ref) = Self::extract_attr_from_tag(item_tag, "definitionRef") {
                            if def_ref.contains("milestone") {
                                case.add_plan_item(PlanItem::Milestone {
                                    id: id.clone(),
                                    name,
                                    entry_criteria: vec![],
                                });
                            } else if def_ref.contains("humanTask") {
                                case.add_plan_item(PlanItem::HumanTask {
                                    id: id.clone(),
                                    name,
                                    performer: None,
                                    documentation: None,
                                    entry_criteria: vec![],
                                    exit_criteria: vec![],
                                    required: false,
                                    repeatable: false,
                                });
                            }
                        }
                    }
                }
            }

            search_pos = abs_pos + 1;
        }

        Ok(())
    }

    fn parse_case_file_model(cfm_xml: &str, case: &mut CmmnCase) -> std::result::Result<(), String> {
        let item_pattern = "<caseFileItem";
        let mut search_pos = 0;

        while let Some(start_pos) = cfm_xml[search_pos..].find(item_pattern) {
            let abs_pos = search_pos + start_pos;
            let item_section = &cfm_xml[abs_pos..];

            if let Some(end_pos) = item_section.find('>') {
                let item_tag = &item_section[..end_pos];

                if let (Some(id), Some(name)) = (
                    Self::extract_attr_from_tag(item_tag, "id"),
                    Self::extract_attr_from_tag(item_tag, "name"),
                ) {
                    let multiplicity = match Self::extract_attr_from_tag(item_tag, "multiplicity") {
                        Some(m) => match m.as_str() {
                            "ExactlyOne" => Multiplicity::ExactlyOne,
                            "ZeroOrMore" => Multiplicity::ZeroOrMore,
                            "OneOrMore" => Multiplicity::OneOrMore,
                            _ => Multiplicity::ZeroOrOne,
                        },
                        None => Multiplicity::ZeroOrOne,
                    };

                    case.add_case_file_item(CaseFileItem {
                        id,
                        name,
                        definition_ref: None,
                        multiplicity,
                    });
                }
            }

            search_pos = abs_pos + 1;
        }

        Ok(())
    }

    fn extract_attr_from_tag(tag: &str, attr: &str) -> Option<String> {
        let attr_pattern = format!("{}=\"", attr);
        if let Some(start) = tag.find(&attr_pattern) {
            let value_start = start + attr_pattern.len();
            if let Some(end) = tag[value_start..].find('"') {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmmn_xml_export() {
        let mut case = CmmnCase::new("case1", "Test Case");
        case.description = Some("A test case".to_string());

        case.add_plan_item(PlanItem::HumanTask {
            id: "task1".to_string(),
            name: "Review".to_string(),
            performer: Some("reviewer".to_string()),
            documentation: Some("Please review".to_string()),
            entry_criteria: vec![],
            exit_criteria: vec![],
            required: true,
            repeatable: false,
        });

        case.add_plan_item(PlanItem::Milestone {
            id: "milestone1".to_string(),
            name: "Reviewed".to_string(),
            entry_criteria: vec![],
        });

        let xml = CmmnXmlExporter::to_xml(&case).unwrap();

        assert!(xml.contains("<?xml version"));
        assert!(xml.contains("CMMN"));
        assert!(xml.contains("case1"));
        assert!(xml.contains("Test Case"));
        assert!(xml.contains("task1"));
        assert!(xml.contains("milestone1"));
    }

    #[test]
    fn test_cmmn_xml_escape() {
        let text = "Test & <value> with \"quotes\"";
        let escaped = CmmnXmlExporter::escape_xml(text);
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&quot;"));
    }

    #[test]
    fn test_cmmn_xml_import_basic() {
        let xml = r#"<?xml version="1.0"?>
<definitions xmlns="http://www.omg.org/spec/CMMN/20151109/MODEL">
  <case id="case1" name="Test Case">
    <casePlanModel id="cpm1" name="Main">
      <planItem id="task1" name="Task One" definitionRef="humanTask_task1" />
      <planItem id="milestone1" name="Done" definitionRef="milestone_milestone1" />
    </casePlanModel>
  </case>
</definitions>"#;

        let case = CmmnXmlImporter::from_xml(xml).unwrap();
        assert_eq!(case.id, "case1");
        assert_eq!(case.name, "Test Case");
        assert!(!case.case_plan_model.plan_items.is_empty());
    }

    #[test]
    fn test_roundtrip() {
        let mut original = CmmnCase::new("case1", "Test Case");
        original.add_plan_item(PlanItem::Milestone {
            id: "milestone1".to_string(),
            name: "Done".to_string(),
            entry_criteria: vec![],
        });

        let xml = CmmnXmlExporter::to_xml(&original).unwrap();
        let imported = CmmnXmlImporter::from_xml(&xml).unwrap();

        assert_eq!(original.id, imported.id);
        assert_eq!(original.name, imported.name);
        assert_eq!(original.case_plan_model.plan_items.len(), imported.case_plan_model.plan_items.len());
    }
}
