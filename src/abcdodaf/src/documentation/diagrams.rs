//! SVG diagram generation from BPMN processes
//!
//! Generates visual diagrams of BPMN processes in SVG format for documentation

use crate::bpmn::process::Process;
use serde::{Deserialize, Serialize};

/// Diagram format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramFormat {
    /// SVG format
    Svg,
    /// ASCII art format
    Ascii,
}

/// SVG diagram representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgDiagram {
    /// SVG content
    pub content: String,
    /// Diagram width
    pub width: f32,
    /// Diagram height
    pub height: f32,
}

impl SvgDiagram {
    /// Create a new SVG diagram
    pub fn new(content: String, width: f32, height: f32) -> Self {
        Self { content, width, height }
    }

    /// Save SVG to file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, &self.content)
            .map_err(|e| format!("Failed to write SVG file: {}", e))
    }
}

/// Diagram generator
pub struct DiagramGenerator {
    format: DiagramFormat,
}

impl DiagramGenerator {
    /// Create a new diagram generator
    pub fn new(format: DiagramFormat) -> Self {
        Self { format }
    }

    /// Create a diagram from a process
    pub fn generate(&self, process: &Process) -> Result<String, String> {
        match self.format {
            DiagramFormat::Svg => self.generate_svg(process),
            DiagramFormat::Ascii => self.generate_ascii(process),
        }
    }

    /// Generate SVG diagram
    fn generate_svg(&self, process: &Process) -> Result<String, String> {
        let mut svg = String::new();

        // SVG header
        let width = 800;
        let height = 600;
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, width, height
        ));

        // Add stylesheet
        svg.push_str(Self::svg_styles());

        // Draw background
        svg.push_str(&format!(
            r#"<rect width="{}" height="{}" fill="white" stroke="{}" />"#,
            width, height, "#ddd"
        ));

        // Draw title
        svg.push_str(&format!(
            r#"<text x="20" y="30" class="title">{}</text>"#,
            escape_xml(&process.name)
        ));

        // Calculate positions
        let start_x = 100;
        let start_y = 100;
        let _element_spacing = 80;
        let step_spacing = 200;

        let current_y = start_y;

        // Draw start event
        svg.push_str(&Self::draw_start_event(start_x, current_y, "Start"));

        // Draw main flow with tasks
        let flow_y = start_y + 50;
        if !process.tasks.is_empty() {
            for (idx, task) in process.tasks.iter().enumerate() {
                let task_x = start_x + step_spacing + (idx as i32 * 200);
                svg.push_str(&Self::draw_flow_box(
                    task_x,
                    flow_y,
                    &task.name,
                    "Task"
                ));
            }
        } else {
            svg.push_str(&Self::draw_flow_box(
                start_x + step_spacing,
                flow_y,
                &process.name,
                "Process"
            ));
        }

        // Draw end event
        let end_x = start_x + step_spacing * 2 + if !process.tasks.is_empty() {
            (process.tasks.len() as i32 * 200)
        } else {
            0
        };
        svg.push_str(&Self::draw_end_event(end_x, current_y, "End"));

        // Draw connectors
        let from_x = start_x + 30;
        let to_x = start_x + step_spacing - 30;
        svg.push_str(&Self::draw_connector(from_x, current_y, to_x, flow_y));

        if !process.tasks.is_empty() {
            let last_task_x = start_x + step_spacing + ((process.tasks.len() - 1) as i32 * 200);
            svg.push_str(&Self::draw_connector(
                last_task_x + 30,
                flow_y,
                end_x - 30,
                current_y
            ));
        } else {
            let from_x = start_x + step_spacing + 30;
            let to_x = end_x - 30;
            svg.push_str(&Self::draw_connector(from_x, flow_y, to_x, current_y));
        }

        // Draw legend
        svg.push_str(Self::svg_legend());

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Generate ASCII diagram
    fn generate_ascii(&self, process: &Process) -> Result<String, String> {
        let mut ascii = String::new();

        ascii.push_str(&format!("{}\n", "=".repeat(60)));
        ascii.push_str(&format!("Process: {}\n", process.name));
        ascii.push_str(&format!("{}\n\n", "=".repeat(60)));

        // Start
        ascii.push_str("START\n");
        ascii.push_str("  [*]\n\n");

        // Tasks
        if !process.tasks.is_empty() {
            ascii.push_str("TASKS:\n");
            for task in &process.tasks {
                ascii.push_str(&format!("  - {} ({})\n", task.name, task.id));
            }
            ascii.push_str("\n");
        }

        // Gateways
        if !process.gateways.is_empty() {
            ascii.push_str("DECISION POINTS:\n");
            for gateway in &process.gateways {
                ascii.push_str(&format!("  - {} ({})\n", gateway.name, gateway.id));
            }
            ascii.push_str("\n");
        }

        // End
        ascii.push_str("END\n");
        ascii.push_str("  [*]\n");

        Ok(ascii)
    }

    fn svg_styles() -> &'static str {
        r#"<style>
    .title { font-size: 24px; font-weight: bold; fill: #333; }
    .event { fill: #90EE90; stroke: #333; stroke-width: 2; }
    .process-box { fill: #87CEEB; stroke: #333; stroke-width: 2; }
    .connector { stroke: #333; stroke-width: 2; fill: none; }
    .label { font-size: 12px; fill: #333; }
    .legend-title { font-size: 14px; font-weight: bold; fill: #333; }
</style>"#
    }

    fn draw_start_event(x: i32, y: i32, label: &str) -> String {
        format!(
            r#"<circle cx="{}" cy="{}" r="20" class="event"/>
            <text x="{}" y="{}" text-anchor="middle" class="label">{}</text>"#,
            x,
            y,
            x,
            y + 35,
            escape_xml(label)
        )
    }

    fn draw_end_event(x: i32, y: i32, label: &str) -> String {
        format!(
            r#"<circle cx="{}" cy="{}" r="20" class="event" stroke-width="3"/>
            <text x="{}" y="{}" text-anchor="middle" class="label">{}</text>"#,
            x,
            y,
            x,
            y + 35,
            escape_xml(label)
        )
    }

    fn draw_flow_box(x: i32, y: i32, label: &str, process_type: &str) -> String {
        format!(
            r#"<rect x="{}" y="{}" width="120" height="60" rx="5" class="process-box"/>
            <text x="{}" y="{}" text-anchor="middle" class="label">{}</text>
            <text x="{}" y="{}" text-anchor="middle" class="label" font-size="10">{}</text>"#,
            x - 60,
            y - 30,
            x,
            y - 5,
            escape_xml(label),
            x,
            y + 15,
            process_type
        )
    }

    fn draw_connector(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> String {
        let mid_x = (from_x + to_x) / 2;
        format!(
            r#"<path d="M {} {} Q {} {} {} {}" class="connector" marker-end="url(#arrowhead)"/>"#,
            from_x,
            from_y,
            mid_x,
            from_y,
            to_x,
            to_y
        )
    }

    fn svg_legend() -> &'static str {
        r#"<g id="legend">
        <text x="20" y="550" class="legend-title">Legend:</text>
        <circle cx="40" cy="570" r="8" class="event"/>
        <text x="55" y="575" class="label">Event</text>
        <rect x="125" y="562" width="40" height="25" rx="3" class="process-box"/>
        <text x="175" y="575" class="label">Process</text>
        <line x1="260" y1="570" x2="300" y2="570" class="connector"/>
        <text x="310" y="575" class="label">Flow</text>
    </g>"#
    }
}

/// Helper function to escape XML
fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::ProcessBuilder;

    #[test]
    fn test_diagram_generator_creation() {
        let gen = DiagramGenerator::new(DiagramFormat::Svg);
        assert_eq!(gen.format, DiagramFormat::Svg);
    }

    #[test]
    fn test_svg_diagram_creation() {
        let diagram = SvgDiagram::new("<svg></svg>".to_string(), 100.0, 100.0);
        assert_eq!(diagram.width, 100.0);
        assert_eq!(diagram.height, 100.0);
    }

    #[test]
    fn test_generate_ascii_diagram() {
        let process = ProcessBuilder::new("test")
            .with_name("Test Process")
            .build()
            .expect("Failed to build process");

        let gen = DiagramGenerator::new(DiagramFormat::Ascii);
        let result = gen.generate(&process);
        assert!(result.is_ok());
        let ascii = result.unwrap();
        assert!(ascii.contains("Test Process"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
    }
}
