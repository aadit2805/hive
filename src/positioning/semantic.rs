use std::collections::HashMap;

use super::Position;
use crate::state::field::StoredLandmark;
use crate::event::LandmarkId;

/// Semantic positioning engine that maps keywords to 2D positions
pub struct SemanticPositioner {
    /// Cached keyword positions
    keyword_cache: HashMap<String, Position>,
    /// Predefined concept clusters
    concept_clusters: Vec<ConceptCluster>,
}

/// A predefined concept cluster for semantic positioning
#[derive(Debug, Clone)]
struct ConceptCluster {
    center: Position,
    keywords: Vec<String>,
    radius: f32,
}

impl SemanticPositioner {
    pub fn new() -> Self {
        let mut positioner = Self {
            keyword_cache: HashMap::new(),
            concept_clusters: Vec::new(),
        };

        // Initialize default concept clusters
        positioner.init_default_clusters();

        positioner
    }

    /// Initialize predefined concept clusters for common programming domains
    fn init_default_clusters(&mut self) {
        // Top-left: Frontend/UI
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.2, 0.2),
            keywords: vec![
                "frontend", "ui", "css", "html", "react", "vue", "angular",
                "component", "button", "form", "layout", "style", "design",
            ].into_iter().map(String::from).collect(),
            radius: 0.15,
        });

        // Top-right: Backend/API
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.8, 0.2),
            keywords: vec![
                "backend", "api", "rest", "graphql", "endpoint", "server",
                "route", "controller", "middleware", "http", "request",
            ].into_iter().map(String::from).collect(),
            radius: 0.15,
        });

        // Bottom-left: Database
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.2, 0.8),
            keywords: vec![
                "database", "sql", "postgres", "mysql", "mongodb", "redis",
                "query", "schema", "migration", "model", "table", "index",
            ].into_iter().map(String::from).collect(),
            radius: 0.15,
        });

        // Bottom-right: Infrastructure
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.8, 0.8),
            keywords: vec![
                "docker", "kubernetes", "deploy", "ci", "cd", "pipeline",
                "aws", "cloud", "terraform", "infrastructure", "devops",
            ].into_iter().map(String::from).collect(),
            radius: 0.15,
        });

        // Center-top: Authentication
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.5, 0.15),
            keywords: vec![
                "auth", "authentication", "jwt", "oauth", "session", "login",
                "password", "token", "security", "permission", "role",
            ].into_iter().map(String::from).collect(),
            radius: 0.12,
        });

        // Center-bottom: Testing
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.5, 0.85),
            keywords: vec![
                "test", "testing", "unit", "integration", "e2e", "mock",
                "jest", "pytest", "spec", "coverage", "assertion",
            ].into_iter().map(String::from).collect(),
            radius: 0.12,
        });

        // Left-center: State/Data
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.15, 0.5),
            keywords: vec![
                "state", "store", "redux", "context", "data", "cache",
                "memory", "storage", "persist", "sync",
            ].into_iter().map(String::from).collect(),
            radius: 0.12,
        });

        // Right-center: Logic/Business
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.85, 0.5),
            keywords: vec![
                "logic", "business", "service", "handler", "processor",
                "workflow", "validation", "rule", "algorithm",
            ].into_iter().map(String::from).collect(),
            radius: 0.12,
        });

        // Center: Core/Main
        self.concept_clusters.push(ConceptCluster {
            center: Position::new(0.5, 0.5),
            keywords: vec![
                "main", "core", "app", "init", "config", "setup",
                "entry", "root", "base",
            ].into_iter().map(String::from).collect(),
            radius: 0.1,
        });
    }

    /// Calculate position for a set of focus keywords
    pub fn calculate_position(
        &mut self,
        focus: &[String],
        landmarks: &HashMap<LandmarkId, StoredLandmark>,
    ) -> Position {
        if focus.is_empty() {
            return Position::new(0.5, 0.5);
        }

        let mut total_weight = 0.0;
        let mut weighted_x = 0.0;
        let mut weighted_y = 0.0;

        for keyword in focus {
            let kw_lower = keyword.to_lowercase();

            // Check landmarks first
            let mut found_landmark = false;
            for landmark in landmarks.values() {
                if landmark.keywords.iter().any(|k| k.to_lowercase() == kw_lower) {
                    weighted_x += landmark.position.x;
                    weighted_y += landmark.position.y;
                    total_weight += 1.0;
                    found_landmark = true;
                    break;
                }
            }

            if found_landmark {
                continue;
            }

            // Check cache
            if let Some(pos) = self.keyword_cache.get(&kw_lower) {
                weighted_x += pos.x;
                weighted_y += pos.y;
                total_weight += 1.0;
                continue;
            }

            // Calculate position from concept clusters
            let pos = self.keyword_to_position(&kw_lower);
            self.keyword_cache.insert(kw_lower, pos.clone());

            weighted_x += pos.x;
            weighted_y += pos.y;
            total_weight += 1.0;
        }

        if total_weight > 0.0 {
            Position::new(weighted_x / total_weight, weighted_y / total_weight).clamp()
        } else {
            Position::new(0.5, 0.5)
        }
    }

    /// Map a single keyword to a position
    fn keyword_to_position(&self, keyword: &str) -> Position {
        // Check concept clusters for matches
        let mut best_cluster: Option<&ConceptCluster> = None;
        let mut best_score = 0.0;

        for cluster in &self.concept_clusters {
            for cluster_keyword in &cluster.keywords {
                // Check for exact match or substring match
                let score = if keyword == cluster_keyword {
                    1.0
                } else if keyword.contains(cluster_keyword) || cluster_keyword.contains(keyword) {
                    0.5
                } else {
                    0.0
                };

                if score > best_score {
                    best_score = score;
                    best_cluster = Some(cluster);
                }
            }
        }

        if let Some(cluster) = best_cluster {
            // Add some variation within the cluster
            let hash = hash_string(keyword);
            let angle = (hash % 360) as f32 * std::f32::consts::PI / 180.0;
            let distance = ((hash / 360) % 100) as f32 / 100.0 * cluster.radius * 0.8;

            Position::new(
                cluster.center.x + angle.cos() * distance,
                cluster.center.y + angle.sin() * distance,
            )
            .clamp()
        } else {
            // No cluster match - use hash-based positioning
            let hash = hash_string(keyword);
            let x = ((hash % 1000) as f32 / 1000.0) * 0.7 + 0.15;
            let y = (((hash / 1000) % 1000) as f32 / 1000.0) * 0.7 + 0.15;
            Position::new(x, y)
        }
    }

    /// Register a landmark and return its position
    pub fn register_landmark(&mut self, keywords: &[String]) -> Position {
        if keywords.is_empty() {
            return Position::new(0.5, 0.5);
        }

        // Average the positions of all keywords
        let mut x_sum = 0.0;
        let mut y_sum = 0.0;

        for keyword in keywords {
            let pos = self.keyword_to_position(&keyword.to_lowercase());
            x_sum += pos.x;
            y_sum += pos.y;
        }

        Position::new(x_sum / keywords.len() as f32, y_sum / keywords.len() as f32).clamp()
    }
}

/// Simple hash function for strings
fn hash_string(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for c in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u32);
    }
    hash
}

impl Default for SemanticPositioner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similar_keywords_cluster() {
        let mut positioner = SemanticPositioner::new();
        let landmarks = HashMap::new();

        let pos1 = positioner.calculate_position(&["react".to_string()], &landmarks);
        let pos2 = positioner.calculate_position(&["vue".to_string()], &landmarks);
        let pos3 = positioner.calculate_position(&["database".to_string()], &landmarks);

        // React and Vue should be closer to each other than to database
        let dist_react_vue = pos1.distance_to(&pos2);
        let dist_react_db = pos1.distance_to(&pos3);

        assert!(dist_react_vue < dist_react_db);
    }
}
