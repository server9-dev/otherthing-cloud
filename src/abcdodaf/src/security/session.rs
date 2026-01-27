//! Session management for user sessions
//!
//! Provides session creation, validation, and lifecycle management with
//! configurable timeout and security features.

use crate::security::error::{SecurityError, SecurityResult};
use crate::security::rbac::Subject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and valid
    Active,
    /// Session is suspended
    Suspended,
    /// Session has been terminated
    Terminated,
    /// Session has expired
    Expired,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Active => write!(f, "Active"),
            SessionState::Suspended => write!(f, "Suspended"),
            SessionState::Terminated => write!(f, "Terminated"),
            SessionState::Expired => write!(f, "Expired"),
        }
    }
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub session_id: String,
    /// Subject associated with this session
    pub subject: Subject,
    /// Session state
    pub state: SessionState,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Session timeout duration
    pub timeout_duration: chrono::Duration,
    /// IP address of the session
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(subject: Subject, timeout_duration: chrono::Duration) -> Self {
        let now = chrono::Utc::now();
        let expires_at = now + timeout_duration;

        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            subject,
            state: SessionState::Active,
            created_at: now,
            last_activity_at: now,
            expires_at,
            timeout_duration,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        }
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if session is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.state == SessionState::Active && chrono::Utc::now() <= self.expires_at
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity_at = chrono::Utc::now();

        // Extend expiration if still active
        if self.state == SessionState::Active {
            self.expires_at = chrono::Utc::now() + self.timeout_duration;
        }
    }

    /// Terminate the session
    pub fn terminate(&mut self) {
        self.state = SessionState::Terminated;
    }

    /// Suspend the session
    pub fn suspend(&mut self) {
        self.state = SessionState::Suspended;
    }

    /// Mark as expired
    pub fn mark_expired(&mut self) {
        self.state = SessionState::Expired;
    }

    /// Get idle duration (time since last activity)
    pub fn idle_duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.last_activity_at
    }
}

/// Session manager configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Default session timeout
    pub default_timeout: chrono::Duration,
    /// Maximum idle time before session expires
    pub max_idle_time: chrono::Duration,
    /// Whether to extend session on activity
    pub extend_on_activity: bool,
    /// Maximum number of concurrent sessions per subject
    pub max_concurrent_sessions: usize,
    /// Whether to validate IP address consistency
    pub validate_ip_consistency: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_timeout: chrono::Duration::hours(24),
            max_idle_time: chrono::Duration::hours(1),
            extend_on_activity: true,
            max_concurrent_sessions: 5,
            validate_ip_consistency: false,
        }
    }
}

/// Session manager
#[derive(Clone)]
pub struct SessionManager {
    /// Stored sessions (session_id -> session)
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Subject to sessions mapping (subject_id -> session_ids)
    subject_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Configuration
    config: Arc<SessionConfig>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            subject_sessions: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SessionConfig::default())
    }

    /// Create a new session
    pub async fn create_session(&self, subject: Subject) -> SecurityResult<Session> {
        let subject_id = subject.id.clone();

        // Check concurrent sessions limit
        let subject_sessions = self.subject_sessions.read().await;
        if let Some(sessions) = subject_sessions.get(&subject_id) {
            if sessions.len() >= self.config.max_concurrent_sessions {
                return Err(SecurityError::Other(
                    "Maximum concurrent sessions exceeded".to_string(),
                ));
            }
        }
        drop(subject_sessions);

        // Create new session
        let session = Session::new(subject, self.config.default_timeout);
        let session_id = session.session_id.clone();

        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        // Map subject to session
        let mut subject_sessions = self.subject_sessions.write().await;
        subject_sessions
            .entry(subject_id)
            .or_insert_with(Vec::new)
            .push(session_id);

        Ok(session)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> SecurityResult<Option<Session>> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            // Check if expired
            if session.is_expired() {
                return Ok(None);
            }
            Ok(Some(session.clone()))
        } else {
            Ok(None)
        }
    }

    /// Validate session
    pub async fn validate_session(&self, session_id: &str) -> SecurityResult<bool> {
        if let Some(session) = self.get_session(session_id).await? {
            Ok(session.is_valid())
        } else {
            Ok(false)
        }
    }

    /// Touch session (update activity time)
    pub async fn touch_session(&self, session_id: &str) -> SecurityResult<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_expired() && session.state == SessionState::Active {
                session.touch();
                Ok(())
            } else {
                Err(SecurityError::InvalidSession("Session is not valid".to_string()))
            }
        } else {
            Err(SecurityError::InvalidSession(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Terminate session
    pub async fn terminate_session(&self, session_id: &str) -> SecurityResult<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            let subject_id = session.subject.id.clone();
            session.terminate();

            drop(sessions);

            // Remove from subject mapping
            let mut subject_sessions = self.subject_sessions.write().await;
            if let Some(sessions_vec) = subject_sessions.get_mut(&subject_id) {
                sessions_vec.retain(|s| s != session_id);
            }

            Ok(())
        } else {
            Err(SecurityError::InvalidSession(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Suspend session
    pub async fn suspend_session(&self, session_id: &str) -> SecurityResult<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.suspend();
            Ok(())
        } else {
            Err(SecurityError::InvalidSession(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Resume suspended session
    pub async fn resume_session(&self, session_id: &str) -> SecurityResult<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            if session.state == SessionState::Suspended {
                session.state = SessionState::Active;
                Ok(())
            } else {
                Err(SecurityError::Other(
                    "Can only resume suspended sessions".to_string(),
                ))
            }
        } else {
            Err(SecurityError::InvalidSession(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Get all active sessions for a subject
    pub async fn get_subject_sessions(&self, subject_id: &str) -> SecurityResult<Vec<Session>> {
        let subject_sessions = self.subject_sessions.read().await;
        let session_ids = subject_sessions
            .get(subject_id)
            .cloned()
            .unwrap_or_default();

        let sessions = self.sessions.read().await;
        let mut result = Vec::new();

        for session_id in session_ids {
            if let Some(session) = sessions.get(&session_id) {
                if session.is_valid() {
                    result.push(session.clone());
                }
            }
        }

        Ok(result)
    }

    /// Terminate all sessions for a subject
    pub async fn terminate_subject_sessions(&self, subject_id: &str) -> SecurityResult<usize> {
        let subject_sessions = self.subject_sessions.read().await;
        let session_ids = subject_sessions
            .get(subject_id)
            .cloned()
            .unwrap_or_default();
        drop(subject_sessions);

        let mut sessions = self.sessions.write().await;
        let mut count = 0;

        for session_id in session_ids {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.terminate();
                count += 1;
            }
        }

        Ok(count)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> SecurityResult<usize> {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();

        sessions.retain(|_, session| !session.is_expired());

        let removed = before - sessions.len();
        Ok(removed)
    }

    /// Get session count
    pub async fn count_sessions(&self) -> SecurityResult<usize> {
        let sessions = self.sessions.read().await;
        Ok(sessions.len())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::rbac::SubjectType;

    #[test]
    fn test_session_creation() {
        let subject = Subject::new("user1", SubjectType::User);
        let session = Session::new(subject, chrono::Duration::hours(1));

        assert_eq!(session.state, SessionState::Active);
        assert!(session.is_valid());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_touch() {
        let subject = Subject::new("user1", SubjectType::User);
        let mut session = Session::new(subject, chrono::Duration::hours(1));

        let original_expires = session.expires_at;
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { tokio::time::sleep(std::time::Duration::from_millis(10)).await });

        session.touch();
        assert!(session.expires_at > original_expires);
    }

    #[test]
    fn test_session_termination() {
        let subject = Subject::new("user1", SubjectType::User);
        let mut session = Session::new(subject, chrono::Duration::hours(1));

        session.terminate();
        assert_eq!(session.state, SessionState::Terminated);
        assert!(!session.is_valid());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::with_defaults();
        let subject = Subject::new("user1", SubjectType::User);

        let session = manager.create_session(subject).await.unwrap();
        assert!(manager.validate_session(&session.session_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_session_termination_via_manager() {
        let manager = SessionManager::with_defaults();
        let subject = Subject::new("user1", SubjectType::User);

        let session = manager.create_session(subject).await.unwrap();
        assert!(manager.validate_session(&session.session_id).await.unwrap());

        manager.terminate_session(&session.session_id).await.unwrap();
        assert!(!manager.validate_session(&session.session_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_subject_sessions() {
        let manager = SessionManager::with_defaults();
        let subject = Subject::new("user1", SubjectType::User);

        let _session1 = manager.create_session(subject.clone()).await.unwrap();
        let _session2 = manager.create_session(subject.clone()).await.unwrap();

        let sessions = manager.get_subject_sessions("user1").await.unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
