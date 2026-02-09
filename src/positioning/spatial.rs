use std::collections::HashMap;

use super::Position;

/// Cell size for spatial hash grid
/// Should be >= 2x min_distance for correct neighbor detection
const DEFAULT_CELL_SIZE: f32 = 0.16;

/// Minimum distance between agents
const DEFAULT_MIN_DISTANCE: f32 = 0.08;

/// Separation force strength
const DEFAULT_SEPARATION_FORCE: f32 = 0.5;

/// Spatial hash grid for O(1) average collision detection
/// Instead of O(n^2) checking all pairs, we only check agents in neighboring cells
#[derive(Debug)]
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
    grid_width: i32,
    grid_height: i32,
}

impl SpatialHash {
    /// Create a new spatial hash with default cell size (0.16)
    pub fn new() -> Self {
        Self::with_cell_size(DEFAULT_CELL_SIZE)
    }

    /// Create a new spatial hash with custom cell size
    pub fn with_cell_size(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            grid_width: (1.0 / cell_size).ceil() as i32,
            grid_height: (1.0 / cell_size).ceil() as i32,
        }
    }

    /// Clear and rebuild the spatial hash with current positions
    pub fn rebuild(&mut self, positions: &[Position]) {
        self.cells.clear();

        for (i, pos) in positions.iter().enumerate() {
            let cell = self.position_to_cell(pos);
            self.cells.entry(cell).or_insert_with(Vec::new).push(i);
        }
    }

    /// Convert a position to cell coordinates
    fn position_to_cell(&self, pos: &Position) -> (i32, i32) {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cy = (pos.y / self.cell_size).floor() as i32;
        (
            cx.clamp(0, self.grid_width - 1),
            cy.clamp(0, self.grid_height - 1),
        )
    }

    /// Get indices of agents that might collide with agent at given position
    /// Only checks current cell and 8 neighbors (9 cells total)
    pub fn get_nearby(&self, pos: &Position) -> Vec<usize> {
        let (cx, cy) = self.position_to_cell(pos);
        let mut nearby = Vec::new();

        // Check current cell and all 8 neighbors
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_cell = (cx + dx, cy + dy);
                if let Some(indices) = self.cells.get(&check_cell) {
                    nearby.extend(indices.iter().copied());
                }
            }
        }

        nearby
    }

    /// Get the number of agents in the spatial hash
    pub fn agent_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }

    /// Clear all cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self::new()
    }
}

/// Collision avoidance system using spatial hash for efficient neighbor detection
#[derive(Debug)]
pub struct CollisionAvoidance {
    spatial_hash: SpatialHash,
    /// Minimum distance between agents (default: 0.08)
    pub min_distance: f32,
    /// Separation force strength (default: 0.5)
    pub separation_force: f32,
}

impl CollisionAvoidance {
    /// Create a new collision avoidance system with default parameters
    pub fn new() -> Self {
        Self {
            // Cell size should be >= 2x min_distance for correct neighbor detection
            spatial_hash: SpatialHash::with_cell_size(DEFAULT_CELL_SIZE),
            min_distance: DEFAULT_MIN_DISTANCE,
            separation_force: DEFAULT_SEPARATION_FORCE,
        }
    }

    /// Create with custom parameters
    pub fn with_params(min_distance: f32, separation_force: f32) -> Self {
        Self {
            // Cell size should be >= 2x min_distance
            spatial_hash: SpatialHash::with_cell_size(min_distance * 2.0),
            min_distance,
            separation_force,
        }
    }

    /// Apply separation forces to all agents in O(n) average time
    /// Returns the forces to be applied (does not modify positions directly)
    pub fn calculate_separation_forces(&mut self, positions: &[Position]) -> Vec<(f32, f32)> {
        // Rebuild spatial hash with current positions
        self.spatial_hash.rebuild(positions);

        // Calculate forces for each agent
        positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let mut force_x = 0.0;
                let mut force_y = 0.0;

                // Only check nearby agents (9 cells instead of all agents)
                for j in self.spatial_hash.get_nearby(pos) {
                    if j == i {
                        continue;
                    }

                    let other = &positions[j];
                    let dx = pos.x - other.x;
                    let dy = pos.y - other.y;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt();

                    // Apply separation force if within min_distance
                    if dist < self.min_distance && dist > 0.001 {
                        // Strength increases as agents get closer
                        let strength = (self.min_distance - dist) / self.min_distance;
                        force_x += (dx / dist) * strength * self.separation_force;
                        force_y += (dy / dist) * strength * self.separation_force;
                    }
                }

                (force_x, force_y)
            })
            .collect()
    }

    /// Apply separation forces directly to mutable positions slice
    pub fn apply_separation(&mut self, positions: &mut [Position]) {
        let forces = self.calculate_separation_forces(positions);

        // Apply forces with clamping
        for (i, (fx, fy)) in forces.into_iter().enumerate() {
            positions[i].x = (positions[i].x + fx).clamp(0.05, 0.95);
            positions[i].y = (positions[i].y + fy).clamp(0.05, 0.95);
        }
    }
}

impl Default for CollisionAvoidance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_rebuild() {
        let mut hash = SpatialHash::new();
        let positions = vec![
            Position::new(0.1, 0.1),
            Position::new(0.5, 0.5),
            Position::new(0.9, 0.9),
        ];

        hash.rebuild(&positions);
        assert_eq!(hash.agent_count(), 3);
    }

    #[test]
    fn test_get_nearby_finds_close_agents() {
        let mut hash = SpatialHash::new();
        let positions = vec![
            Position::new(0.1, 0.1),
            Position::new(0.12, 0.12), // Close to first
            Position::new(0.9, 0.9),   // Far from first
        ];

        hash.rebuild(&positions);
        let nearby = hash.get_nearby(&positions[0]);

        // Should find at least agents 0 and 1 (both in nearby cells)
        assert!(nearby.contains(&0));
        assert!(nearby.contains(&1));
    }

    #[test]
    fn test_collision_avoidance_separates_close_agents() {
        let mut ca = CollisionAvoidance::new();
        let mut positions = vec![
            Position::new(0.5, 0.5),
            Position::new(0.52, 0.5), // Very close (distance = 0.02 < min_distance 0.08)
        ];

        let original_dist = positions[0].distance_to(&positions[1]);
        ca.apply_separation(&mut positions);
        let new_dist = positions[0].distance_to(&positions[1]);

        // Agents should be pushed apart
        assert!(new_dist > original_dist);
    }

    #[test]
    fn test_collision_avoidance_ignores_far_agents() {
        let mut ca = CollisionAvoidance::new();
        let mut positions = vec![
            Position::new(0.2, 0.2),
            Position::new(0.8, 0.8), // Far apart (distance > min_distance)
        ];

        let original_positions = positions.clone();
        ca.apply_separation(&mut positions);

        // Positions should be unchanged (or nearly so)
        assert!((positions[0].x - original_positions[0].x).abs() < 0.001);
        assert!((positions[1].x - original_positions[1].x).abs() < 0.001);
    }
}
