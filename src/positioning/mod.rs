mod semantic;
mod interpolation;
pub mod spatial;

pub use semantic::SemanticPositioner;
pub use interpolation::*;
pub use spatial::{CollisionAvoidance, SpatialHash};

/// A 2D position in normalized coordinates (0.0 to 1.0)
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a position from terminal coordinates
    pub fn from_terminal(col: u16, row: u16, width: u16, height: u16) -> Self {
        Self {
            x: col as f32 / width as f32,
            y: row as f32 / height as f32,
        }
    }

    /// Convert to terminal coordinates
    pub fn to_terminal(&self, width: u16, height: u16) -> (u16, u16) {
        let col = (self.x * (width - 1) as f32).round() as u16;
        let row = (self.y * (height - 1) as f32).round() as u16;
        (col.min(width - 1), row.min(height - 1))
    }

    /// Linear interpolation toward another position
    pub fn lerp(&self, target: &Position, t: f32) -> Position {
        let t = t.clamp(0.0, 1.0);
        Position {
            x: self.x + (target.x - self.x) * t,
            y: self.y + (target.y - self.y) * t,
        }
    }

    /// Distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Clamp position to valid range
    pub fn clamp(&self) -> Position {
        Position {
            x: self.x.clamp(0.05, 0.95),
            y: self.y.clamp(0.05, 0.95),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}
