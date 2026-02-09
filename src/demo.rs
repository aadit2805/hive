use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::event::{AgentStatus, AgentUpdate, Connection, HiveEvent, Landmark};

// ============================================================================
// AGENT PERSONALITIES
// ============================================================================

/// Activity style determines how an agent moves and works
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityStyle {
    Fast,    // Quick movements, high intensity bursts, short idle periods
    Steady,  // Consistent medium activity, reliable worker
    Bursty,  // Long idle periods then sudden high activity
}

/// Agent personality defining behavior patterns
#[derive(Debug, Clone)]
pub struct AgentPersonality {
    pub name: &'static str,
    pub role: &'static str,
    pub preferred_areas: &'static [&'static str],
    pub activity_style: ActivityStyle,
    pub collaboration_tendency: f32,  // 0.0-1.0 how often they connect with others
    pub base_intensity: f32,          // baseline intensity level
    pub messages: &'static [&'static str],  // context-aware messages for this role
}

/// The six demo agents with distinct personalities
const AGENT_PERSONALITIES: [AgentPersonality; 6] = [
    AgentPersonality {
        name: "Atlas",
        role: "Backend Specialist",
        preferred_areas: &["api", "database", "schema", "query", "model", "endpoint"],
        activity_style: ActivityStyle::Steady,
        collaboration_tendency: 0.3,
        base_intensity: 0.5,
        messages: &[
            "Optimizing query performance",
            "Schema migration in progress",
            "Refactoring data access layer",
            "Indexing database tables",
            "Reviewing API contracts",
            "Tuning connection pool",
        ],
    },
    AgentPersonality {
        name: "Nova",
        role: "Frontend Explorer",
        preferred_areas: &["frontend", "react", "component", "ui", "style", "layout"],
        activity_style: ActivityStyle::Fast,
        collaboration_tendency: 0.8,
        base_intensity: 0.7,
        messages: &[
            "Building new component",
            "Styling user interface",
            "Optimizing render cycle",
            "Testing responsiveness",
            "Exploring design patterns",
            "Refining user experience",
        ],
    },
    AgentPersonality {
        name: "Echo",
        role: "Quality Tester",
        preferred_areas: &["test", "unit", "integration", "mock", "coverage", "debug"],
        activity_style: ActivityStyle::Bursty,
        collaboration_tendency: 0.4,
        base_intensity: 0.4,
        messages: &[
            "Running test suite",
            "Analyzing test coverage",
            "Found edge case issue",
            "Validating error handling",
            "Checking regression tests",
            "Investigating flaky test",
        ],
    },
    AgentPersonality {
        name: "Cipher",
        role: "Security Specialist",
        preferred_areas: &["auth", "jwt", "session", "login", "permission", "security"],
        activity_style: ActivityStyle::Steady,
        collaboration_tendency: 0.2,
        base_intensity: 0.45,
        messages: &[
            "Auditing access controls",
            "Validating JWT tokens",
            "Reviewing auth flow",
            "Checking permission matrix",
            "Scanning for vulnerabilities",
            "Hardening session management",
        ],
    },
    AgentPersonality {
        name: "Flux",
        role: "DevOps Engineer",
        preferred_areas: &["deploy", "docker", "ci", "kubernetes", "pipeline", "infra"],
        activity_style: ActivityStyle::Fast,
        collaboration_tendency: 0.6,
        base_intensity: 0.6,
        messages: &[
            "Configuring deployment",
            "Building container image",
            "Updating CI pipeline",
            "Scaling infrastructure",
            "Monitoring health checks",
            "Optimizing build times",
        ],
    },
    AgentPersonality {
        name: "Sage",
        role: "Architecture Planner",
        preferred_areas: &["architecture", "design", "pattern", "planning", "review"],
        activity_style: ActivityStyle::Bursty,
        collaboration_tendency: 0.5,
        base_intensity: 0.3,
        messages: &[
            "Reviewing system design",
            "Planning module structure",
            "Analyzing dependencies",
            "Documenting architecture",
            "Evaluating trade-offs",
            "Proposing improvements",
        ],
    },
];

// ============================================================================
// NARRATIVE PHASES
// ============================================================================

/// Narrative phases for structured demo progression
#[derive(Debug, Clone, Copy, PartialEq)]
enum NarrativePhase {
    Exploration,    // Agents spread out, exploring different areas
    Discovery,      // Some agents find interesting things, start focusing
    Collaboration,  // Agents begin connecting and working together
    Resolution,     // Work concludes, agents disperse to new tasks
}

impl NarrativePhase {
    fn duration_range(&self) -> (u64, u64) {
        match self {
            NarrativePhase::Exploration => (8000, 12000),   // 8-12 seconds
            NarrativePhase::Discovery => (6000, 10000),     // 6-10 seconds
            NarrativePhase::Collaboration => (10000, 15000), // 10-15 seconds
            NarrativePhase::Resolution => (5000, 8000),     // 5-8 seconds
        }
    }

    fn next(&self) -> Self {
        match self {
            NarrativePhase::Exploration => NarrativePhase::Discovery,
            NarrativePhase::Discovery => NarrativePhase::Collaboration,
            NarrativePhase::Collaboration => NarrativePhase::Resolution,
            NarrativePhase::Resolution => NarrativePhase::Exploration,
        }
    }
}

// ============================================================================
// SWARM STATE
// ============================================================================

/// State for managing gradual swarm convergence
struct SwarmState {
    is_active: bool,
    buildup_progress: f32,  // 0.0 to 1.0
    target_area: Option<usize>,
    converged_agents: Vec<usize>,
    resolution_progress: f32,
}

impl SwarmState {
    fn new() -> Self {
        Self {
            is_active: false,
            buildup_progress: 0.0,
            target_area: None,
            converged_agents: Vec::new(),
            resolution_progress: 0.0,
        }
    }

    fn start(&mut self, target_area: usize) {
        self.is_active = true;
        self.buildup_progress = 0.0;
        self.target_area = Some(target_area);
        self.converged_agents.clear();
        self.resolution_progress = 0.0;
    }

    fn is_building_up(&self) -> bool {
        self.is_active && self.buildup_progress < 1.0
    }

    fn is_resolving(&self) -> bool {
        self.is_active && self.buildup_progress >= 1.0 && self.resolution_progress > 0.0
    }
}

// ============================================================================
// CONTEXT-AWARE MESSAGES
// ============================================================================

/// Get a context-aware message based on agent's current focus area
fn get_contextual_message(personality: &AgentPersonality, focus: &[String], rng: &mut StdRng) -> String {
    // Check if focus matches agent's preferred areas - use their specialized messages
    let focus_matches_preferred = focus.iter().any(|f| {
        personality.preferred_areas.iter().any(|p| f.contains(p) || p.contains(f.as_str()))
    });

    if focus_matches_preferred {
        // Use personality-specific messages
        return personality.messages[rng.gen_range(0..personality.messages.len())].to_string();
    }

    // Otherwise, generate focus-specific messages based on the area
    let focus_str = focus.first().map(|s| s.as_str()).unwrap_or("");

    let messages: &[&str] = match focus_str {
        s if s.contains("auth") || s.contains("jwt") || s.contains("login") => &[
            "Reviewing authentication flow",
            "Checking JWT validation",
            "Auditing session handling",
            "Validating credentials",
        ],
        s if s.contains("database") || s.contains("schema") || s.contains("query") => &[
            "Analyzing query patterns",
            "Reviewing schema design",
            "Optimizing data access",
            "Checking index usage",
        ],
        s if s.contains("frontend") || s.contains("react") || s.contains("ui") => &[
            "Inspecting component tree",
            "Checking render performance",
            "Reviewing state management",
            "Analyzing UI patterns",
        ],
        s if s.contains("api") || s.contains("endpoint") => &[
            "Mapping API routes",
            "Reviewing endpoint contracts",
            "Checking request handlers",
            "Validating response formats",
        ],
        s if s.contains("test") || s.contains("unit") => &[
            "Examining test cases",
            "Reviewing test coverage",
            "Checking assertions",
            "Analyzing test patterns",
        ],
        s if s.contains("deploy") || s.contains("docker") || s.contains("ci") => &[
            "Reviewing deployment config",
            "Checking container setup",
            "Analyzing pipeline stages",
            "Validating infrastructure",
        ],
        s if s.contains("cache") || s.contains("redis") => &[
            "Analyzing cache patterns",
            "Reviewing cache keys",
            "Checking cache invalidation",
            "Optimizing cache usage",
        ],
        s if s.contains("logging") || s.contains("error") => &[
            "Reviewing error handling",
            "Analyzing log patterns",
            "Checking error boundaries",
            "Validating error messages",
        ],
        _ => &[
            "Exploring code patterns",
            "Analyzing structure",
            "Reviewing implementation",
            "Checking dependencies",
        ],
    };

    messages[rng.gen_range(0..messages.len())].to_string()
}

// ============================================================================
// CONNECTION LABELS
// ============================================================================

/// Get meaningful connection labels based on the context
fn get_connection_label(
    from_personality: &AgentPersonality,
    to_personality: &AgentPersonality,
    rng: &mut StdRng,
) -> String {
    // Specific collaboration patterns between agent types
    let labels: &[&str] = match (from_personality.role, to_personality.role) {
        ("Backend Specialist", "Frontend Explorer") => &[
            "API contract review",
            "data format sync",
            "endpoint validation",
        ],
        ("Frontend Explorer", "Backend Specialist") => &[
            "requesting data shape",
            "query optimization ask",
            "API feedback",
        ],
        ("Quality Tester", _) => &[
            "found test case",
            "coverage report",
            "regression check",
        ],
        (_, "Quality Tester") => &[
            "needs testing",
            "review test plan",
            "edge case found",
        ],
        ("Security Specialist", _) => &[
            "security review",
            "auth validation",
            "permission check",
        ],
        (_, "Security Specialist") => &[
            "needs security review",
            "auth question",
            "access check",
        ],
        ("DevOps Engineer", _) => &[
            "deploy config",
            "infra update",
            "pipeline change",
        ],
        (_, "DevOps Engineer") => &[
            "needs deployment",
            "env config ask",
            "build help",
        ],
        ("Architecture Planner", _) => &[
            "design guidance",
            "pattern suggestion",
            "review request",
        ],
        (_, "Architecture Planner") => &[
            "design question",
            "architecture review",
            "pattern advice",
        ],
        _ => &[
            "sharing findings",
            "coordinating work",
            "syncing progress",
            "knowledge transfer",
        ],
    };

    labels[rng.gen_range(0..labels.len())].to_string()
}

/// Get swarm-specific connection labels during convergence
fn get_swarm_connection_label(focus_area: &str, rng: &mut StdRng) -> String {
    let area_labels: &[&str] = match focus_area {
        s if s.contains("auth") => &[
            "auth issue found",
            "security concern",
            "credential problem",
        ],
        s if s.contains("database") => &[
            "data integrity issue",
            "query bottleneck",
            "schema conflict",
        ],
        s if s.contains("frontend") => &[
            "UI regression",
            "render issue",
            "component bug",
        ],
        s if s.contains("api") => &[
            "API breaking change",
            "endpoint failure",
            "contract violation",
        ],
        s if s.contains("test") => &[
            "test failure cascade",
            "coverage gap",
            "critical regression",
        ],
        s if s.contains("deploy") => &[
            "deployment blocker",
            "infra issue",
            "pipeline failure",
        ],
        _ => &[
            "critical issue found",
            "needs collaboration",
            "converging on problem",
        ],
    };

    area_labels[rng.gen_range(0..area_labels.len())].to_string()
}

// ============================================================================
// TIMING UTILITIES
// ============================================================================

/// Get update interval based on personality's activity style
fn get_update_interval(style: ActivityStyle, rng: &mut StdRng) -> Duration {
    let (min_ms, max_ms) = match style {
        ActivityStyle::Fast => (500, 900),
        ActivityStyle::Steady => (800, 1200),
        ActivityStyle::Bursty => (1000, 1500),
    };
    Duration::from_millis(rng.gen_range(min_ms..max_ms))
}

/// Get intensity based on activity style and phase
fn get_intensity(
    personality: &AgentPersonality,
    phase: NarrativePhase,
    rng: &mut StdRng,
) -> f32 {
    let base = personality.base_intensity;

    let phase_modifier = match phase {
        NarrativePhase::Exploration => 0.7,
        NarrativePhase::Discovery => 1.0,
        NarrativePhase::Collaboration => 1.2,
        NarrativePhase::Resolution => 0.6,
    };

    let style_variance = match personality.activity_style {
        ActivityStyle::Fast => rng.gen_range(0.2..0.4),
        ActivityStyle::Steady => rng.gen_range(0.05..0.15),
        ActivityStyle::Bursty => if rng.gen_bool(0.3) { rng.gen_range(0.4..0.6) } else { rng.gen_range(-0.2..0.1) },
    };

    ((base + style_variance) * phase_modifier).clamp(0.1, 1.0)
}

/// Get status based on activity style and phase
fn get_status(
    personality: &AgentPersonality,
    phase: NarrativePhase,
    rng: &mut StdRng,
) -> AgentStatus {
    match personality.activity_style {
        ActivityStyle::Fast => {
            match rng.gen_range(0..10) {
                0..=7 => AgentStatus::Active,
                8 => AgentStatus::Thinking,
                _ => AgentStatus::Idle,
            }
        }
        ActivityStyle::Steady => {
            match (phase, rng.gen_range(0..10)) {
                (NarrativePhase::Exploration, 0..=4) => AgentStatus::Active,
                (NarrativePhase::Exploration, 5..=7) => AgentStatus::Thinking,
                (NarrativePhase::Exploration, _) => AgentStatus::Idle,
                (NarrativePhase::Discovery, 0..=6) => AgentStatus::Active,
                (NarrativePhase::Discovery, _) => AgentStatus::Thinking,
                (NarrativePhase::Collaboration, 0..=7) => AgentStatus::Active,
                (NarrativePhase::Collaboration, _) => AgentStatus::Thinking,
                (NarrativePhase::Resolution, 0..=3) => AgentStatus::Active,
                (NarrativePhase::Resolution, 4..=6) => AgentStatus::Thinking,
                (NarrativePhase::Resolution, _) => AgentStatus::Idle,
            }
        }
        ActivityStyle::Bursty => {
            if rng.gen_bool(0.7) {
                // Long idle periods
                match rng.gen_range(0..10) {
                    0..=1 => AgentStatus::Active,
                    2..=4 => AgentStatus::Thinking,
                    5..=6 => AgentStatus::Waiting,
                    _ => AgentStatus::Idle,
                }
            } else {
                // Burst of activity
                match rng.gen_range(0..10) {
                    0..=7 => AgentStatus::Active,
                    _ => AgentStatus::Thinking,
                }
            }
        }
    }
}

// ============================================================================
// FOCUS AREAS
// ============================================================================

/// All possible focus areas for the demo
const FOCUS_AREAS: [[&str; 2]; 8] = [
    ["authentication", "jwt"],
    ["database", "schema"],
    ["frontend", "react"],
    ["api", "endpoints"],
    ["testing", "unit"],
    ["deploy", "docker"],
    ["cache", "redis"],
    ["logging", "errors"],
];

/// Get focus area based on personality preferences
fn get_focus_for_personality(
    personality: &AgentPersonality,
    phase: NarrativePhase,
    rng: &mut StdRng,
) -> Vec<String> {
    // During exploration, agents stick more to their preferred areas
    // During collaboration, they might venture to other areas
    let prefer_own_area = match phase {
        NarrativePhase::Exploration => 0.9,
        NarrativePhase::Discovery => 0.7,
        NarrativePhase::Collaboration => 0.5,
        NarrativePhase::Resolution => 0.8,
    };

    if rng.gen_bool(prefer_own_area) {
        // Find a focus area that overlaps with preferred areas
        let matching_areas: Vec<_> = FOCUS_AREAS.iter()
            .filter(|area| {
                area.iter().any(|kw| {
                    personality.preferred_areas.iter().any(|p| kw.contains(p) || p.contains(*kw))
                })
            })
            .collect();

        if !matching_areas.is_empty() {
            let area = matching_areas[rng.gen_range(0..matching_areas.len())];
            return area.iter().map(|s| s.to_string()).collect();
        }
    }

    // Random area
    let idx = rng.gen_range(0..FOCUS_AREAS.len());
    FOCUS_AREAS[idx].iter().map(|s| s.to_string()).collect()
}

// ============================================================================
// DEMO EVENT GENERATION
// ============================================================================

/// Generate demo events continuously with improved pacing and personalities
pub async fn generate_demo_events(tx: mpsc::Sender<HiveEvent>) {
    let mut rng = StdRng::from_entropy();

    // First, create landmarks
    let landmarks = [
        ("auth-zone", "Authentication", vec!["auth", "jwt", "session", "login"]),
        ("data-zone", "Database", vec!["database", "schema", "query", "model"]),
        ("ui-zone", "Frontend", vec!["frontend", "react", "component", "ui"]),
        ("api-zone", "API Layer", vec!["api", "endpoint", "rest", "handler"]),
        ("test-zone", "Testing", vec!["test", "unit", "integration", "mock"]),
        ("ops-zone", "DevOps", vec!["deploy", "docker", "ci", "kubernetes"]),
    ];

    for (id, label, keywords) in landmarks {
        let event = HiveEvent::Landmark(Landmark {
            id: id.to_string(),
            label: label.to_string(),
            keywords: keywords.into_iter().map(String::from).collect(),
            timestamp: current_timestamp(),
        });

        if tx.send(event).await.is_err() {
            return;
        }
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Initialize agents with their personalities
    for (i, personality) in AGENT_PERSONALITIES.iter().enumerate() {
        let focus = get_focus_for_personality(personality, NarrativePhase::Exploration, &mut rng);
        let event = HiveEvent::AgentUpdate(AgentUpdate {
            agent_id: personality.name.to_string(),
            status: AgentStatus::Idle,
            focus,
            intensity: 0.1,
            message: format!("{} starting up...", personality.role),
            timestamp: current_timestamp(),
        });

        if tx.send(event).await.is_err() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(300 + (i as u64 * 100))).await;
    }

    // State tracking
    let mut phase = NarrativePhase::Exploration;
    let mut phase_start = std::time::Instant::now();
    let mut phase_duration = Duration::from_millis(rng.gen_range(
        phase.duration_range().0..phase.duration_range().1
    ));
    let mut swarm_state = SwarmState::new();
    let mut cycles_since_swarm: u32 = 0;
    let mut last_agent_idx: usize = 0;

    // Main demo loop
    loop {
        // Check for phase transition
        if phase_start.elapsed() >= phase_duration {
            phase = phase.next();
            phase_start = std::time::Instant::now();
            phase_duration = Duration::from_millis(rng.gen_range(
                phase.duration_range().0..phase.duration_range().1
            ));
        }

        // Handle swarm moments (every ~90 seconds, or 3 full narrative cycles)
        cycles_since_swarm += 1;
        let should_start_swarm = cycles_since_swarm > 90 && phase == NarrativePhase::Discovery && rng.gen_bool(0.1);

        if should_start_swarm && !swarm_state.is_active {
            let target_area = rng.gen_range(0..FOCUS_AREAS.len());
            swarm_state.start(target_area);
            cycles_since_swarm = 0;
        }

        // Handle active swarm
        if swarm_state.is_active {
            if let Err(_) = handle_swarm_update(&tx, &mut swarm_state, &mut rng).await {
                return;
            }

            // Check if swarm is complete
            if swarm_state.resolution_progress >= 1.0 {
                swarm_state.is_active = false;
            }

            tokio::time::sleep(Duration::from_millis(400)).await;
            continue;
        }

        // Regular agent updates - update 1-2 agents per cycle
        let num_updates = if phase == NarrativePhase::Collaboration { 2 } else { 1 };

        for _ in 0..num_updates {
            // Round-robin with some randomness for variety
            let agent_idx = if rng.gen_bool(0.7) {
                last_agent_idx = (last_agent_idx + 1) % AGENT_PERSONALITIES.len();
                last_agent_idx
            } else {
                rng.gen_range(0..AGENT_PERSONALITIES.len())
            };

            let personality = &AGENT_PERSONALITIES[agent_idx];
            let focus = get_focus_for_personality(personality, phase, &mut rng);
            let status = get_status(personality, phase, &mut rng);
            let intensity = get_intensity(personality, phase, &mut rng);
            let message = get_contextual_message(personality, &focus, &mut rng);

            let event = HiveEvent::AgentUpdate(AgentUpdate {
                agent_id: personality.name.to_string(),
                status,
                focus,
                intensity,
                message,
                timestamp: current_timestamp(),
            });

            if tx.send(event).await.is_err() {
                return;
            }

            // Variable sleep based on personality
            let interval = get_update_interval(personality.activity_style, &mut rng);
            tokio::time::sleep(interval).await;
        }

        // Connections based on phase and personality
        if phase == NarrativePhase::Collaboration || phase == NarrativePhase::Discovery {
            let from_idx = rng.gen_range(0..AGENT_PERSONALITIES.len());
            let from_personality = &AGENT_PERSONALITIES[from_idx];

            // Check if this agent wants to collaborate
            if rng.gen_bool(from_personality.collaboration_tendency as f64) {
                let mut to_idx = rng.gen_range(0..AGENT_PERSONALITIES.len());
                while to_idx == from_idx {
                    to_idx = rng.gen_range(0..AGENT_PERSONALITIES.len());
                }
                let to_personality = &AGENT_PERSONALITIES[to_idx];

                let label = get_connection_label(from_personality, to_personality, &mut rng);

                let event = HiveEvent::Connection(Connection {
                    from: from_personality.name.to_string(),
                    to: to_personality.name.to_string(),
                    label,
                    timestamp: current_timestamp(),
                });

                if tx.send(event).await.is_err() {
                    return;
                }
            }
        }

        // Base sleep between cycles (reduced from original)
        tokio::time::sleep(Duration::from_millis(rng.gen_range(300..600))).await;
    }
}

/// Handle swarm updates with gradual buildup
async fn handle_swarm_update(
    tx: &mpsc::Sender<HiveEvent>,
    state: &mut SwarmState,
    rng: &mut StdRng,
) -> Result<(), ()> {
    let target_area = state.target_area.unwrap_or(0);
    let converge_focus: Vec<String> = FOCUS_AREAS[target_area].iter().map(|s| s.to_string()).collect();
    let focus_str = converge_focus.first().map(|s| s.as_str()).unwrap_or("issue");

    if state.is_building_up() {
        // Gradual buildup phase - agents converge one at a time
        state.buildup_progress += 0.15; // ~7 steps to full convergence

        // Add one agent to the converging group
        if state.converged_agents.len() < AGENT_PERSONALITIES.len() {
            // Pick an agent that hasn't converged yet
            let remaining: Vec<usize> = (0..AGENT_PERSONALITIES.len())
                .filter(|i| !state.converged_agents.contains(i))
                .collect();

            if !remaining.is_empty() {
                let next_agent = remaining[rng.gen_range(0..remaining.len())];
                state.converged_agents.push(next_agent);

                let personality = &AGENT_PERSONALITIES[next_agent];

                // Update the newly converging agent
                let intensity = 0.6 + state.buildup_progress * 0.4;
                let message = format!("Investigating {} issue...", focus_str);

                let event = HiveEvent::AgentUpdate(AgentUpdate {
                    agent_id: personality.name.to_string(),
                    status: AgentStatus::Active,
                    focus: converge_focus.clone(),
                    intensity,
                    message,
                    timestamp: current_timestamp(),
                });

                tx.send(event).await.map_err(|_| ())?;

                // Create a connection to a random already-converged agent
                if state.converged_agents.len() > 1 {
                    let other_idx = state.converged_agents[rng.gen_range(0..state.converged_agents.len() - 1)];
                    let other_personality = &AGENT_PERSONALITIES[other_idx];

                    let label = get_swarm_connection_label(focus_str, rng);

                    let event = HiveEvent::Connection(Connection {
                        from: personality.name.to_string(),
                        to: other_personality.name.to_string(),
                        label,
                        timestamp: current_timestamp(),
                    });

                    tx.send(event).await.map_err(|_| ())?;
                }
            }
        }

        // Keep existing converged agents active
        for &idx in &state.converged_agents[..state.converged_agents.len().saturating_sub(1)] {
            let personality = &AGENT_PERSONALITIES[idx];
            let intensity = 0.7 + state.buildup_progress * 0.3;

            let event = HiveEvent::AgentUpdate(AgentUpdate {
                agent_id: personality.name.to_string(),
                status: AgentStatus::Active,
                focus: converge_focus.clone(),
                intensity,
                message: "Collaborating on issue".to_string(),
                timestamp: current_timestamp(),
            });

            tx.send(event).await.map_err(|_| ())?;
        }
    } else if state.buildup_progress >= 1.0 && state.resolution_progress < 1.0 {
        // Hold at peak for a moment, then start resolution
        if state.resolution_progress == 0.0 {
            // Peak moment - all agents fully engaged
            for (idx, personality) in AGENT_PERSONALITIES.iter().enumerate() {
                let event = HiveEvent::AgentUpdate(AgentUpdate {
                    agent_id: personality.name.to_string(),
                    status: AgentStatus::Active,
                    focus: converge_focus.clone(),
                    intensity: rng.gen_range(0.85..1.0),
                    message: "Critical issue identified!".to_string(),
                    timestamp: current_timestamp(),
                });

                tx.send(event).await.map_err(|_| ())?;

                // Create mesh of connections
                if idx > 0 {
                    let other = &AGENT_PERSONALITIES[rng.gen_range(0..idx)];
                    let event = HiveEvent::Connection(Connection {
                        from: personality.name.to_string(),
                        to: other.name.to_string(),
                        label: "working together".to_string(),
                        timestamp: current_timestamp(),
                    });
                    tx.send(event).await.map_err(|_| ())?;
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
            state.resolution_progress = 0.1;
        } else {
            // Gradual dispersion
            state.resolution_progress += 0.2;

            // Agents gradually return to their preferred areas
            let num_dispersing = (state.resolution_progress * AGENT_PERSONALITIES.len() as f32) as usize;

            for (idx, personality) in AGENT_PERSONALITIES.iter().enumerate() {
                if idx < num_dispersing {
                    // This agent is dispersing back to normal work
                    let focus = get_focus_for_personality(personality, NarrativePhase::Resolution, rng);
                    let intensity = 0.3 + rng.gen_range(0.0..0.2);

                    let event = HiveEvent::AgentUpdate(AgentUpdate {
                        agent_id: personality.name.to_string(),
                        status: AgentStatus::Thinking,
                        focus,
                        intensity,
                        message: "Issue resolved, returning to work".to_string(),
                        timestamp: current_timestamp(),
                    });

                    tx.send(event).await.map_err(|_| ())?;
                } else {
                    // Still on the issue but winding down
                    let intensity = 0.5 + (1.0 - state.resolution_progress) * 0.3;

                    let event = HiveEvent::AgentUpdate(AgentUpdate {
                        agent_id: personality.name.to_string(),
                        status: AgentStatus::Active,
                        focus: converge_focus.clone(),
                        intensity,
                        message: "Wrapping up issue work".to_string(),
                        timestamp: current_timestamp(),
                    });

                    tx.send(event).await.map_err(|_| ())?;
                }
            }
        }
    }

    Ok(())
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_personalities_valid() {
        for personality in &AGENT_PERSONALITIES {
            assert!(!personality.name.is_empty());
            assert!(!personality.preferred_areas.is_empty());
            assert!(personality.collaboration_tendency >= 0.0 && personality.collaboration_tendency <= 1.0);
            assert!(personality.base_intensity >= 0.0 && personality.base_intensity <= 1.0);
            assert!(!personality.messages.is_empty());
        }
    }

    #[test]
    fn test_narrative_phase_cycle() {
        let mut phase = NarrativePhase::Exploration;
        phase = phase.next();
        assert_eq!(phase, NarrativePhase::Discovery);
        phase = phase.next();
        assert_eq!(phase, NarrativePhase::Collaboration);
        phase = phase.next();
        assert_eq!(phase, NarrativePhase::Resolution);
        phase = phase.next();
        assert_eq!(phase, NarrativePhase::Exploration);
    }

    #[test]
    fn test_get_intensity_clamped() {
        let mut rng = StdRng::seed_from_u64(42);
        for personality in &AGENT_PERSONALITIES {
            for _ in 0..100 {
                let intensity = get_intensity(personality, NarrativePhase::Collaboration, &mut rng);
                assert!(intensity >= 0.1 && intensity <= 1.0);
            }
        }
    }

    #[test]
    fn test_contextual_messages() {
        let mut rng = StdRng::seed_from_u64(42);
        let personality = &AGENT_PERSONALITIES[0]; // Atlas

        // Test with preferred focus
        let focus = vec!["database".to_string(), "query".to_string()];
        let msg = get_contextual_message(personality, &focus, &mut rng);
        assert!(!msg.is_empty());

        // Test with non-preferred focus
        let focus = vec!["frontend".to_string(), "react".to_string()];
        let msg = get_contextual_message(personality, &focus, &mut rng);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_activity_style_intervals() {
        let mut rng = StdRng::seed_from_u64(42);

        let fast_interval = get_update_interval(ActivityStyle::Fast, &mut rng);
        let steady_interval = get_update_interval(ActivityStyle::Steady, &mut rng);
        let bursty_interval = get_update_interval(ActivityStyle::Bursty, &mut rng);

        // Fast should generally be shorter than bursty
        assert!(fast_interval.as_millis() >= 500 && fast_interval.as_millis() < 900);
        assert!(steady_interval.as_millis() >= 800 && steady_interval.as_millis() < 1200);
        assert!(bursty_interval.as_millis() >= 1000 && bursty_interval.as_millis() < 1500);
    }
}
