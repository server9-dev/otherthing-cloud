//! Example demonstrating the comprehensive security module
//!
//! This example showcases:
//! - Role-Based Access Control (RBAC)
//! - Permission checking
//! - Audit logging
//! - Compliance validation
//! - Session management
//! - Secret management
//! - Security policy enforcement

use abcdodaf::security::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ABCDODAF Security Module Example ===\n");

    // Initialize RBAC system
    println!("1. Setting up Role-Based Access Control (RBAC)");
    let role_manager = Arc::new(RoleManager::with_enterprise_defaults().await);
    println!("   Created role manager with standard enterprise roles\n");

    // Create users
    println!("2. Creating subjects (users)");
    let admin_user = Subject::new("admin1", SubjectType::Admin)
        .with_name("Admin User")
        .with_email("admin@example.com");

    let editor_user = Subject::new("editor1", SubjectType::User)
        .with_name("Editor User")
        .with_email("editor@example.com");

    let viewer_user = Subject::new("viewer1", SubjectType::User)
        .with_name("Viewer User")
        .with_email("viewer@example.com");

    println!("   Created {} subjects", 3);
    println!("   - Admin: {}", admin_user.name.as_ref().unwrap());
    println!("   - Editor: {}", editor_user.name.as_ref().unwrap());
    println!("   - Viewer: {}", viewer_user.name.as_ref().unwrap());
    println!();

    // Assign roles
    println!("3. Assigning roles to subjects");
    role_manager.assign_role("admin1", "admin").await?;
    role_manager.assign_role("editor1", "editor").await?;
    role_manager.assign_role("viewer1", "viewer").await?;
    println!("   Assigned roles:\n");
    println!("   - admin1: admin (full access)");
    println!("   - editor1: editor (create/edit workflows)");
    println!("   - viewer1: viewer (read-only access)\n");

    // Setup permission checking
    println!("4. Setting up Permission Model");
    let permission_model = PermissionModel::new(role_manager.clone());
    let admin_ctx = SecurityContext::new(admin_user.clone())
        .with_role(role_manager.get_role("admin").await.unwrap().unwrap());
    let editor_ctx = SecurityContext::new(editor_user.clone())
        .with_role(role_manager.get_role("editor").await.unwrap().unwrap());

    // Check permissions
    match permission_model
        .verify_permission(&admin_user, ResourceType::Workflow, "execute")
        .await
    {
        Ok(_) => println!("   ✓ Admin can execute workflows"),
        Err(e) => println!("   ✗ Admin cannot execute workflows: {}", e),
    }

    match permission_model
        .verify_permission(&viewer_user, ResourceType::Workflow, "edit")
        .await
    {
        Ok(_) => println!("   ✓ Viewer can edit workflows"),
        Err(e) => println!("   ✗ Viewer cannot edit workflows (expected)"),
    }
    println!();

    // Setup audit logging
    println!("5. Initializing Audit Logging");
    let audit_logger = AuditLogger::with_defaults();

    // Log some events
    audit_logger
        .log(
            AuditEvent::new("admin1", "create_workflow", EventCategory::WorkflowOperation)
                .with_level(AuditLevel::Info)
                .with_resource("workflow", "wf001")
                .with_detail("status", "created"),
        )
        .await?;

    audit_logger
        .log(
            AuditEvent::new("viewer1", "unauthorized_access", EventCategory::Authorization)
                .with_level(AuditLevel::Warning)
                .with_error("Attempted to edit workflow without permission"),
        )
        .await?;

    let events = audit_logger.get_events().await?;
    println!("   Logged {} events", events.len());
    for event in events.iter().take(2) {
        println!(
            "   - [{}] {} by {} on {}",
            event.level,
            event.action,
            event.subject_id,
            event.resource_type.as_ref().unwrap_or(&"N/A".to_string())
        );
    }
    println!();

    // Setup session management
    println!("6. Setting up Session Management");
    let session_manager = SessionManager::with_defaults();
    let session = session_manager.create_session(admin_user.clone()).await?;
    println!(
        "   Created session: {} for {}",
        &session.session_id[..8],
        session.subject.name.as_ref().unwrap()
    );
    println!("   Session valid: {}", session.is_valid());
    println!();

    // Setup secret management
    println!("7. Initializing Secret Manager");
    let secret_manager = SecretManager::new().await?;
    secret_manager
        .store_plaintext("db_password", SecretType::DatabasePassword, "SecurePass123!")
        .await?;

    secret_manager
        .store_plaintext("api_key", SecretType::ApiKey, "sk-proj-abc123xyz789")
        .await?;

    let stored_secrets = secret_manager.list_secrets().await?;
    println!("   Stored {} secrets:", stored_secrets.len());
    for secret_name in stored_secrets {
        println!("   - {}", secret_name);
    }

    // Retrieve and log access
    let _password = secret_manager
        .get_secret_string("db_password", "admin1")
        .await?;
    let access_log = secret_manager.get_access_log("db_password").await?;
    println!("   Access log for db_password: {} accesses", access_log.len());
    println!();

    // Setup compliance validation
    println!("8. Setting up Compliance Framework");
    let compliance_validator = ComplianceValidator::with_standard_rules().await;

    // Generate compliance report
    let mut report = compliance_validator
        .generate_report(vec![
            ComplianceStandard::SOC2TypeII,
            ComplianceStandard::ISO27001,
        ])
        .await?;

    println!("   Compliance Report:");
    println!("   - Total rules: {}", report.total_rules);
    println!("   - Compliance score: {:.1}%", report.compliance_score);
    println!("   - Rules by status:");
    println!("     • Compliant: {}", report.compliant_rules);
    println!("     • Non-compliant: {}", report.non_compliant_rules);
    println!();

    // Setup security policies
    println!("9. Configuring Security Policies");
    let policy_engine = SecurityPolicyEngine::with_standard_policies().await;

    // Validate password against policy
    match policy_engine.validate_password("WeakPass").await {
        Ok(_) => println!("   ✓ Password meets policy requirements"),
        Err(_) => println!("   ✗ Password does not meet policy requirements (expected)"),
    }

    match policy_engine.validate_password("StrongPass123!@#").await {
        Ok(_) => println!("   ✓ Strong password accepted"),
        Err(e) => println!("   ✗ Strong password rejected: {}", e),
    }

    let policies = policy_engine.list_policies().await?;
    println!("   Active security policies: {}", policies.len());
    println!();

    // Audit export
    println!("10. Audit Log Export");
    let events = audit_logger.get_events().await?;
    let mut export_metadata = AuditExportMetadata::new("admin1");
    export_metadata.format = "JSON".to_string();

    let mut export = AuditLogExport::new(export_metadata);

    for event in events {
        let export_event = AuditEventExport::from_event(event, false); // Redact sensitive data
        export.add_event(export_event);
    }

    let json_export = AuditExporter::to_json(&export)?;
    println!("   Generated JSON export with {} events", export.metadata.event_count);

    let csv_export = AuditExporter::to_csv(&export)?;
    println!("   Generated CSV export ({} bytes)", csv_export.len());

    println!("\n=== Example Complete ===");
    Ok(())
}

// Note: This example uses the following types from the security module
// These are imported at the top using the security module exports

use abcdodaf::security::audit_export::{AuditEventExport, AuditExporter, AuditLogExport, AuditExportMetadata};
use abcdodaf::security::compliance::ComplianceStandard;
