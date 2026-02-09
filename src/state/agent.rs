use crate::event::{AgentId, AgentStatus, AgentUpdate};
use crate::positioning::Position;
use crate::render::symbols::{get_agent_shape, get_status_indicator, detect_unicode, AGENT_SHAPES};
use std::collections::VecDeque;
use std::time::Instant;

/// Maximum number of trail points to keep
const MAX_TRAIL_LENGTH: usize = 50;

/// Represents the visual state of an agent
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub status: AgentStatus,
    pub focus: Vec<String>,
    pub intensity: f32,
    pub message: String,

    /// Current rendered position
    pub position: Position,
    /// Target position (where the agent is moving toward)
    pub target_position: Position,

    /// Trail of recent positions for rendering
    pub trail: VecDeque<TrailPoint>,

    /// Animation state
    pub pulse_phase: f32,
    pub last_update: Instant,

    /// Color index for consistent coloring
    pub color_index: usize,

    /// Shape index for unique agent shape (0-7 maps to AGENT_SHAPES)
    pub shape_index: usize,
}

/// A point in the agent's movement trail
#[derive(Debug, Clone)]
pub struct TrailPoint {
    pub position: Position,
    pub timestamp: Instant,
    pub intensity: f32,
}

impl Agent {
    /// Create a new agent with a color index (shape_index defaults to color_index)
    pub fn new(id: AgentId, color_index: usize) -> Self {
        Self::with_shape(id, color_index, color_index)
    }

    /// Create a new agent with explicit color and shape indices
    pub fn with_shape(id: AgentId, color_index: usize, shape_index: usize) -> Self {
        Self {
            id,
            status: AgentStatus::Idle,
            focus: Vec::new(),
            intensity: 0.0,
            message: String::new(),
            position: Position::new(0.5, 0.5),
            target_position: Position::new(0.5, 0.5),
            trail: VecDeque::with_capacity(MAX_TRAIL_LENGTH),
            pulse_phase: 0.0,
            last_update: Instant::now(),
            color_index,
            shape_index,
        }
    }

    /// Update agent state from an event
    pub fn apply_update(&mut self, update: &AgentUpdate) {
        self.status = update.status.clone();
        self.focus = update.focus.clone();
        self.intensity = update.intensity.clamp(0.0, 1.0);
        self.message = update.message.clone();
        self.last_update = Instant::now();
    }

    /// Set the target position for smooth movement
    pub fn set_target(&mut self, target: Position) {
        self.target_position = target;
    }

    /// Add current position to trail
    pub fn record_trail(&mut self) {
        // Only add if we've moved significantly
        if let Some(last) = self.trail.back() {
            let dist = self.position.distance_to(&last.position);
            if dist < 0.01 {
                return;
            }
        }

        self.trail.push_back(TrailPoint {
            position: self.position.clone(),
            timestamp: Instant::now(),
            intensity: self.intensity,
        });

        // Trim old trail points
        while self.trail.len() > MAX_TRAIL_LENGTH {
            self.trail.pop_front();
        }
    }

    /// Update animation state (called every frame)
    pub fn tick(&mut self, dt: f32) {
        // Update pulse animation
        let pulse_speed = 2.0 + self.intensity * 3.0; // Faster pulse when more intense
        self.pulse_phase = (self.pulse_phase + dt * pulse_speed) % (2.0 * std::f32::consts::PI);

        // Smooth position interpolation toward target
        let lerp_speed = 3.0 * dt;
        self.position = self.position.lerp(&self.target_position, lerp_speed);

        // Record trail periodically
        self.record_trail();
    }

    /// Check if this agent should have pulsing animation
    /// Only agents that are Active with high intensity (> 0.6) pulse
    pub fn should_pulse(&self) -> bool {
        self.status == AgentStatus::Active && self.intensity > 0.6
    }

    /// Get the current pulse brightness multiplier (0.0 to 1.0)
    /// Only active agents with intensity > 0.6 actually pulse;
    /// other agents have static brightness based on their intensity
    pub fn pulse_brightness(&self) -> f32 {
        if self.should_pulse() {
            // Pulsing animation for highly active agents
            let base = 0.85;
            let variation = 0.15;
            base + variation * (self.pulse_phase.sin() * 0.5 + 0.5)
        } else {
            // Static brightness based on intensity for all other agents
            0.6 + self.intensity * 0.4
        }
    }

    /// Get a display symbol based on intensity and status (legacy, returns static str)
    /// Use `symbol_char()` for the new symbol system with Unicode/ASCII support
    pub fn symbol(&self) -> &'static str {
        match self.status {
            AgentStatus::Active => {
                if self.intensity > 0.7 {
                    "◉"
                } else if self.intensity > 0.4 {
                    "◐"
                } else {
                    "◍"
                }
            }
            AgentStatus::Thinking => "◌",
            AgentStatus::Waiting => "○",
            AgentStatus::Idle => "·",
            AgentStatus::Error => "✗",
        }
    }

    /// Get the agent's shape symbol as a char based on shape_index
    /// Uses the new symbol system with Unicode/ASCII fallback support
    pub fn shape_symbol(&self, use_unicode: bool) -> char {
        get_agent_shape(self.shape_index).render(use_unicode)
    }

    /// Get the agent's shape symbol, auto-detecting Unicode support
    pub fn shape_symbol_auto(&self) -> char {
        self.shape_symbol(detect_unicode())
    }

    /// Get the status indicator symbol as a char
    pub fn status_symbol(&self, use_unicode: bool) -> char {
        get_status_indicator(&self.status).render(use_unicode)
    }

    /// Get the status indicator symbol, auto-detecting Unicode support
    pub fn status_symbol_auto(&self) -> char {
        self.status_symbol(detect_unicode())
    }

    /// Get full symbol representation: shape + status indicator
    /// Returns a tuple of (shape_char, status_char)
    pub fn full_symbol(&self, use_unicode: bool) -> (char, char) {
        (
            self.shape_symbol(use_unicode),
            self.status_symbol(use_unicode),
        )
    }

    /// Get the Symbol struct for the agent's shape
    pub fn get_shape(&self) -> &'static crate::render::symbols::Symbol {
        get_agent_shape(self.shape_index)
    }

    /// Get the Symbol struct for the agent's status indicator
    pub fn get_status_indicator(&self) -> &'static crate::render::symbols::Symbol {
        get_status_indicator(&self.status)
    }

    /// Get short display name
    pub fn short_name(&self) -> String {
        if self.id.len() <= 8 {
            self.id.clone()
        } else {
            format!("{}…", &self.id[..7])
        }
    }
}
