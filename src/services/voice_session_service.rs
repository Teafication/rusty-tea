use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug};

/// In-memory voice chat session with TTL
#[derive(Debug, Clone)]
pub struct VoiceSession {
    pub messages: Vec<(String, String)>, // (role, content)
    pub last_activity: Instant,
}

impl VoiceSession {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            last_activity: Instant::now(),
        }
    }

    fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push((role.to_string(), content.to_string()));
        self.update_activity();
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.last_activity.elapsed() > ttl
    }
}

/// Service for managing ephemeral voice chat sessions
#[derive(Clone)]
pub struct VoiceSessionService {
    sessions: Arc<RwLock<HashMap<Uuid, VoiceSession>>>,
    session_ttl: Duration,
}

impl VoiceSessionService {
    pub fn new(session_ttl_minutes: u64) -> Self {
        info!("Initializing VoiceSessionService with TTL: {} minutes", session_ttl_minutes);
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_ttl: Duration::from_secs(session_ttl_minutes * 60),
        }
    }

    /// Get conversation history for a session
    pub async fn get_history(&self, session_id: Uuid) -> Vec<(String, String)> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(&session_id) {
            debug!("Retrieved history for session {}: {} messages", session_id, session.messages.len());
            session.messages.clone()
        } else {
            debug!("No history found for session {}, creating new session", session_id);
            Vec::new()
        }
    }

    /// Add a message to the session history
    pub async fn add_message(&self, session_id: Uuid, role: &str, content: &str) {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.entry(session_id).or_insert_with(VoiceSession::new);
        session.add_message(role, content);
        
        debug!("Added {} message to session {}: {} total messages", 
               role, session_id, session.messages.len());
    }

    /// Clean up expired sessions (call periodically)
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|session_id, session| {
            let expired = session.is_expired(self.session_ttl);
            if expired {
                info!("Expiring session {} (inactive for {:?})", session_id, session.last_activity.elapsed());
            }
            !expired
        });
        
        let removed = initial_count - sessions.len();
        if removed > 0 {
            info!("Cleaned up {} expired voice sessions ({} active remaining)", removed, sessions.len());
        }
    }

    /// Get current session count (for monitoring)
    pub async fn active_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5 * 60)); // Check every 5 minutes
            
            loop {
                interval.tick().await;
                self.cleanup_expired_sessions().await;
            }
        });
        
        info!("Started voice session cleanup background task");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let service = VoiceSessionService::new(30);
        let session_id = Uuid::new_v4();
        
        service.add_message(session_id, "user", "Hello").await;
        let history = service.get_history(session_id).await;
        
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0, "user");
        assert_eq!(history[0].1, "Hello");
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let service = VoiceSessionService::new(0); // 0 minute TTL for testing
        let session_id = Uuid::new_v4();
        
        service.add_message(session_id, "user", "Test").await;
        assert_eq!(service.active_session_count().await, 1);
        
        // Wait a bit and cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
        service.cleanup_expired_sessions().await;
        
        assert_eq!(service.active_session_count().await, 0);
    }
}
