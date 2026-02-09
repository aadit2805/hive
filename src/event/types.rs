use serde::{Deserialize, Serialize};

/// Represents a unique identifier for an agent
pub type AgentId = String;

/// Represents a unique identifier for a landmark
pub type LandmarkId = String;

/// Status of an agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Active,
    Thinking,
    Waiting,
    Idle,
    Error,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// An event from an agent updating its state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUpdate {
    pub agent_id: AgentId,
    pub status: AgentStatus,
    pub focus: Vec<String>,
    pub intensity: f32,
    pub message: String,
    pub timestamp: u64,
}

/// A connection event between two agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: AgentId,
    pub to: AgentId,
    pub label: String,
    pub timestamp: u64,
}

/// A landmark definition for semantic positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landmark {
    pub id: LandmarkId,
    pub label: String,
    pub keywords: Vec<String>,
    pub timestamp: u64,
}

/// All possible event types that can be received
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HiveEvent {
    AgentUpdate(AgentUpdate),
    Connection(Connection),
    Landmark(Landmark),
}

impl HiveEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            HiveEvent::AgentUpdate(e) => e.timestamp,
            HiveEvent::Connection(e) => e.timestamp,
            HiveEvent::Landmark(e) => e.timestamp,
        }
    }
}

/// A timestamped event for history tracking
#[derive(Debug, Clone)]
pub struct TimestampedEvent {
    pub event: HiveEvent,
    pub received_at: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_update() {
        let json = r#"{"type": "agent_update", "agent_id": "explorer-1", "status": "active", "focus": ["auth", "jwt"], "intensity": 0.8, "message": "Testing", "timestamp": 123}"#;
        let event: HiveEvent = serde_json::from_str(json).unwrap();
        match event {
            HiveEvent::AgentUpdate(u) => {
                assert_eq!(u.agent_id, "explorer-1");
                assert_eq!(u.status, AgentStatus::Active);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_connection() {
        let json = r#"{"type": "connection", "from": "a", "to": "b", "label": "test", "timestamp": 123}"#;
        let event: HiveEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, HiveEvent::Connection(_)));
    }
}
