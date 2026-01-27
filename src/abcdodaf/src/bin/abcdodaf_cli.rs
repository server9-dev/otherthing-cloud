//! ABCDODAF CLI - Command-line tool for BPMN JSON workflow validation and manipulation
//!
//! A comprehensive CLI tool for working with BPMN JSON workflow files.
//! Provides validation, conversion, information display, and auto-fixing capabilities.
//!
//! # Installation
//!
//! ```bash
//! cargo install --path . --bin abcdodaf-cli
//! ```
//!
//! # Usage Examples
//!
//! ```bash
//! # Validate a workflow file
//! abcdodaf-cli validate workflows/examples/simple_process.json
//!
//! # Get workflow information
//! abcdodaf-cli info workflows/examples/complex_workflow.json
//!
//! # Convert workflow format
//! abcdodaf-cli convert input.json output.json --format bpmn-json
//!
//! # Auto-fix common issues
//! abcdodaf-cli fix workflows/examples/broken.json --in-place
//!
//! # List and validate all workflows in a directory
//! abcdodaf-cli list workflows/examples/
//!
//! # Get JSON output for machine processing
//! abcdodaf-cli validate workflow.json --json
//! ```

use abcdodaf::bpmn::{
    validate_bpmn_json, BpmnJsonWorkflow, ErrorSeverity, ValidationError, ValidationSummary,
};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// Exit codes
const EXIT_SUCCESS: i32 = 0;
const EXIT_VALIDATION_ERROR: i32 = 1;
const EXIT_FILE_ERROR: i32 = 2;

/// ABCDODAF CLI - BPMN JSON workflow validation and editing tool
#[derive(Parser)]
#[command(name = "abcdodaf-cli")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output results in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Suppress colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a BPMN JSON workflow file
    Validate {
        /// Path to the workflow file to validate
        file: PathBuf,

        /// Show only errors (hide warnings and info)
        #[arg(short, long)]
        errors_only: bool,

        /// Exit with success even if there are warnings
        #[arg(short, long)]
        warn_as_success: bool,
    },

    /// Convert between workflow formats
    Convert {
        /// Input file path
        input: PathBuf,

        /// Output file path
        output: PathBuf,

        /// Target format
        #[arg(short, long, value_enum)]
        format: ConversionFormat,

        /// Overwrite output file if it exists
        #[arg(long)]
        force: bool,
    },

    /// Display workflow information
    Info {
        /// Path to the workflow file
        file: PathBuf,

        /// Show detailed node information
        #[arg(short, long)]
        detailed: bool,

        /// Show only statistics
        #[arg(short, long)]
        stats_only: bool,
    },

    /// Auto-fix common workflow issues
    Fix {
        /// Path to the workflow file
        file: PathBuf,

        /// Apply fixes in-place (overwrite the original file)
        #[arg(short, long)]
        in_place: bool,

        /// Output file path (if not using --in-place)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Fix only specific categories
        #[arg(short, long)]
        categories: Option<Vec<String>>,
    },

    /// List and validate all workflows in a directory
    List {
        /// Directory to scan for workflow files
        directory: PathBuf,

        /// Recursively scan subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// Show only files with errors
        #[arg(short, long)]
        errors_only: bool,

        /// Validate each file (slower but more thorough)
        #[arg(short, long)]
        validate: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ConversionFormat {
    /// BPMN JSON format
    BpmnJson,
    /// Snarl diagram format
    Snarl,
    /// Internal diagram format
    Diagram,
}

/// JSON output structure for validation results
#[derive(Serialize)]
struct ValidationOutput {
    success: bool,
    file: String,
    errors: Vec<ValidationError>,
    summary: ValidationSummaryOutput,
}

#[derive(Serialize)]
struct ValidationSummaryOutput {
    errors: usize,
    warnings: usize,
    info: usize,
}

/// JSON output structure for info command
#[derive(Serialize)]
struct InfoOutput {
    file: String,
    process: ProcessInfoOutput,
    statistics: StatisticsOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    nodes: Option<Vec<NodeInfoOutput>>,
}

#[derive(Serialize)]
struct ProcessInfoOutput {
    id: String,
    name: String,
    version: String,
    is_executable: bool,
    process_type: String,
}

#[derive(Serialize)]
struct StatisticsOutput {
    total_nodes: usize,
    total_flows: usize,
    node_types: HashMap<String, usize>,
    start_events: usize,
    end_events: usize,
    tasks: usize,
    gateways: usize,
}

#[derive(Serialize)]
struct NodeInfoOutput {
    id: String,
    name: String,
    node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_type: Option<String>,
}

/// JSON output for list command
#[derive(Serialize)]
struct ListOutput {
    directory: String,
    total_files: usize,
    valid_files: usize,
    files_with_errors: usize,
    files: Vec<FileListEntry>,
}

#[derive(Serialize)]
struct FileListEntry {
    path: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    let exit_code = match cli.command {
        Commands::Validate { file, errors_only, warn_as_success } => {
            handle_validate(&file, errors_only, warn_as_success, cli.json)
        },
        Commands::Convert { input, output, format, force } => {
            handle_convert(&input, &output, format, force, cli.json)
        },
        Commands::Info { file, detailed, stats_only } => {
            handle_info(&file, detailed, stats_only, cli.json)
        },
        Commands::Fix { file, in_place, output, categories } => {
            handle_fix(&file, in_place, output, categories, cli.json)
        },
        Commands::List { directory, recursive, errors_only, validate } => {
            handle_list(&directory, recursive, errors_only, validate, cli.json)
        },
    };

    process::exit(exit_code);
}

/// Handle the validate subcommand
fn handle_validate(
    file: &Path,
    errors_only: bool,
    warn_as_success: bool,
    json_output: bool,
) -> i32 {
    // Read the file
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to read file: {}\"}}", e);
            } else {
                eprintln!(
                    "{} Failed to read file '{}': {}",
                    "Error:".red().bold(),
                    file.display(),
                    e
                );
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Validate the workflow
    let result = validate_bpmn_json(&content);

    match result {
        Ok(()) => {
            if json_output {
                let output = ValidationOutput {
                    success: true,
                    file: file.display().to_string(),
                    errors: Vec::new(),
                    summary: ValidationSummaryOutput { errors: 0, warnings: 0, info: 0 },
                };
                match serde_json::to_string_pretty(&output) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("{{\"error\": \"Failed to serialize output: {}\"}}", e);
                        return EXIT_FILE_ERROR;
                    }
                }
            } else {
                println!("{} Workflow '{}' is valid!", "Success:".green().bold(), file.display());
            }
            EXIT_SUCCESS
        },
        Err(errors) => {
            let summary = ValidationSummary::from_errors(&errors);

            if json_output {
                let output = ValidationOutput {
                    success: false,
                    file: file.display().to_string(),
                    errors,
                    summary: ValidationSummaryOutput {
                        errors: summary.errors,
                        warnings: summary.warnings,
                        info: summary.info,
                    },
                };
                match serde_json::to_string_pretty(&output) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("{{\"error\": \"Failed to serialize output: {}\"}}", e);
                        return EXIT_FILE_ERROR;
                    }
                }
            } else {
                print_validation_errors(&errors, errors_only);
                println!("\n{}", summary.format());
            }

            // Determine exit code
            if summary.has_errors() {
                EXIT_VALIDATION_ERROR
            } else if warn_as_success {
                EXIT_SUCCESS
            } else {
                EXIT_VALIDATION_ERROR
            }
        },
    }
}

/// Print validation errors with colored output
fn print_validation_errors(errors: &[ValidationError], errors_only: bool) {
    for error in errors {
        // Skip warnings and info if errors_only is set
        if errors_only && error.severity != ErrorSeverity::Error {
            continue;
        }

        let severity_str = match error.severity {
            ErrorSeverity::Error => "ERROR".red().bold(),
            ErrorSeverity::Warning => "WARNING".yellow().bold(),
            ErrorSeverity::Info => "INFO".blue().bold(),
        };

        let category_str = format!("{:?}", error.category).dimmed();

        print!("[{}] [{}] {}", severity_str, category_str, error.message);

        if let Some(ref context) = error.context {
            print!(" {}", format!("({})", context).dimmed());
        }
        println!();

        if let Some(ref suggestion) = error.suggestion {
            println!("  {} {}", "Suggestion:".cyan(), suggestion);
        }
    }
}

/// Handle the convert subcommand
fn handle_convert(
    input: &Path,
    output: &Path,
    format: ConversionFormat,
    force: bool,
    json_output: bool,
) -> i32 {
    // Check if output exists and force is not set
    if output.exists() && !force {
        if json_output {
            eprintln!("{{\"error\": \"Output file exists. Use --force to overwrite.\"}}");
        } else {
            eprintln!(
                "{} Output file '{}' already exists. Use --force to overwrite.",
                "Error:".red().bold(),
                output.display()
            );
        }
        return EXIT_FILE_ERROR;
    }

    // Read input file
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to read input file: {}\"}}", e);
            } else {
                eprintln!("{} Failed to read input file: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Parse the workflow
    let workflow: BpmnJsonWorkflow = match serde_json::from_str(&content) {
        Ok(w) => w,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to parse workflow: {}\"}}", e);
            } else {
                eprintln!("{} Failed to parse workflow: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Convert based on format
    let output_content = match format {
        ConversionFormat::BpmnJson => {
            // Just pretty-print the existing format
            match serde_json::to_string_pretty(&workflow) {
                Ok(s) => s,
                Err(e) => {
                    if json_output {
                        eprintln!("{{\"error\": \"Failed to serialize workflow: {}\"}}", e);
                    } else {
                        eprintln!("{} Failed to serialize workflow: {}", "Error:".red().bold(), e);
                    }
                    return EXIT_FILE_ERROR;
                },
            }
        },
        ConversionFormat::Snarl | ConversionFormat::Diagram => {
            if json_output {
                eprintln!(
                    "{{\"error\": \"Format conversion to {:?} is not yet implemented\"}}",
                    format
                );
            } else {
                eprintln!(
                    "{} Format conversion to {:?} is not yet implemented",
                    "Error:".red().bold(),
                    format
                );
                eprintln!("Currently only 'bpmn-json' format is supported for output.");
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Write output file
    match fs::write(output, output_content) {
        Ok(_) => {
            if json_output {
                println!("{{\"success\": true, \"output\": \"{}\"}}", output.display());
            } else {
                println!(
                    "{} Converted '{}' to '{}'",
                    "Success:".green().bold(),
                    input.display(),
                    output.display()
                );
            }
            EXIT_SUCCESS
        },
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to write output file: {}\"}}", e);
            } else {
                eprintln!("{} Failed to write output file: {}", "Error:".red().bold(), e);
            }
            EXIT_FILE_ERROR
        },
    }
}

/// Handle the info subcommand
fn handle_info(file: &Path, detailed: bool, stats_only: bool, json_output: bool) -> i32 {
    // Read the file
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to read file: {}\"}}", e);
            } else {
                eprintln!("{} Failed to read file: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Parse the workflow
    let workflow: BpmnJsonWorkflow = match serde_json::from_str(&content) {
        Ok(w) => w,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to parse workflow: {}\"}}", e);
            } else {
                eprintln!("{} Failed to parse workflow: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Calculate statistics
    let mut node_types: HashMap<String, usize> = HashMap::new();
    let mut start_events = 0;
    let mut end_events = 0;
    let mut tasks = 0;
    let mut gateways = 0;

    for step in &workflow.workflow_steps {
        *node_types.entry(step.step_type.clone()).or_insert(0) += 1;

        match step.step_type.as_str() {
            "startEvent" => start_events += 1,
            "endEvent" => end_events += 1,
            t if t.ends_with("Task") => tasks += 1,
            t if t.ends_with("Gateway") => gateways += 1,
            _ => {},
        }
    }

    if json_output {
        let nodes = if detailed {
            Some(
                workflow
                    .workflow_steps
                    .iter()
                    .map(|s| NodeInfoOutput {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        node_type: s.step_type.clone(),
                        task_type: s.task_type.clone(),
                        event_type: s.event_type.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        let output = InfoOutput {
            file: file.display().to_string(),
            process: ProcessInfoOutput {
                id: workflow.bpmn_process.id.clone(),
                name: workflow.bpmn_process.name.clone(),
                version: workflow.bpmn_process.version.clone(),
                is_executable: workflow.bpmn_process.is_executable,
                process_type: workflow.bpmn_process.process_type.clone(),
            },
            statistics: StatisticsOutput {
                total_nodes: workflow.workflow_steps.len(),
                total_flows: workflow.sequence_flows.len(),
                node_types,
                start_events,
                end_events,
                tasks,
                gateways,
            },
            nodes,
        };

        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("{{\"error\": \"Failed to serialize output: {}\"}}", e);
                return EXIT_FILE_ERROR;
            }
        }
    } else {
        if !stats_only {
            println!("{}", "Workflow Information".bold().underline());
            println!();
            println!("{}: {}", "File".cyan(), file.display());
            println!("{}: {}", "Process ID".cyan(), workflow.bpmn_process.id);
            println!("{}: {}", "Process Name".cyan(), workflow.bpmn_process.name);
            println!("{}: {}", "Version".cyan(), workflow.bpmn_process.version);
            println!("{}: {}", "Executable".cyan(), workflow.bpmn_process.is_executable);
            println!("{}: {}", "Process Type".cyan(), workflow.bpmn_process.process_type);
            println!();
        }

        println!("{}", "Statistics".bold().underline());
        println!();
        println!("{}: {}", "Total Nodes".cyan(), workflow.workflow_steps.len());
        println!("{}: {}", "Total Flows".cyan(), workflow.sequence_flows.len());
        println!("{}: {}", "Start Events".cyan(), start_events);
        println!("{}: {}", "End Events".cyan(), end_events);
        println!("{}: {}", "Tasks".cyan(), tasks);
        println!("{}: {}", "Gateways".cyan(), gateways);
        println!();

        if !node_types.is_empty() {
            println!("{}", "Node Types:".cyan());
            for (node_type, count) in node_types.iter() {
                println!("  {}: {}", node_type, count);
            }
            println!();
        }

        if detailed && !workflow.workflow_steps.is_empty() {
            println!("{}", "Nodes".bold().underline());
            println!();
            for step in &workflow.workflow_steps {
                println!("{} {} ({})", "•".cyan(), step.name.bold(), step.id);
                println!("  {}: {}", "Type".dimmed(), step.step_type);
                if let Some(ref task_type) = step.task_type {
                    println!("  {}: {}", "Task Type".dimmed(), task_type);
                }
                if let Some(ref event_type) = step.event_type {
                    println!("  {}: {}", "Event Type".dimmed(), event_type);
                }
                if !step.inputs.is_empty() {
                    println!("  {}: {}", "Inputs".dimmed(), step.inputs.join(", "));
                }
                if !step.outputs.is_empty() {
                    println!("  {}: {}", "Outputs".dimmed(), step.outputs.join(", "));
                }
                println!();
            }
        }
    }

    EXIT_SUCCESS
}

/// Handle the fix subcommand
fn handle_fix(
    file: &Path,
    in_place: bool,
    output: Option<PathBuf>,
    categories: Option<Vec<String>>,
    json_output: bool,
) -> i32 {
    // Read the file
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to read file: {}\"}}", e);
            } else {
                eprintln!("{} Failed to read file: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Parse the workflow
    let mut workflow: BpmnJsonWorkflow = match serde_json::from_str(&content) {
        Ok(w) => w,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to parse workflow: {}\"}}", e);
            } else {
                eprintln!("{} Failed to parse workflow: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Apply fixes
    let fixes_applied = apply_automatic_fixes(&mut workflow, categories);

    if fixes_applied == 0 {
        if json_output {
            println!("{{\"fixes_applied\": 0, \"message\": \"No fixes needed\"}}");
        } else {
            println!("{} No fixes needed", "Info:".blue().bold());
        }
        return EXIT_SUCCESS;
    }

    // Serialize the fixed workflow
    let fixed_content = match serde_json::to_string_pretty(&workflow) {
        Ok(s) => s,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to serialize fixed workflow: {}\"}}", e);
            } else {
                eprintln!("{} Failed to serialize fixed workflow: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    // Determine output destination
    if in_place {
        // Write back to original file
        match fs::write(file, fixed_content) {
            Ok(_) => {
                if json_output {
                    println!(
                        "{{\"fixes_applied\": {}, \"output\": \"{}\"}}",
                        fixes_applied,
                        file.display()
                    );
                } else {
                    println!(
                        "{} Applied {} fix(es) to '{}'",
                        "Success:".green().bold(),
                        fixes_applied,
                        file.display()
                    );
                }
                EXIT_SUCCESS
            },
            Err(e) => {
                if json_output {
                    eprintln!("{{\"error\": \"Failed to write file: {}\"}}", e);
                } else {
                    eprintln!("{} Failed to write file: {}", "Error:".red().bold(), e);
                }
                EXIT_FILE_ERROR
            },
        }
    } else if let Some(output_path) = output {
        // Write to specified output file
        match fs::write(&output_path, fixed_content) {
            Ok(_) => {
                if json_output {
                    println!(
                        "{{\"fixes_applied\": {}, \"output\": \"{}\"}}",
                        fixes_applied,
                        output_path.display()
                    );
                } else {
                    println!(
                        "{} Applied {} fix(es), saved to '{}'",
                        "Success:".green().bold(),
                        fixes_applied,
                        output_path.display()
                    );
                }
                EXIT_SUCCESS
            },
            Err(e) => {
                if json_output {
                    eprintln!("{{\"error\": \"Failed to write output file: {}\"}}", e);
                } else {
                    eprintln!("{} Failed to write output file: {}", "Error:".red().bold(), e);
                }
                EXIT_FILE_ERROR
            },
        }
    } else {
        // Output to stdout
        println!("{}", fixed_content);
        EXIT_SUCCESS
    }
}

/// Apply automatic fixes to a workflow
fn apply_automatic_fixes(
    workflow: &mut BpmnJsonWorkflow,
    _categories: Option<Vec<String>>,
) -> usize {
    let mut fixes = 0;

    // Fix 1: Ensure version format
    if !workflow.bpmn_process.version.contains('.') {
        workflow.bpmn_process.version = "1.0.0".to_string();
        fixes += 1;
    }

    // Fix 2: Trim whitespace from IDs and names
    workflow.bpmn_process.id = workflow.bpmn_process.id.trim().to_string();
    workflow.bpmn_process.name = workflow.bpmn_process.name.trim().to_string();

    for step in &mut workflow.workflow_steps {
        let old_id = step.id.clone();
        step.id = step.id.trim().to_string();
        step.name = step.name.trim().to_string();

        if old_id != step.id {
            fixes += 1;
        }
    }

    for flow in &mut workflow.sequence_flows {
        flow.id = flow.id.trim().to_string();
        flow.source_ref = flow.source_ref.trim().to_string();
        flow.target_ref = flow.target_ref.trim().to_string();
    }

    // Fix 3: Ensure process name is not empty
    if workflow.bpmn_process.name.is_empty() {
        workflow.bpmn_process.name = workflow.bpmn_process.id.clone();
        fixes += 1;
    }

    // Fix 4: Ensure node names are not empty
    for step in &mut workflow.workflow_steps {
        if step.name.is_empty() {
            step.name = step.id.clone();
            fixes += 1;
        }
    }

    fixes
}

/// Handle the list subcommand
fn handle_list(
    directory: &Path,
    recursive: bool,
    errors_only: bool,
    validate: bool,
    json_output: bool,
) -> i32 {
    if !directory.is_dir() {
        if json_output {
            eprintln!("{{\"error\": \"Not a directory: {}\"}}", directory.display());
        } else {
            eprintln!("{} '{}' is not a directory", "Error:".red().bold(), directory.display());
        }
        return EXIT_FILE_ERROR;
    }

    // Find all JSON files
    let files = match find_json_files(directory, recursive) {
        Ok(f) => f,
        Err(e) => {
            if json_output {
                eprintln!("{{\"error\": \"Failed to scan directory: {}\"}}", e);
            } else {
                eprintln!("{} Failed to scan directory: {}", "Error:".red().bold(), e);
            }
            return EXIT_FILE_ERROR;
        },
    };

    if files.is_empty() {
        if json_output {
            println!("{{\"total_files\": 0, \"message\": \"No JSON files found\"}}");
        } else {
            println!("{} No JSON files found in '{}'", "Info:".blue().bold(), directory.display());
        }
        return EXIT_SUCCESS;
    }

    // Process each file
    let mut entries = Vec::new();
    let mut valid_files = 0;
    let mut files_with_errors = 0;

    for file in &files {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => {
                entries.push(FileListEntry {
                    path: file.display().to_string(),
                    status: "unreadable".to_string(),
                    errors: None,
                    warnings: None,
                });
                continue;
            },
        };

        if validate {
            match validate_bpmn_json(&content) {
                Ok(()) => {
                    valid_files += 1;
                    if !errors_only {
                        entries.push(FileListEntry {
                            path: file.display().to_string(),
                            status: "valid".to_string(),
                            errors: Some(0),
                            warnings: Some(0),
                        });
                    }
                },
                Err(errors) => {
                    let summary = ValidationSummary::from_errors(&errors);
                    if summary.has_errors() {
                        files_with_errors += 1;
                    } else {
                        valid_files += 1;
                    }

                    entries.push(FileListEntry {
                        path: file.display().to_string(),
                        status: if summary.has_errors() { "invalid" } else { "warnings" }
                            .to_string(),
                        errors: Some(summary.errors),
                        warnings: Some(summary.warnings),
                    });
                },
            }
        } else {
            // Just check if it's valid JSON
            match serde_json::from_str::<BpmnJsonWorkflow>(&content) {
                Ok(_) => {
                    valid_files += 1;
                    if !errors_only {
                        entries.push(FileListEntry {
                            path: file.display().to_string(),
                            status: "parseable".to_string(),
                            errors: None,
                            warnings: None,
                        });
                    }
                },
                Err(_) => {
                    files_with_errors += 1;
                    entries.push(FileListEntry {
                        path: file.display().to_string(),
                        status: "parse_error".to_string(),
                        errors: None,
                        warnings: None,
                    });
                },
            }
        }
    }

    if json_output {
        let output = ListOutput {
            directory: directory.display().to_string(),
            total_files: files.len(),
            valid_files,
            files_with_errors,
            files: entries,
        };
        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("{{\"error\": \"Failed to serialize output: {}\"}}", e);
                return EXIT_FILE_ERROR;
            }
        }
    } else {
        println!("{}", "Workflow Files".bold().underline());
        println!();
        println!("{}: {}", "Directory".cyan(), directory.display());
        println!("{}: {}", "Total Files".cyan(), files.len());
        println!("{}: {}", "Valid".cyan(), valid_files.to_string().green());
        println!(
            "{}: {}",
            "With Errors".cyan(),
            if files_with_errors > 0 {
                files_with_errors.to_string().red()
            } else {
                files_with_errors.to_string().green()
            }
        );
        println!();

        for entry in &entries {
            let status_str = match entry.status.as_str() {
                "valid" => "✓".green(),
                "parseable" => "✓".green(),
                "warnings" => "⚠".yellow(),
                "invalid" => "✗".red(),
                "parse_error" => "✗".red(),
                "unreadable" => "?".dimmed(),
                _ => "?".dimmed(),
            };

            print!("{} {}", status_str, entry.path);

            if let (Some(errors), Some(warnings)) = (entry.errors, entry.warnings) {
                if errors > 0 || warnings > 0 {
                    print!(" (");
                    if errors > 0 {
                        print!("{} errors", errors.to_string().red());
                    }
                    if errors > 0 && warnings > 0 {
                        print!(", ");
                    }
                    if warnings > 0 {
                        print!("{} warnings", warnings.to_string().yellow());
                    }
                    print!(")");
                }
            }
            println!();
        }
    }

    if files_with_errors > 0 {
        EXIT_VALIDATION_ERROR
    } else {
        EXIT_SUCCESS
    }
}

/// Find all JSON files in a directory
fn find_json_files(dir: &Path, recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    files.push(path);
                }
            }
        } else if path.is_dir() && recursive {
            files.extend(find_json_files(&path, recursive)?);
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_json_files() {
        // This test would need a test directory structure
        // For now, just test that the function doesn't panic
        let temp_dir = std::env::temp_dir();
        let result = find_json_files(&temp_dir, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_automatic_fixes() {
        let mut workflow = BpmnJsonWorkflow {
            bpmn_process: abcdodaf::bpmn::BpmnProcessInfo {
                id: "  test  ".to_string(),
                name: "".to_string(),
                version: "1".to_string(),
                is_executable: true,
                process_type: "test".to_string(),
            },
            workflow_steps: vec![],
            sequence_flows: vec![],
            dodaf_metadata: None,
            task_metadata: None,
        };

        let fixes = apply_automatic_fixes(&mut workflow, None);
        assert!(fixes > 0);
        assert_eq!(workflow.bpmn_process.id, "test");
        assert_eq!(workflow.bpmn_process.name, "test");
        assert_eq!(workflow.bpmn_process.version, "1.0.0");
    }
}
