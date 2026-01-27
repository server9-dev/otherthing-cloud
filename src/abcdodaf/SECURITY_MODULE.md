# ABCDODAF Security Module Documentation

## Overview

The ABCDODAF Security Module provides enterprise-grade security, compliance, and audit logging for the ABCD ODAF library. This module implements:

1. **Role-Based Access Control (RBAC)** - Hierarchical role management with permission inheritance
2. **Permission Model** - Fine-grained access control for workflows, tasks, and resources
3. **Audit Logging** - Comprehensive event logging with filtering and categorization
4. **Encryption Support** - AES-256-GCM encryption for sensitive data
5. **Secret Management** - Secure storage and retrieval of credentials and API keys
6. **Session Management** - User session lifecycle with configurable timeouts
7. **Compliance Framework** - Validation rules and compliance reporting for SOC2, ISO27001, HIPAA, GDPR, PCI DSS
8. **Security Policies** - Enforced policies for passwords, encryption, sessions, and data retention
9. **Audit Export** - Export audit logs in JSON, CSV, and XML formats with optional data redaction

## Architecture

```
┌─────────────────────────────────────────────┐
│        Security Module Architecture         │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────────┐  ┌─────────────────┐   │
│  │    RBAC      │  │  Permissions    │   │
│  │   System     │──┤    Model        │   │
│  └──────────────┘  └─────────────────┘   │
│        │                    │             │
│        └────────┬───────────┘             │
│                 │                         │
│         ┌───────▼────────┐               │
│         │ Security       │               │
│         │ Context        │               │
│         └────────────────┘               │
│                 │                         │
│  ┌──────────────┼──────────────┐         │
│  │              │              │         │
│  ▼              ▼              ▼         │
│ Audit      Encryption      Session      │
│ Logger     Provider         Manager     │
│  │              │              │         │
│  └──────────────┼──────────────┘         │
│                 │                         │
│         ┌───────▼────────────────┐       │
│         │  Secrets Manager       │       │
│         │  (Encrypted Storage)   │       │
│         └────────────────────────┘       │
│                 │                         │
│  ┌──────────────┼──────────────┐         │
│  │              │              │         │
│  ▼              ▼              ▼         │
│Compliance  Security         Audit       │
│Validator   Policy Engine    Exporter    │
│                                         │
└─────────────────────────────────────────┘
```

## Module Components

### 1. RBAC System (`security/rbac.rs`)

Implements a hierarchical role-based access control system with subject-to-role mapping.

#### Key Types:
- `Subject` - Represents a principal (user, service, admin, external)
- `SubjectType` - Enum for subject types
- `Role` - Role definition with permissions and optional parent role
- `RoleManager` - Manages roles and subject-role assignments

#### Standard Roles:
- **viewer** - Read-only access to workflows and tasks
- **editor** - Can create and modify workflows
- **executor** - Can execute workflows and tasks
- **admin** - Full administrative access
- **auditor** - Can read and export audit logs

#### Usage Example:
```rust
use abcdodaf::security::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with standard roles
    let role_manager = Arc::new(RoleManager::with_enterprise_defaults().await);

    // Create a subject
    let user = Subject::new("user123", SubjectType::User)
        .with_name("John Doe")
        .with_email("john@example.com");

    // Assign role
    role_manager.assign_role("user123", "editor").await?;

    // Get permissions
    let perms = role_manager.get_subject_permissions("user123").await?;
    println!("User permissions: {:?}", perms);

    Ok(())
}
```

### 2. Permission Model (`security/permissions.rs`)

Provides fine-grained access control combining role-based and resource-based permissions.

#### Key Types:
- `ResourceType` - Enum for resource types (Workflow, Task, Execution, etc.)
- `Permission` - Individual permission specification
- `ResourcePermissionModel` - Per-resource permission grants and denials
- `PermissionChecker` - Validates permissions
- `PermissionModel` - High-level permission API

#### Usage Example:
```rust
use abcdodaf::security::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let role_manager = Arc::new(RoleManager::with_enterprise_defaults().await);
    let perm_model = PermissionModel::new(role_manager.clone());

    role_manager.assign_role("user1", "editor").await?;
    let user = Subject::new("user1", SubjectType::User);

    // Check permission
    perm_model.verify_permission(&user, ResourceType::Workflow, "read").await?;
    perm_model.verify_permission(&user, ResourceType::Workflow, "edit").await?;

    // Try denied action
    if perm_model.verify_permission(&user, ResourceType::SecurityConfiguration, "edit").await.is_err() {
        println!("User cannot modify security configuration");
    }

    Ok(())
}
```

### 3. Audit Logging (`security/audit.rs`)

Comprehensive audit trail for security-relevant events.

#### Key Types:
- `AuditLevel` - Severity levels (Info, Warning, Error, Critical)
- `EventCategory` - Event categories (Authentication, Authorization, WorkflowOperation, etc.)
- `AuditEvent` - Single audit event with full context
- `AuditLogger` - Main audit logging interface
- `AuditLoggerConfig` - Configuration for logging behavior

#### Logged Events:
- User authentication and session management
- Authorization checks (both successful and denied)
- Workflow and task operations
- Data access and modifications
- Configuration changes
- Security policy violations
- Compliance events

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = AuditLogger::with_defaults();

    // Log an event
    logger.log(
        AuditEvent::new("user1", "create_workflow", EventCategory::WorkflowOperation)
            .with_level(AuditLevel::Info)
            .with_resource("workflow", "wf123")
            .with_ip_address("192.168.1.100")
            .with_session_id("sess456")
            .with_detail("status", "created")
            .with_detail("description", "New workflow created")
    ).await?;

    // Retrieve events by various filters
    let events = logger.get_events().await?;
    let critical = logger.get_events_by_level(AuditLevel::Critical).await?;
    let user_events = logger.get_subject_events("user1").await?;

    Ok(())
}
```

### 4. Encryption Support (`security/encryption.rs`)

Provides encryption/decryption of sensitive data using AES-256-GCM.

#### Key Types:
- `EncryptionAlgorithm` - Enum for encryption algorithms
- `EncryptedData` - Container for encrypted data with metadata
- `EncryptionProvider` - Trait for encryption implementations
- `Aes256GcmProvider` - AES-256-GCM implementation
- `NoOpEncryptionProvider` - Testing provider
- `EncryptionUtils` - Utility functions for string/JSON encryption

#### Features:
- AES-256-GCM encryption with authenticated encryption
- Initialization vectors (nonces) for security
- Associated data support
- Key rotation capability
- String and JSON encryption utilities

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Aes256GcmProvider::default();

    // Encrypt string
    let plaintext = "sensitive data";
    let encrypted = provider.encrypt(plaintext.as_bytes(), None)?;
    println!("Encrypted: {:?}", encrypted);

    // Decrypt
    let decrypted = provider.decrypt(&encrypted)?;
    let decrypted_str = String::from_utf8(decrypted)?;
    assert_eq!(decrypted_str, plaintext);

    // Encrypt JSON
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Config {
        api_key: String,
        endpoint: String,
    }

    let config = Config {
        api_key: "secret123".to_string(),
        endpoint: "https://api.example.com".to_string(),
    };

    let encrypted_json = EncryptionUtils::encrypt_json(&provider, &config)?;
    let decrypted_config: Config = EncryptionUtils::decrypt_json(&provider, &encrypted_json)?;

    Ok(())
}
```

### 5. Secret Management (`security/secrets.rs`)

Secure storage and lifecycle management of credentials and API keys.

#### Key Types:
- `SecretType` - Enum for secret types (ApiKey, DatabasePassword, OAuthToken, etc.)
- `SecretValue` - Encrypted secret with metadata
- `SecretManager` - Manages secret storage and access
- `AccessRecord` - Records of who accessed a secret and when

#### Features:
- Encrypted secret storage
- Access logging and auditing
- Expiration and lifecycle management
- Activation/deactivation
- Metadata tagging
- Access history tracking

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = SecretManager::new().await?;

    // Store secrets
    manager.store_plaintext("db_password", SecretType::DatabasePassword, "SecurePass123!").await?;
    manager.store_plaintext("api_key", SecretType::ApiKey, "sk-proj-abc123").await?;

    // List stored secrets (names only)
    let secret_names = manager.list_secrets().await?;

    // Retrieve secret
    let password = manager.get_secret_string("db_password", "admin1").await?;

    // Check access history
    let access_log = manager.get_access_log("db_password").await?;
    for record in access_log {
        println!("Accessed by {} at {}", record.subject_id, record.timestamp);
    }

    // Deactivate secret
    manager.deactivate_secret("old_api_key").await?;

    // Delete secret
    manager.delete_secret("temporary_key").await?;

    Ok(())
}
```

### 6. Session Management (`security/session.rs`)

User session lifecycle management with configurable timeouts and validation.

#### Key Types:
- `SessionState` - States (Active, Suspended, Terminated, Expired)
- `Session` - User session with full context
- `SessionConfig` - Configuration for session behavior
- `SessionManager` - Manages sessions

#### Session Features:
- Configurable timeout (default 24 hours)
- Maximum idle time before expiration
- Activity-based session extension
- Concurrent session limits per user
- IP address consistency validation
- Session suspension and resuming
- Automatic cleanup of expired sessions

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create with custom config
    let config = SessionConfig {
        default_timeout: chrono::Duration::hours(8),
        max_idle_time: chrono::Duration::minutes(30),
        extend_on_activity: true,
        max_concurrent_sessions: 3,
        validate_ip_consistency: true,
    };

    let manager = SessionManager::new(config);

    // Create session
    let user = Subject::new("user1", SubjectType::User);
    let session = manager.create_session(user).await?;
    println!("Session created: {}", &session.session_id[..8]);

    // Validate session
    if manager.validate_session(&session.session_id).await? {
        println!("Session is valid");
    }

    // Keep session active
    manager.touch_session(&session.session_id).await?;

    // Terminate session
    manager.terminate_session(&session.session_id).await?;

    Ok(())
}
```

### 7. Compliance Framework (`security/compliance.rs`)

Define and validate compliance rules against industry standards.

#### Key Types:
- `ComplianceStandard` - Enum for standards (SOC2TypeII, ISO27001, HIPAA, GDPR, PCIDSS)
- `RuleStatus` - States (Compliant, NonCompliant, Unknown, NotApplicable)
- `ComplianceRule` - Individual compliance rule
- `ComplianceReport` - Generated compliance report
- `ComplianceValidator` - Manages and validates rules

#### Standard Rules:
- Comprehensive audit logging
- Role-based access control
- Data encryption at rest and in transit
- Session management
- Secret management
- Data retention policies

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = ComplianceValidator::with_standard_rules().await;

    // Generate report for specific standards
    let report = validator.generate_report(vec![
        ComplianceStandard::SOC2TypeII,
        ComplianceStandard::ISO27001,
    ]).await?;

    println!("Compliance Report:");
    println!("  Total rules: {}", report.total_rules);
    println!("  Compliant: {}", report.compliant_rules);
    println!("  Non-compliant: {}", report.non_compliant_rules);
    println!("  Score: {:.1}%", report.compliance_score);

    // Get findings
    for finding in report.findings {
        println!("  Finding: {}", finding);
    }

    Ok(())
}
```

### 8. Security Policy Engine (`security/security_policy.rs`)

Define and enforce security policies across the system.

#### Key Types:
- `PolicyType` - Enum for policy types (PasswordPolicy, EncryptionPolicy, SessionPolicy, etc.)
- `SecurityPolicy` - Policy definition with rules
- `PasswordPolicyConfig` - Specific password requirements
- `SecurityPolicyEngine` - Manages and enforces policies

#### Standard Policies:
- **Password Policy**: Min 12 chars, uppercase, lowercase, numbers, special chars, 90-day expiration
- **Encryption Policy**: AES-256-GCM, TLS 1.3 required
- **Session Policy**: 24-hour default timeout, 60-minute idle timeout
- **Audit Policy**: INFO level, comprehensive event tracking
- **Data Retention Policy**: 365 days for audit logs, 90 days for access logs

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SecurityPolicyEngine::with_standard_policies().await;

    // Validate password
    match engine.validate_password("WeakPass").await {
        Ok(_) => println!("Password accepted"),
        Err(e) => println!("Password rejected: {}", e),
    }

    match engine.validate_password("StrongPass123!@").await {
        Ok(_) => println!("Strong password accepted"),
        Err(_) => {},
    }

    // Get active policies
    let policies = engine.get_active_policies().await?;
    println!("Active policies: {}", policies.len());

    Ok(())
}
```

### 9. Audit Export (`security/audit_export.rs`)

Export audit logs in multiple formats with optional data redaction.

#### Key Types:
- `ExportFormat` - JSON, CSV, XML, Binary
- `ExportOptions` - Configuration for export behavior
- `ExportMetadata` - Metadata about the export
- `AuditEventExport` - Audit event for export (may be redacted)
- `AuditExporter` - Converts audit logs to various formats

#### Export Features:
- Multiple format support (JSON, JSON Lines, CSV, XML)
- Data redaction for sensitive information
- Compression support
- Digital signatures
- Export metadata and integrity verification

#### Usage Example:
```rust
use abcdodaf::security::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = AuditLogger::with_defaults();

    // Log events...

    // Export to JSON
    let events = logger.get_events().await?;
    let metadata = AuditExportMetadata::new("admin1");
    let mut export = AuditLogExport::new(metadata);

    for event in events {
        let export_event = AuditEventExport::from_event(event, false); // Redact sensitive
        export.add_event(export_event);
    }

    let json = AuditExporter::to_json(&export)?;
    let csv = AuditExporter::to_csv(&export)?;
    let xml = AuditExporter::to_xml(&export)?;

    Ok(())
}
```

## Security Context

The `SecurityContext` is the central object for access control decisions:

```rust
let context = SecurityContext::new(user_subject)
    .with_role(viewer_role)
    .with_role(editor_role)
    .with_session("session_id_123");

// Check roles
if context.has_role("admin") {
    // Allow admin actions
}

// Get all permissions
let perms = context.get_all_permissions();
```

## Integration Patterns

### Pattern 1: Access Control Check Before Operation

```rust
async fn create_workflow(
    context: &SecurityContext,
    perm_model: &PermissionModel,
    workflow_def: WorkflowDefinition,
) -> Result<String> {
    // Verify permission
    perm_model.verify_permission(
        &context.subject,
        ResourceType::Workflow,
        "create"
    ).await?;

    let wf_id = create_workflow_internal(workflow_def).await?;

    // Log the action
    audit_logger.log(
        AuditEvent::new(&context.subject.id, "create_workflow", EventCategory::WorkflowOperation)
            .with_resource("workflow", &wf_id)
    ).await?;

    Ok(wf_id)
}
```

### Pattern 2: Resource-Level Permissions

```rust
async fn share_workflow(
    context: &SecurityContext,
    perm_checker: &PermissionChecker,
    workflow_id: &str,
    target_user_id: &str,
) -> Result<()> {
    // Check ownership or admin permission
    let is_owner = workflow_owner_is(workflow_id, &context.subject.id).await?;
    let is_admin = context.has_any_role(&["admin"]);

    if !is_owner && !is_admin {
        return Err(SecurityError::PermissionDenied("Not owner of workflow".into()));
    }

    // Grant resource-level permission
    perm_checker.grant_resource_permission(
        workflow_id,
        target_user_id,
        "read"
    ).await?;

    Ok(())
}
```

### Pattern 3: Secret Usage in Workflows

```rust
async fn execute_external_api_task(
    context: &SecurityContext,
    secret_manager: &SecretManager,
    task_config: &TaskConfig,
) -> Result<ApiResponse> {
    // Verify execution permission
    // ... permission check ...

    // Retrieve API key
    let api_key = secret_manager.get_secret_string(
        &task_config.api_key_secret_name,
        &context.subject.id
    ).await?;

    // Log secret access
    audit_logger.log(
        AuditEvent::new(&context.subject.id, "access_secret", EventCategory::DataAccess)
            .with_resource("secret", &task_config.api_key_secret_name)
            .with_level(AuditLevel::Info)
    ).await?;

    // Call API
    let response = call_api(&api_key, task_config).await?;

    Ok(response)
}
```

## Best Practices

1. **Always Create SecurityContext**: Create a SecurityContext for each request/operation
2. **Check Permissions Early**: Verify permissions before expensive operations
3. **Log All Sensitive Operations**: Use AuditLogger for security-relevant events
4. **Never Log Secrets**: Always use SecretManager; never log plaintext credentials
5. **Enforce Policies**: Use SecurityPolicyEngine for consistent enforcement
6. **Regular Compliance Reports**: Generate compliance reports regularly
7. **Export Audit Logs**: Periodically export and archive audit logs
8. **Review Access Logs**: Monitor secret access and audit logs for suspicious activity
9. **Rotate Keys Regularly**: Use encryption provider's key rotation capabilities
10. **Test RBAC**: Thoroughly test role assignments and permission checks

## Compliance Mappings

The compliance framework maps to industry standards:

### SOC 2 Type II
- Audit logging ✓
- Access controls (RBAC) ✓
- Encryption ✓
- Session management ✓
- Secret management ✓

### ISO 27001
- Information security policies ✓
- Access control ✓
- Encryption ✓
- Audit logging ✓
- User authentication ✓

### HIPAA
- Data encryption ✓
- Access controls ✓
- Audit controls ✓
- Integrity controls ✓
- Transmission security ✓

### GDPR
- Data protection ✓
- Access controls ✓
- Data retention ✓
- Breach notification (audit logs) ✓
- User consent (policy management) ✓

### PCI DSS
- Access control ✓
- Encryption ✓
- Audit logging ✓
- Security testing (compliance validator) ✓

## Testing

All security components include comprehensive tests:

```bash
cargo test security::
```

Tests cover:
- RBAC functionality and inheritance
- Permission checks and denials
- Audit event logging and filtering
- Encryption/decryption roundtrips
- Secret storage and retrieval
- Session lifecycle
- Compliance validation
- Policy enforcement
- Audit export formats

## Performance Considerations

- **Audit Logging**: Asynchronous, configurable max events
- **RBAC Lookups**: Cached role and permission data
- **Encryption**: Efficient AES-256-GCM implementation
- **Session Management**: In-memory storage with periodic cleanup
- **Secret Manager**: Lazy decryption on access

## Future Enhancements

1. **Hardware Security Modules (HSM)** support for key storage
2. **Distributed session management** across multiple instances
3. **OAuth/OIDC integration** for external authentication
4. **Multi-factor authentication** support
5. **Fine-grained audit filtering** with query DSL
6. **Encryption key rotation** automation
7. **Secrets scanning** for workflows to detect hardcoded credentials
8. **Rate limiting** for API operations
9. **IP whitelisting** for additional security
10. **Integration with external audit systems** (Splunk, ELK, etc.)

## See Also

- `examples/security_example.rs` - Comprehensive example demonstrating all features
- Individual module documentation in code comments
- Compliance and security standards documentation
