//! Documentation exporters for various formats
//!
//! Supports exporting process documentation to Markdown, HTML, and plain text formats

use super::ProcessDocumentation;

/// Trait for exporting documentation
pub trait Exporter {
    /// Export documentation to string
    fn export(&self, doc: &ProcessDocumentation) -> Result<String, String>;

    /// Export documentation to file
    fn export_to_file(&self, doc: &ProcessDocumentation, path: &str) -> Result<(), String> {
        let content = self.export(doc)?;
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write file: {}", e))
    }
}

/// Markdown exporter
pub struct MarkdownExporter;

impl Exporter for MarkdownExporter {
    fn export(&self, doc: &ProcessDocumentation) -> Result<String, String> {
        let mut output = String::new();

        // Title
        output.push_str(&format!("# {}\n\n", doc.name));

        // Description
        if !doc.description.is_empty() {
            output.push_str(&format!("## Description\n\n{}\n\n", doc.description));
        }

        // Purpose
        if let Some(ref purpose) = doc.purpose {
            output.push_str(&format!("## Purpose\n\n{}\n\n", purpose));
        }

        // Inputs
        if !doc.inputs.is_empty() {
            output.push_str("## Inputs\n\n");
            for input in &doc.inputs {
                output.push_str(&format!("- **{}** ({}): {}\n", input.name, input.param_type, input.description));
                if input.required {
                    output.push_str("  - Required: Yes\n");
                }
                if let Some(ref def) = input.default {
                    output.push_str(&format!("  - Default: {}\n", def));
                }
                if let Some(ref constraints) = input.constraints {
                    output.push_str(&format!("  - Constraints: {}\n", constraints));
                }
            }
            output.push_str("\n");
        }

        // Outputs
        if !doc.outputs.is_empty() {
            output.push_str("## Outputs\n\n");
            for output_param in &doc.outputs {
                output.push_str(&format!("- **{}** ({}): {}\n",
                    output_param.name, output_param.param_type, output_param.description));
            }
            output.push_str("\n");
        }

        // Participants
        if !doc.participants.is_empty() {
            output.push_str("## Participants\n\n");
            for participant in &doc.participants {
                output.push_str(&format!("### {}\n\n", participant.name));
                output.push_str(&format!("- Type: {}\n", participant.participant_type));
                if let Some(ref role) = participant.role {
                    output.push_str(&format!("- Role: {}\n", role));
                }
                if !participant.responsibilities.is_empty() {
                    output.push_str("- Responsibilities:\n");
                    for resp in &participant.responsibilities {
                        output.push_str(&format!("  - {}\n", resp));
                    }
                }
                output.push_str("\n");
            }
        }

        // Process Flow
        output.push_str(&format!("## Process Flow\n\n{}\n\n", doc.flow_description));

        // Error Handling
        if !doc.error_handling.is_empty() {
            output.push_str("## Error Handling\n\n");
            for error in &doc.error_handling {
                output.push_str(&format!("### {}\n\n", error.condition));
                output.push_str(&format!("**Handling:** {}\n\n", error.handling));
                if !error.recovery_steps.is_empty() {
                    output.push_str("**Recovery Steps:**\n");
                    for step in &error.recovery_steps {
                        output.push_str(&format!("1. {}\n", step));
                    }
                    output.push_str("\n");
                }
                if let Some(ref impact) = error.impact {
                    output.push_str(&format!("**Impact:** {}\n\n", impact));
                }
            }
        }

        // DoDAF Views
        if !doc.dodaf_views.is_empty() {
            output.push_str("## DoDAF Views\n\n");
            for view in &doc.dodaf_views {
                output.push_str(&format!("- {}\n", view));
            }
            output.push_str("\n");
        }

        // Metadata
        output.push_str("## Metadata\n\n");
        output.push_str(&format!("- **Process ID:** {}\n", doc.process_id));
        output.push_str(&format!("- **Generated:** {}\n", doc.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        for (key, value) in &doc.metadata {
            output.push_str(&format!("- **{}:** {}\n", key, value));
        }

        Ok(output)
    }
}

/// HTML exporter
pub struct HtmlExporter {
    /// Optional CSS stylesheet
    pub stylesheet: Option<String>,
}

impl Default for HtmlExporter {
    fn default() -> Self {
        Self {
            stylesheet: Some(Self::default_stylesheet()),
        }
    }
}

impl HtmlExporter {
    /// Create with default styling
    pub fn new() -> Self {
        Self::default()
    }

    /// Get default stylesheet
    fn default_stylesheet() -> String {
        r#"
<style>
    body {
        font-family: Arial, sans-serif;
        line-height: 1.6;
        color: #333;
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
    }
    h1 { color: #1a5490; border-bottom: 3px solid #1a5490; padding-bottom: 10px; }
    h2 { color: #2a7ab0; margin-top: 30px; }
    h3 { color: #3a8ac0; }
    .metadata { background: #f5f5f5; padding: 15px; border-left: 4px solid #1a5490; }
    .parameter, .error-scenario {
        background: #f9f9f9;
        padding: 10px;
        margin: 10px 0;
        border-left: 4px solid #2a7ab0;
    }
    .required { color: red; font-weight: bold; }
    code { background: #f4f4f4; padding: 2px 5px; border-radius: 3px; }
    table { border-collapse: collapse; width: 100%; margin: 20px 0; }
    th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
    th { background: #2a7ab0; color: white; }
    tr:nth-child(even) { background: #f9f9f9; }
</style>
"#.to_string()
    }
}

impl Exporter for HtmlExporter {
    fn export(&self, doc: &ProcessDocumentation) -> Result<String, String> {
        let mut output = String::new();

        // HTML Header
        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str(&format!("<title>{}</title>\n", doc.name));
        output.push_str("<meta charset=\"UTF-8\">\n");
        output.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

        if let Some(ref stylesheet) = self.stylesheet {
            output.push_str(stylesheet);
        }

        output.push_str("</head>\n<body>\n");

        // Title
        output.push_str(&format!("<h1>{}</h1>\n", html_escape(&doc.name)));

        // Description
        if !doc.description.is_empty() {
            output.push_str("<h2>Description</h2>\n");
            output.push_str(&format!("<p>{}</p>\n", html_escape(&doc.description)));
        }

        // Purpose
        if let Some(ref purpose) = doc.purpose {
            output.push_str("<h2>Purpose</h2>\n");
            output.push_str(&format!("<p>{}</p>\n", html_escape(purpose)));
        }

        // Inputs
        if !doc.inputs.is_empty() {
            output.push_str("<h2>Inputs</h2>\n<ul>\n");
            for input in &doc.inputs {
                output.push_str(&format!("<li class=\"parameter\">\n"));
                output.push_str(&format!("<strong>{}</strong> ({}): {}<br>\n",
                    html_escape(&input.name),
                    html_escape(&input.param_type),
                    html_escape(&input.description)));
                if input.required {
                    output.push_str("<span class=\"required\">Required</span><br>\n");
                }
                if let Some(ref def) = input.default {
                    output.push_str(&format!("Default: <code>{}</code><br>\n", html_escape(def)));
                }
                output.push_str("</li>\n");
            }
            output.push_str("</ul>\n");
        }

        // Outputs
        if !doc.outputs.is_empty() {
            output.push_str("<h2>Outputs</h2>\n<ul>\n");
            for output_param in &doc.outputs {
                output.push_str(&format!("<li>\n"));
                output.push_str(&format!("<strong>{}</strong> ({}): {}\n",
                    html_escape(&output_param.name),
                    html_escape(&output_param.param_type),
                    html_escape(&output_param.description)));
                output.push_str("</li>\n");
            }
            output.push_str("</ul>\n");
        }

        // Participants
        if !doc.participants.is_empty() {
            output.push_str("<h2>Participants</h2>\n");
            for participant in &doc.participants {
                output.push_str(&format!("<h3>{}</h3>\n", html_escape(&participant.name)));
                output.push_str(&format!("<p>Type: <strong>{}</strong></p>\n",
                    html_escape(&participant.participant_type)));
                if let Some(ref role) = participant.role {
                    output.push_str(&format!("<p>Role: <strong>{}</strong></p>\n",
                        html_escape(role)));
                }
                if !participant.responsibilities.is_empty() {
                    output.push_str("<p>Responsibilities:</p>\n<ul>\n");
                    for resp in &participant.responsibilities {
                        output.push_str(&format!("<li>{}</li>\n", html_escape(resp)));
                    }
                    output.push_str("</ul>\n");
                }
            }
        }

        // Process Flow
        output.push_str("<h2>Process Flow</h2>\n");
        output.push_str(&format!("<p>{}</p>\n", html_escape(&doc.flow_description).replace("\n", "<br>")));

        // Error Handling
        if !doc.error_handling.is_empty() {
            output.push_str("<h2>Error Handling</h2>\n");
            for error in &doc.error_handling {
                output.push_str(&format!("<div class=\"error-scenario\">\n"));
                output.push_str(&format!("<h3>{}</h3>\n", html_escape(&error.condition)));
                output.push_str(&format!("<p><strong>Handling:</strong> {}</p>\n",
                    html_escape(&error.handling)));
                if !error.recovery_steps.is_empty() {
                    output.push_str("<p><strong>Recovery Steps:</strong></p>\n<ol>\n");
                    for step in &error.recovery_steps {
                        output.push_str(&format!("<li>{}</li>\n", html_escape(step)));
                    }
                    output.push_str("</ol>\n");
                }
                output.push_str("</div>\n");
            }
        }

        // DoDAF Views
        if !doc.dodaf_views.is_empty() {
            output.push_str("<h2>DoDAF Views</h2>\n<ul>\n");
            for view in &doc.dodaf_views {
                output.push_str(&format!("<li>{}</li>\n", html_escape(view)));
            }
            output.push_str("</ul>\n");
        }

        // Metadata
        output.push_str("<div class=\"metadata\">\n");
        output.push_str("<h2>Metadata</h2>\n");
        output.push_str(&format!("<p><strong>Process ID:</strong> {}</p>\n", html_escape(&doc.process_id)));
        output.push_str(&format!("<p><strong>Generated:</strong> {}</p>\n",
            doc.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        for (key, value) in &doc.metadata {
            output.push_str(&format!("<p><strong>{}:</strong> {}</p>\n",
                html_escape(key), html_escape(value)));
        }
        output.push_str("</div>\n");

        output.push_str("</body>\n</html>\n");

        Ok(output)
    }
}

/// Plain text exporter
pub struct PlainTextExporter;

impl Exporter for PlainTextExporter {
    fn export(&self, doc: &ProcessDocumentation) -> Result<String, String> {
        let mut output = String::new();

        // Title
        output.push_str(&format!("{}\n", doc.name));
        output.push_str(&"=".repeat(doc.name.len()));
        output.push_str("\n\n");

        // Description
        if !doc.description.is_empty() {
            output.push_str("DESCRIPTION\n");
            output.push_str("-----------\n");
            output.push_str(&format!("{}\n\n", doc.description));
        }

        // Purpose
        if let Some(ref purpose) = doc.purpose {
            output.push_str("PURPOSE\n");
            output.push_str("-------\n");
            output.push_str(&format!("{}\n\n", purpose));
        }

        // Inputs
        if !doc.inputs.is_empty() {
            output.push_str("INPUTS\n");
            output.push_str("------\n");
            for input in &doc.inputs {
                output.push_str(&format!("{} ({})\n", input.name, input.param_type));
                output.push_str(&format!("  {}\n", input.description));
                if input.required {
                    output.push_str("  [REQUIRED]\n");
                }
                if let Some(ref def) = input.default {
                    output.push_str(&format!("  Default: {}\n", def));
                }
                output.push_str("\n");
            }
        }

        // Outputs
        if !doc.outputs.is_empty() {
            output.push_str("OUTPUTS\n");
            output.push_str("-------\n");
            for output_param in &doc.outputs {
                output.push_str(&format!("{} ({})\n", output_param.name, output_param.param_type));
                output.push_str(&format!("  {}\n\n", output_param.description));
            }
        }

        // Process Flow
        output.push_str("PROCESS FLOW\n");
        output.push_str("------------\n");
        output.push_str(&format!("{}\n\n", doc.flow_description));

        // Metadata
        output.push_str("METADATA\n");
        output.push_str("--------\n");
        output.push_str(&format!("Process ID: {}\n", doc.process_id));
        output.push_str(&format!("Generated: {}\n", doc.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        for (key, value) in &doc.metadata {
            output.push_str(&format!("{}: {}\n", key, value));
        }

        Ok(output)
    }
}

/// Helper function to escape HTML
fn html_escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_doc() -> ProcessDocumentation {
        ProcessDocumentation {
            process_id: "test".to_string(),
            name: "Test Process".to_string(),
            description: "A test process".to_string(),
            purpose: None,
            inputs: vec![],
            outputs: vec![],
            participants: vec![],
            flow_description: "Simple flow".to_string(),
            error_handling: vec![],
            dodaf_views: vec![],
            metadata: HashMap::new(),
            generated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_markdown_export() {
        let exporter = MarkdownExporter;
        let doc = create_sample_doc();
        let result = exporter.export(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("# Test Process"));
        assert!(content.contains("## Description"));
    }

    #[test]
    fn test_html_export() {
        let exporter = HtmlExporter::new();
        let doc = create_sample_doc();
        let result = exporter.export(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("Test Process"));
    }

    #[test]
    fn test_plain_text_export() {
        let exporter = PlainTextExporter;
        let doc = create_sample_doc();
        let result = exporter.export(&doc);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Test Process"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<test>"), "&lt;test&gt;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }
}
