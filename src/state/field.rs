use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::event::{AgentId, Connection, HiveEvent, Landmark, LandmarkId};
use crate::positioning::{CollisionAvoidance, Position, SemanticPositioner};

use super::agent::Agent;

/// Active connection between agents with animation state
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub from: AgentId,
    pub to: AgentId,
    pub label: String,
    pub created_at: Instant,
    pub opacity: f32,
    pub fading_out: bool,
}

impl ActiveConnection {
    pub fn new(conn: &Connection) -> Self {
        Self {
            from: conn.from.clone(),
            to: conn.to.clone(),
            label: conn.label.clone(),
            created_at: Instant::now(),
            opacity: 0.0,
            fading_out: false,
        }
    }

    /// Update animation state, returns true if connection should be removed
    pub fn tick(&mut self, dt: f32) -> bool {
        let age = self.created_at.elapsed();

        if self.fading_out {
            self.opacity = (self.opacity - dt * 2.0).max(0.0);
            return self.opacity <= 0.0;
        }

        // Fade in over 0.3 seconds
        if age < Duration::from_millis(300) {
            self.opacity = (age.as_secs_f32() / 0.3).min(1.0);
        }
        // Hold for 3 seconds, then start fading
        else if age > Duration::from_secs(3) {
            self.fading_out = true;
        }

        false
    }
}

/// Stored landmark for display
#[derive(Debug, Clone)]
pub struct StoredLandmark {
    pub id: LandmarkId,
    pub label: String,
    pub keywords: Vec<String>,
    pub position: Position,
}

/// The field state containing all agents, connections, and landmarks
pub struct Field {
    pub agents: HashMap<AgentId, Agent>,
    pub connections: Vec<ActiveConnection>,
    pub landmarks: HashMap<LandmarkId, StoredLandmark>,
    pub positioner: SemanticPositioner,

    /// Counter for assigning colors to new agents
    agent_color_counter: usize,

    /// Paused state for replay
    pub paused: bool,

    /// Playback speed multiplier
    pub playback_speed: f32,

    /// Collision avoidance system using spatial hash
    collision_avoidance: CollisionAvoidance,
}

impl Field {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            connections: Vec::new(),
            landmarks: HashMap::new(),
            positioner: SemanticPositioner::new(),
            agent_color_counter: 0,
            paused: false,
            playback_speed: 1.0,
            collision_avoidance: CollisionAvoidance::new(),
        }
    }

    /// Process an incoming event
    pub fn process_event(&mut self, event: &HiveEvent) {
        match event {
            HiveEvent::AgentUpdate(update) => {
                let agent = self.agents.entry(update.agent_id.clone()).or_insert_with(|| {
                    let color_idx = self.agent_color_counter;
                    self.agent_color_counter += 1;
                    Agent::new(update.agent_id.clone(), color_idx)
                });

                agent.apply_update(update);

                // Calculate new target position based on focus
                let target = self.positioner.calculate_position(&update.focus, &self.landmarks);
                agent.set_target(target);
            }

            HiveEvent::Connection(conn) => {
                // Remove any existing connection between same agents
                self.connections.retain(|c| {
                    !((c.from == conn.from && c.to == conn.to)
                        || (c.from == conn.to && c.to == conn.from))
                });

                self.connections.push(ActiveConnection::new(conn));
            }

            HiveEvent::Landmark(landmark) => {
                let position = self.positioner.register_landmark(&landmark.keywords);

                self.landmarks.insert(
                    landmark.id.clone(),
                    StoredLandmark {
                        id: landmark.id.clone(),
                        label: landmark.label.clone(),
                        keywords: landmark.keywords.clone(),
                        position,
                    },
                );
            }
        }
    }

    /// Update all animations (called every frame)
    pub fn tick(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        let adjusted_dt = dt * self.playback_speed;

        // Update agents
        for agent in self.agents.values_mut() {
            agent.tick(adjusted_dt);
        }

        // Apply collision avoidance after position updates
        self.apply_collision_avoidance();

        // Update connections, removing expired ones
        self.connections.retain_mut(|conn| !conn.tick(adjusted_dt));
    }

    /// Apply collision avoidance to prevent agents from overlapping
    /// Uses spatial hash for O(n) average time complexity
    fn apply_collision_avoidance(&mut self) {
        if self.agents.len() < 2 {
            return;
        }

        // Collect agent IDs and positions in a stable order
        let mut agent_ids: Vec<AgentId> = self.agents.keys().cloned().collect();
        agent_ids.sort();

        let mut positions: Vec<Position> = agent_ids
            .iter()
            .map(|id| self.agents.get(id).unwrap().position.clone())
            .collect();

        // Calculate and apply separation forces using spatial hash
        let forces = self.collision_avoidance.calculate_separation_forces(&positions);

        // Apply forces to positions
        for (i, (fx, fy)) in forces.into_iter().enumerate() {
            positions[i].x = (positions[i].x + fx).clamp(0.05, 0.95);
            positions[i].y = (positions[i].y + fy).clamp(0.05, 0.95);
        }

        // Update agent positions
        for (i, id) in agent_ids.iter().enumerate() {
            if let Some(agent) = self.agents.get_mut(id) {
                agent.position = positions[i].clone();
            }
        }
    }

    /// Get agent position by ID
    pub fn get_agent_position(&self, id: &str) -> Option<Position> {
        self.agents.get(id).map(|a| a.position.clone())
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Adjust playback speed
    pub fn adjust_speed(&mut self, delta: f32) {
        self.playback_speed = (self.playback_speed + delta).clamp(0.25, 4.0);
    }

    /// Get sorted list of agents for consistent rendering
    pub fn agents_sorted(&self) -> Vec<&Agent> {
        let mut agents: Vec<_> = self.agents.values().collect();
        agents.sort_by(|a, b| a.id.cmp(&b.id));
        agents
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}
