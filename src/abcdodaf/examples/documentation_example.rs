//! Example demonstrating auto-documentation generation and template library
//!
//! This example shows how to:
//! 1. Generate documentation from BPMN processes
//! 2. Export documentation in multiple formats (Markdown, HTML, PlainText)
//! 3. Use pre-built workflow templates
//! 4. Create and search the template library
//! 5. Generate SVG diagrams

use abcdodaf::bpmn::ProcessBuilder;
use abcdodaf::documentation::{
    create_builtin_library, DiagramFormat, DiagramGenerator, DocumentationGenerator, Exporter,
    HtmlExporter, MarkdownExporter, PlainTextExporter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ABCDODAF Documentation and Template System Demo ===\n");

    // Part 1: Documentation Generation
    println!("1. DOCUMENTATION GENERATION");
    println!("{}", "-".repeat(50));
    demonstrate_documentation_generation()?;

    println!("\n");

    // Part 2: Template Library
    println!("2. TEMPLATE LIBRARY");
    println!("{}", "-".repeat(50));
    demonstrate_template_library()?;

    println!("\n");

    // Part 3: Diagram Generation
    println!("3. DIAGRAM GENERATION");
    println!("{}", "-".repeat(50));
    demonstrate_diagram_generation()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn demonstrate_documentation_generation() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample process
    let process = ProcessBuilder::new("approval_process", "Purchase Order Approval")
        .description("A workflow for approving purchase orders")
        .build()?;

    // Create documentation generator with default config
    let generator = DocumentationGenerator::default();

    // Generate documentation
    let doc = generator.generate_from_process(&process)?;

    println!("Generated documentation for: {}", doc.name);
    println!("Process ID: {}", doc.process_id);
    println!("Generated at: {}", doc.generated_at);

    // Export to Markdown
    println!("\nExporting to Markdown...");
    let markdown_exporter = MarkdownExporter;
    let md_content = markdown_exporter.export(&doc)?;
    println!("Markdown export (first 200 chars): {}", &md_content[..200.min(md_content.len())]);

    // Export to HTML
    println!("\nExporting to HTML...");
    let html_exporter = HtmlExporter::new();
    let html_content = html_exporter.export(&doc)?;
    println!(
        "HTML export (contains <!DOCTYPE html>): {}",
        html_content.contains("<!DOCTYPE html>")
    );

    // Export to PlainText
    println!("\nExporting to PlainText...");
    let plaintext_exporter = PlainTextExporter;
    let plaintext_content = plaintext_exporter.export(&doc)?;
    println!(
        "PlainText export (first 200 chars): {}",
        &plaintext_content[..200.min(plaintext_content.len())]
    );

    // Save to files (demonstration only, will create in current directory)
    println!("\nSaving documentation files...");
    markdown_exporter
        .export_to_file(&doc, "process_doc.md")
        .unwrap_or_else(|e| println!("Note: {}", e));

    Ok(())
}

fn demonstrate_template_library() -> Result<(), Box<dyn std::error::Error>> {
    // Load the built-in template library
    let library = create_builtin_library();

    println!("Loaded library: {}", library.name);
    println!("Library version: {}", library.version);
    println!("Total templates: {}", library.templates.len());

    // Get statistics
    let stats = library.get_statistics();
    println!("\nLibrary Statistics:");
    println!("  Total templates: {}", stats.total_templates);
    println!("  Total categories: {}", stats.total_categories);
    println!("  Templates by category:");
    for (category, count) in stats.templates_by_category {
        println!("    - {}: {}", category, count);
    }

    // List all templates
    println!("\nAvailable Templates:");
    for id in library.list_ids() {
        if let Some(template) = library.get_template(&id) {
            println!(
                "  - {} ({}): {}",
                template.metadata.name, template.metadata.complexity, template.metadata.description
            );
        }
    }

    // Search for templates
    println!("\nSearching for 'approval' templates:");
    let approval_templates = library.search("approval");
    for template in approval_templates {
        println!("  - {}", template.metadata.name);
        println!("    Description: {}", template.metadata.description);
        println!("    Tags: {}", template.metadata.tags.join(", "));
    }

    // Search by complexity
    println!("\nBeginner complexity templates:");
    let beginner_templates = library.get_by_complexity("beginner");
    for template in beginner_templates {
        println!("  - {}", template.metadata.name);
    }

    // Get a specific template
    println!("\nDetailed look at 'Simple Approval Workflow' template:");
    if let Some(template) = library.get_template("approval_simple") {
        println!("  Name: {}", template.metadata.name);
        println!("  Version: {}", template.metadata.version);
        println!("  Category: {:?}", template.metadata.category);
        println!("  Use cases: {}", template.metadata.use_cases.join(", "));
        println!("  Parameters: {}", template.parameters.len());
        for (param_name, param) in &template.parameters {
            println!(
                "    - {}: {} ({})",
                param_name,
                param.param_type,
                if param.required { "required" } else { "optional" }
            );
        }
    }

    // Clone and modify a template
    println!("\nCloning a template with new ID:");
    if let Some(template) = library.get_template("approval_simple") {
        let cloned = template.clone_with_new_id();
        println!("  Original ID: {}", template.metadata.id);
        println!("  Cloned ID: {}", cloned.metadata.id);
        println!(
            "  Both have same name: {} == {}",
            template.metadata.name == cloned.metadata.name,
            template.metadata.name == cloned.metadata.name
        );
    }

    Ok(())
}

fn demonstrate_diagram_generation() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample process
    let process = ProcessBuilder::new("simple_process", "Simple Process").build()?;

    // Generate ASCII diagram
    println!("ASCII Diagram:");
    let ascii_gen = DiagramGenerator::new(DiagramFormat::Ascii);
    let ascii_diagram = ascii_gen.generate(&process)?;
    println!("{}", ascii_diagram);

    // Generate SVG diagram
    println!("\nGenerating SVG diagram...");
    let svg_gen = DiagramGenerator::new(DiagramFormat::Svg);
    let svg_content = svg_gen.generate(&process)?;
    println!("SVG diagram generated");
    println!("  Size: {} chars", svg_content.len());
    println!("  Contains SVG header: {}", svg_content.contains("<svg"));
    println!("  Contains styling: {}", svg_content.contains("<style>"));

    Ok(())
}
