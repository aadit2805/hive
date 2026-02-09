# Hive UI Improvement Plan

## Executive Summary

The Hive visualization is a real-time AI agent monitoring system that represents agents as players on a field, showing their positions, activities, and interactions. While the core concept is powerful, the current implementation suffers from visual clarity issues, information overload, and limited interactivity that reduce its effectiveness as a monitoring tool.

**Vision:** Transform Hive into a professional-grade, accessible visualization that makes complex multi-agent systems instantly comprehensible. The improved UI will:

1. **Differentiate at a glance** - Status and identity must be immediately clear through redundant visual signals
2. **Reduce cognitive load** - Less noise, better layering, on-demand detail
3. **Support real workflows** - Filtering, focusing, and drilling down into agent behavior
4. **Scale gracefully** - Work with 4 agents or 40, quiet moments or "swarm storms"
5. **Fail gracefully** - Handle errors, edge cases, and accessibility needs robustly

The plan is organized into 6 phases, each building on the previous, with clear success criteria and implementation guidance.

---

## Design Principles

These principles guide all decisions in this improvement plan:

### 1. Redundant Encoding (WCAG Compliance)
Never rely on a single visual channel. Every status uses at least 3 signals:
- **Color** - Hue/saturation for quick scanning
- **Shape** - Symbol/icon for colorblind accessibility
- **Animation** - Motion/pulse for attention

### 2. Progressive Disclosure
Show overview by default, details on demand:
- **Glance level** - Agents, zones, general activity
- **Scan level** - Status, connections, recent events
- **Focus level** - Full agent details, message history

### 3. Visual Hierarchy
Layer information by importance:
1. **Critical** - Errors, blocked agents (always visible)
2. **Active** - Currently working agents and their connections
3. **Context** - Zones, trails, heatmap (background)
4. **Chrome** - UI elements, help, status bar

### 4. Calm Technology
Minimize distraction while preserving awareness:
- Animate only when information is new
- Use color sparingly for emphasis
- Prefer subtle indicators over flashy effects

### 5. Performance Budget
Smooth rendering is non-negotiable:
- Target 60 FPS on modern terminals
- Degrade gracefully (reduce effects before dropping frames)
- Batch rendering operations

### 6. Graceful Degradation
Always provide fallbacks:
- ASCII fallbacks for all Unicode symbols
- Monochrome mode for limited color support
- Keyboard-only navigation for non-mouse environments
- Minimum terminal size handling

---

## Error Handling & Graceful Degradation

This section defines how Hive handles failures and edge cases robustly.

### Terminal Environment Detection

```rust
// File: /Users/aaditshah/Documents/hive/src/terminal/capabilities.rs (NEW)

/// Detected terminal capabilities
pub struct TerminalCapabilities {
    pub supports_unicode: bool,
    pub supports_256_color: bool,
    pub supports_true_color: bool,
    pub supports_mouse: bool,
    pub min_width: u16,
    pub min_height: u16,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let term = std::env::var("TERM").unwrap_or_default();
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();

        Self {
            supports_unicode: Self::detect_unicode(),
            supports_256_color: term.contains("256color") || colorterm == "truecolor",
            supports_true_color: colorterm == "truecolor" || colorterm == "24bit",
            supports_mouse: true, // Assume true, fallback on error
            min_width: 60,
            min_height: 20,
        }
    }

    fn detect_unicode() -> bool {
        // Check LANG, LC_ALL, LC_CTYPE for UTF-8
        let lang = std::env::var("LANG").unwrap_or_default();
        let lc_all = std::env::var("LC_ALL").unwrap_or_default();

        lang.to_lowercase().contains("utf")
            || lc_all.to_lowercase().contains("utf")
            || std::env::var("TERM_PROGRAM").map(|p| p == "iTerm.app").unwrap_or(false)
    }
}

/// Global rendering context with fallbacks
pub struct RenderContext {
    pub capabilities: TerminalCapabilities,
    pub use_unicode: bool,
    pub use_color: bool,
    pub color_depth: ColorDepth,
}

pub enum ColorDepth {
    Monochrome,
    Basic16,
    Extended256,
    TrueColor,
}
```

### Rendering Error Recovery

```rust
// File: /Users/aaditshah/Documents/hive/src/render/error_recovery.rs (NEW)

use std::panic;

/// Wrap rendering in panic recovery
pub fn safe_render<F>(render_fn: F, buf: &mut Buffer, area: Rect)
where
    F: FnOnce(&mut Buffer, Rect) + panic::UnwindSafe
{
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        render_fn(buf, area);
    }));

    if result.is_err() {
        // Clear area and show error indicator
        render_error_fallback(buf, area, "Render error - press 'r' to retry");
    }
}

fn render_error_fallback(buf: &mut Buffer, area: Rect, message: &str) {
    // Clear area
    let style = Style::default().fg(Color::Red).bg(Color::Reset);
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_char(' ').set_style(style);
        }
    }

    // Center error message
    let msg_x = area.x + (area.width.saturating_sub(message.len() as u16)) / 2;
    let msg_y = area.y + area.height / 2;

    for (i, ch) in message.chars().enumerate() {
        if msg_x + i as u16 < area.x + area.width {
            buf[(msg_x + i as u16, msg_y)].set_char(ch).set_style(style);
        }
    }
}
```

### Terminal Resize Handling

```rust
// File: /Users/aaditshah/Documents/hive/src/app.rs (additions)

impl App {
    pub fn handle_resize(&mut self, new_width: u16, new_height: u16) {
        let caps = &self.render_context.capabilities;

        if new_width < caps.min_width || new_height < caps.min_height {
            self.show_size_warning = true;
            self.render_minimal_mode = true;
        } else {
            self.show_size_warning = false;
            self.render_minimal_mode = false;
        }

        // Recalculate layout
        self.layout = self.calculate_layout(new_width, new_height);

        // Reposition agents to fit new bounds
        for agent in self.field.agents.values_mut() {
            agent.position = agent.position.clamp();
            agent.target_position = agent.target_position.clamp();
        }
    }
}

/// Minimal mode for small terminals
fn render_minimal_mode(area: Rect, buf: &mut Buffer, agents: &[&Agent]) {
    // Show only: agent count, error count, status summary
    let summary = format!(
        "Agents: {} | Active: {} | Errors: {} | [Expand terminal for full view]",
        agents.len(),
        agents.iter().filter(|a| a.status == AgentStatus::Active).count(),
        agents.iter().filter(|a| a.status == AgentStatus::Error).count(),
    );

    let style = Style::default().fg(Color::Yellow);
    let x = area.x + 1;
    let y = area.y + area.height / 2;

    for (i, ch) in summary.chars().take(area.width as usize - 2).enumerate() {
        buf[(x + i as u16, y)].set_char(ch).set_style(style);
    }
}
```

### Mouse Fallback

```rust
// File: /Users/aaditshah/Documents/hive/src/input/mouse.rs (additions)

impl MouseHandler {
    pub fn new(capabilities: &TerminalCapabilities) -> Self {
        Self {
            enabled: capabilities.supports_mouse,
            last_position: None,
            hovered_agent: None,
            fallback_mode: !capabilities.supports_mouse,
        }
    }

    pub fn enable(&mut self) -> Result<(), std::io::Error> {
        if self.enabled {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::event::EnableMouseCapture
            ).map_err(|_| {
                self.enabled = false;
                self.fallback_mode = true;
                std::io::Error::new(std::io::ErrorKind::Other, "Mouse not supported")
            })
        } else {
            Ok(()) // Already in fallback mode
        }
    }
}

// Show keyboard navigation hint when mouse unavailable
fn render_no_mouse_hint(buf: &mut Buffer, area: Rect) {
    let hint = "Use Tab/Shift+Tab to navigate agents";
    let style = Style::default().fg(Color::Rgb(150, 150, 150));
    // Render in status bar area
}
```

### Filter State Persistence

```rust
// File: /Users/aaditshah/Documents/hive/src/state/filter.rs (additions)

/// How to handle trails/heatmap for filtered agents
#[derive(Clone, Copy, PartialEq)]
pub enum FilteredAgentDisplay {
    /// Hide completely (no trails, no heatmap contribution)
    Hidden,
    /// Show as ghosts (dimmed trails, reduced heatmap)
    Ghost,
    /// Keep trails/heatmap but hide agent symbol
    TrailsOnly,
}

pub struct AgentFilter {
    pub status_filter: Option<Vec<AgentStatus>>,
    pub name_filter: Option<String>,
    pub focus_filter: Option<Vec<String>>,
    pub intensity_threshold: Option<f32>,
    pub filtered_display: FilteredAgentDisplay,
}

impl AgentFilter {
    /// Get display opacity for filtered-out agents (0.0 = hidden, 0.3 = ghost)
    pub fn filtered_opacity(&self) -> f32 {
        match self.filtered_display {
            FilteredAgentDisplay::Hidden => 0.0,
            FilteredAgentDisplay::Ghost => 0.3,
            FilteredAgentDisplay::TrailsOnly => 0.0, // Agent hidden, trails visible
        }
    }

    /// Should this agent contribute to heatmap?
    pub fn contributes_to_heatmap(&self, agent: &Agent) -> bool {
        self.matches(agent) || self.filtered_display != FilteredAgentDisplay::Hidden
    }
}
```

---

## Accessibility Considerations

### Screen Reader Support

```rust
// File: /Users/aaditshah/Documents/hive/src/accessibility/screen_reader.rs (NEW)

/// Generate text description of current state for screen readers
pub struct ScreenReaderAnnouncer {
    last_announced: HashMap<String, String>,
    announcement_queue: VecDeque<String>,
}

impl ScreenReaderAnnouncer {
    /// Generate state summary (call periodically or on significant changes)
    pub fn generate_summary(&mut self, state: &AppState) -> Option<String> {
        let active_count = state.agents.values()
            .filter(|a| a.status == AgentStatus::Active).count();
        let error_count = state.agents.values()
            .filter(|a| a.status == AgentStatus::Error).count();

        let summary = format!(
            "{} agents total. {} active, {} errors. {}",
            state.agents.len(),
            active_count,
            error_count,
            if let Some(selected) = &state.selected_agent {
                format!("Selected: {}", selected)
            } else {
                "No selection.".to_string()
            }
        );

        // Only announce if changed
        if self.last_announced.get("summary") != Some(&summary) {
            self.last_announced.insert("summary".to_string(), summary.clone());
            Some(summary)
        } else {
            None
        }
    }

    /// Announce agent selection change
    pub fn announce_selection(&mut self, agent: &Agent) -> String {
        format!(
            "Selected {}. Status: {}. Intensity: {}%. Focus: {}",
            agent.id,
            agent.status.as_str(),
            (agent.intensity * 100.0) as u32,
            agent.focus.join(", ")
        )
    }
}
```

### Reduced Motion Mode

```rust
// File: /Users/aaditshah/Documents/hive/src/accessibility/motion.rs (NEW)

/// Respect user preference for reduced motion
pub struct MotionPreference {
    pub reduced_motion: bool,
}

impl MotionPreference {
    pub fn detect() -> Self {
        // Check environment variable (user can set HIVE_REDUCED_MOTION=1)
        let reduced = std::env::var("HIVE_REDUCED_MOTION")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Self { reduced_motion: reduced }
    }
}

impl Agent {
    pub fn get_animation_style(&self, motion_pref: &MotionPreference) -> AnimationStyle {
        if motion_pref.reduced_motion {
            // No animations - use static indicators only
            return AnimationStyle::Static;
        }

        // Normal animation logic
        match self.status {
            AgentStatus::Active if self.intensity > 0.6 => AnimationStyle::Pulse {
                frequency: 1.5 + self.intensity,
                amplitude: 0.15,
            },
            AgentStatus::Thinking => AnimationStyle::Breathe { frequency: 0.5 },
            AgentStatus::Waiting => AnimationStyle::Blink { interval: 2.0 },
            AgentStatus::Error => AnimationStyle::Flicker { intensity: 0.8 },
            _ => AnimationStyle::Static,
        }
    }
}
```

### High Contrast Mode

```rust
// File: /Users/aaditshah/Documents/hive/src/accessibility/contrast.rs (NEW)

/// High contrast color scheme for accessibility
pub const HIGH_CONTRAST_COLORS: AgentColorScheme = AgentColorScheme {
    agent_colors: [
        Color::White,           // Instead of Orange
        Color::Cyan,            // Instead of Sky Blue
        Color::Green,           // Instead of Bluish Green
        Color::Yellow,          // Instead of Yellow (unchanged)
        Color::Blue,            // Instead of Blue
        Color::Red,             // Instead of Vermillion
        Color::Magenta,         // Instead of Reddish Purple
        Color::White,           // Instead of Gray
    ],
    status_colors: StatusColorMap {
        active: Color::Green,
        thinking: Color::Cyan,
        waiting: Color::Yellow,
        idle: Color::White,
        error: Color::Red,
    },
    background: Color::Black,
    foreground: Color::White,
};

pub fn get_color_scheme(mode: ColorMode) -> &'static AgentColorScheme {
    match mode {
        ColorMode::Normal => &NORMAL_COLORS,
        ColorMode::HighContrast => &HIGH_CONTRAST_COLORS,
        ColorMode::Monochrome => &MONOCHROME_COLORS,
    }
}
```

### Monochrome Mode Specification

```rust
// File: /Users/aaditshah/Documents/hive/src/accessibility/monochrome.rs (NEW)

/// Monochrome color scheme - uses only grayscale
pub const MONOCHROME_COLORS: AgentColorScheme = AgentColorScheme {
    agent_colors: [
        Color::Rgb(255, 255, 255), // White
        Color::Rgb(220, 220, 220), // Light gray
        Color::Rgb(180, 180, 180), // Medium-light gray
        Color::Rgb(140, 140, 140), // Medium gray
        Color::Rgb(100, 100, 100), // Medium-dark gray
        Color::Rgb(200, 200, 200), // Off-white
        Color::Rgb(160, 160, 160), // Silver
        Color::Rgb(120, 120, 120), // Dark gray
    ],
    status_colors: StatusColorMap {
        active: Color::Rgb(255, 255, 255),   // Bright white
        thinking: Color::Rgb(200, 200, 200), // Light gray
        waiting: Color::Rgb(180, 180, 180),  // Medium gray (with blink)
        idle: Color::Rgb(100, 100, 100),     // Dark gray
        error: Color::Rgb(255, 255, 255),    // White (with rapid blink)
    },
    background: Color::Rgb(0, 0, 0),
    foreground: Color::Rgb(255, 255, 255),
};

/// In monochrome mode, status is conveyed through:
/// 1. Shape (unchanged from normal mode)
/// 2. Animation (more pronounced)
/// 3. Brightness level
/// 4. Text label suffix
impl Agent {
    pub fn monochrome_label_suffix(&self) -> &'static str {
        match self.status {
            AgentStatus::Active => "[*]",   // Asterisk = working
            AgentStatus::Thinking => "[.]", // Dot = processing
            AgentStatus::Waiting => "[~]",  // Tilde = waiting
            AgentStatus::Idle => "[-]",     // Dash = idle
            AgentStatus::Error => "[!]",    // Bang = error
        }
    }
}
```

### Accessibility Shortcuts

```rust
// Additional keyboard shortcuts for accessibility
KeyCode::F1 => Some(Action::ToggleHighContrast),
KeyCode::F2 => Some(Action::ToggleReducedMotion),
KeyCode::F3 => Some(Action::ToggleMonochrome),
KeyCode::F4 => Some(Action::AnnounceState), // Trigger screen reader summary
```

---

## Phase 1: Foundation (Layer-Based Rendering + Visual Clarity)

**Goal:** Establish the rendering foundation and make agent status instantly distinguishable.

### 1.1 Layer-Based Rendering (Moved from Phase 4)

Establish clear render order with z-index logic. This is foundational for all other phases.

```rust
// File: /Users/aaditshah/Documents/hive/src/render/layers.rs (NEW)

/// Render layers in strict order
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Background = 0,      // Grid, zone fills
    Heatmap = 1,         // Activity heatmap
    Trails = 2,          // Agent movement trails
    ZoneLabels = 3,      // Zone text labels
    Connections = 4,     // Lines between agents
    EventFlashes = 5,    // Temporary event indicators
    Agents = 6,          // Agent symbols
    AgentLabels = 7,     // Agent name labels
    StatusIndicators = 8, // Status symbols (above labels)
    Selection = 9,       // Selection highlight
    Overlay = 10,        // Panels, tooltips
    Chrome = 11,         // UI elements, status bar
}

pub fn render_field(ctx: &RenderContext, state: &AppState, area: Rect, buf: &mut Buffer) {
    // Layer 0: Background / Grid (optional)
    if state.display_mode.show_grid {
        render_grid(area, buf, ctx);
    }

    // Layer 1: Heatmap (if enabled)
    if state.display_mode.show_heatmap {
        render_heatmap(&state.heatmap, area, buf, ctx);
    }

    // Layer 2: Trails
    if state.display_mode.show_trails {
        render_trails(&state.agents, area, buf, ctx, &state.filter);
    }

    // Layer 3: Zone labels (subtle)
    render_zone_labels(&state.positioner, area, buf, ctx);

    // Layer 4: Connections (behind agents)
    render_connections(&state.connections, &state.agents, area, buf, ctx);

    // Layer 5: Event flashes
    render_flashes(&state.flashes, area, buf, ctx);

    // Layer 6: Agents (foreground)
    render_agents(&state.agents, area, buf, ctx, &state.selected_agent, &state.filter);

    // Layer 7: Agent labels (on top of agents)
    render_agent_labels(&state.agents, area, buf, ctx, &state.filter);

    // Layer 8: Status indicators (positioned to avoid labels)
    render_status_indicators(&state.agents, area, buf, ctx, &state.filter);

    // Layer 9: Selection highlight
    if let Some(selected) = &state.selected_agent {
        render_selection_highlight(selected, &state.agents, area, buf, ctx);
    }
}
```

### 1.2 Color Module Extraction

```rust
// File: /Users/aaditshah/Documents/hive/src/render/colors.rs (NEW)

/// Okabe-Ito colorblind-safe palette (high contrast version)
pub const AGENT_COLORS: [Color; 8] = [
    Color::Rgb(230, 159, 0),    // Orange (distinct from yellow)
    Color::Rgb(86, 180, 233),   // Sky Blue (distinct from purple)
    Color::Rgb(0, 158, 115),    // Bluish Green (distinct from blue)
    Color::Rgb(240, 228, 66),   // Yellow (bright, use sparingly)
    Color::Rgb(0, 114, 178),    // Blue (darker than sky blue)
    Color::Rgb(213, 94, 0),     // Vermillion (red-orange)
    Color::Rgb(204, 121, 167),  // Reddish Purple
    Color::Rgb(170, 170, 170),  // Gray (neutral fallback)
];

/// Status colors (used for indicators, not agent bodies)
pub const STATUS_COLORS: StatusColorMap = StatusColorMap {
    active: Color::Rgb(0, 200, 100),    // Green - working
    thinking: Color::Rgb(100, 150, 255), // Blue - processing
    waiting: Color::Rgb(255, 200, 80),   // Amber - blocked
    idle: Color::Rgb(100, 100, 100),     // Gray - inactive
    error: Color::Rgb(255, 80, 80),      // Red - problem
};

pub struct StatusColorMap {
    pub active: Color,
    pub thinking: Color,
    pub waiting: Color,
    pub idle: Color,
    pub error: Color,
}

impl StatusColorMap {
    pub fn get(&self, status: AgentStatus) -> Color {
        match status {
            AgentStatus::Active => self.active,
            AgentStatus::Thinking => self.thinking,
            AgentStatus::Waiting => self.waiting,
            AgentStatus::Idle => self.idle,
            AgentStatus::Error => self.error,
        }
    }
}

/// Dim a color by a factor (0.0 = black, 1.0 = unchanged)
pub fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        other => other,
    }
}

/// Get agent color with context-based adjustment
pub fn get_agent_color(
    agent: &Agent,
    scheme: &AgentColorScheme,
    is_selected: bool,
    is_filtered_out: bool,
) -> Color {
    let base = scheme.agent_colors[agent.color_index % scheme.agent_colors.len()];

    if is_filtered_out {
        dim_color(base, 0.3)
    } else if is_selected {
        base // Full brightness
    } else {
        dim_color(base, 0.6 + agent.intensity * 0.4)
    }
}
```

### 1.3 Symbol System with Unicode Codepoints and ASCII Fallbacks

```rust
// File: /Users/aaditshah/Documents/hive/src/render/symbols.rs (NEW)

/// Symbol with Unicode and ASCII fallback
pub struct Symbol {
    pub unicode: char,
    pub ascii: char,
    pub name: &'static str,
}

impl Symbol {
    pub fn render(&self, use_unicode: bool) -> char {
        if use_unicode { self.unicode } else { self.ascii }
    }
}

/// Agent type symbols (identity - based on color_index)
pub const AGENT_SHAPES: [Symbol; 8] = [
    Symbol { unicode: '\u{25C6}', ascii: '<', name: "diamond" },      // U+25C6 Black Diamond
    Symbol { unicode: '\u{25B2}', ascii: '^', name: "triangle_up" },  // U+25B2 Black Up Triangle
    Symbol { unicode: '\u{25A0}', ascii: '#', name: "square" },       // U+25A0 Black Square
    Symbol { unicode: '\u{25BC}', ascii: 'v', name: "triangle_down" },// U+25BC Black Down Triangle
    Symbol { unicode: '\u{2B1F}', ascii: '*', name: "pentagon" },     // U+2B1F Black Pentagon
    Symbol { unicode: '\u{2B22}', ascii: 'H', name: "hexagon" },      // U+2B22 Black Hexagon
    Symbol { unicode: '\u{2605}', ascii: '*', name: "star" },         // U+2605 Black Star
    Symbol { unicode: '\u{2663}', ascii: '&', name: "club" },         // U+2663 Black Club Suit
];

/// Status indicator symbols (single character for layout consistency)
pub const STATUS_INDICATORS: StatusSymbols = StatusSymbols {
    active: Symbol { unicode: '\u{2022}', ascii: '*', name: "active" },     // U+2022 Bullet
    thinking: Symbol { unicode: '\u{2026}', ascii: '.', name: "thinking" }, // U+2026 Ellipsis (SINGLE CHAR)
    waiting: Symbol { unicode: '\u{29D6}', ascii: '~', name: "waiting" },   // U+29D6 Hourglass
    idle: Symbol { unicode: '\u{2013}', ascii: '-', name: "idle" },         // U+2013 En Dash
    error: Symbol { unicode: '\u{2757}', ascii: '!', name: "error" },       // U+2757 Exclamation
};

pub struct StatusSymbols {
    pub active: Symbol,
    pub thinking: Symbol,
    pub waiting: Symbol,
    pub idle: Symbol,
    pub error: Symbol,
}

impl StatusSymbols {
    pub fn get(&self, status: AgentStatus) -> &Symbol {
        match status {
            AgentStatus::Active => &self.active,
            AgentStatus::Thinking => &self.thinking,
            AgentStatus::Waiting => &self.waiting,
            AgentStatus::Idle => &self.idle,
            AgentStatus::Error => &self.error,
        }
    }
}

/// Connection line characters
pub const LINE_CHARS: LineCharset = LineCharset {
    horizontal: Symbol { unicode: '\u{2500}', ascii: '-', name: "h_line" },   // U+2500 Box Drawing Light Horizontal
    vertical: Symbol { unicode: '\u{2502}', ascii: '|', name: "v_line" },     // U+2502 Box Drawing Light Vertical
    cross: Symbol { unicode: '\u{253C}', ascii: '+', name: "cross" },         // U+253C Box Drawing Light Cross
    dot: Symbol { unicode: '\u{00B7}', ascii: '.', name: "dot" },             // U+00B7 Middle Dot
};

/// Trail characters
pub const TRAIL_CHARS: TrailCharset = TrailCharset {
    recent: Symbol { unicode: '\u{2022}', ascii: 'o', name: "trail_recent" }, // U+2022 Bullet
    medium: Symbol { unicode: '\u{00B7}', ascii: '.', name: "trail_medium" }, // U+00B7 Middle Dot
    faded: Symbol { unicode: '\u{2219}', ascii: '.', name: "trail_faded" },   // U+2219 Bullet Operator
};

/// UI chrome characters
pub const CHROME_CHARS: ChromeCharset = ChromeCharset {
    border_h: Symbol { unicode: '\u{2500}', ascii: '-', name: "border_h" },
    border_v: Symbol { unicode: '\u{2502}', ascii: '|', name: "border_v" },
    corner_tl: Symbol { unicode: '\u{250C}', ascii: '+', name: "corner_tl" },
    corner_tr: Symbol { unicode: '\u{2510}', ascii: '+', name: "corner_tr" },
    corner_bl: Symbol { unicode: '\u{2514}', ascii: '+', name: "corner_bl" },
    corner_br: Symbol { unicode: '\u{2518}', ascii: '+', name: "corner_br" },
    scrollbar_track: Symbol { unicode: '\u{2591}', ascii: ':', name: "scroll_track" },
    scrollbar_thumb: Symbol { unicode: '\u{2588}', ascii: '#', name: "scroll_thumb" },
    checkbox_on: Symbol { unicode: '\u{2611}', ascii: 'X', name: "check_on" },
    checkbox_off: Symbol { unicode: '\u{2610}', ascii: 'O', name: "check_off" },
};

pub struct LineCharset {
    pub horizontal: Symbol,
    pub vertical: Symbol,
    pub cross: Symbol,
    pub dot: Symbol,
}

pub struct TrailCharset {
    pub recent: Symbol,
    pub medium: Symbol,
    pub faded: Symbol,
}

pub struct ChromeCharset {
    pub border_h: Symbol,
    pub border_v: Symbol,
    pub corner_tl: Symbol,
    pub corner_tr: Symbol,
    pub corner_bl: Symbol,
    pub corner_br: Symbol,
    pub scrollbar_track: Symbol,
    pub scrollbar_thumb: Symbol,
    pub checkbox_on: Symbol,
    pub checkbox_off: Symbol,
}
```

### 1.4 Easing Functions (Moved from Phase 3)

```rust
// File: /Users/aaditshah/Documents/hive/src/animation/easing.rs (NEW)

/// Easing functions for smooth animations
pub mod easing {
    /// Ease out cubic - starts fast, decelerates
    pub fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    /// Ease in out cubic - smooth start and end
    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// Ease out elastic - slight overshoot
    pub fn ease_out_elastic(t: f32) -> f32 {
        if t == 0.0 || t == 1.0 {
            return t;
        }
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        (2.0_f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
    }

    /// Ease out back - slight overshoot then settle
    pub fn ease_out_back(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
    }

    /// Ease in cubic - starts slow, accelerates
    pub fn ease_in_cubic(t: f32) -> f32 {
        t * t * t
    }
}
```

### 1.5 Label and Status Indicator Positioning (Conflict Resolved)

```rust
// File: /Users/aaditshah/Documents/hive/src/render/agent.rs

/// Label rendering with collision avoidance
/// Status indicator is rendered ABOVE the agent, label to the side
struct LabelRenderer {
    rendered_labels: Vec<Rect>,
    rendered_indicators: Vec<Rect>,
}

impl LabelRenderer {
    /// Render agent with proper layering:
    /// 1. Agent symbol at (x, y)
    /// 2. Status indicator at (x, y-1) - ABOVE agent
    /// 3. Label to the right, left, or below - avoiding indicator
    fn render_agent_complete(
        &mut self,
        buf: &mut Buffer,
        agent: &Agent,
        agent_pos: (u16, u16),
        area: Rect,
        ctx: &RenderContext,
    ) {
        let (x, y) = agent_pos;

        // 1. Render agent symbol
        let shape = &AGENT_SHAPES[agent.color_index % AGENT_SHAPES.len()];
        let agent_color = get_agent_color(agent, ctx.scheme, false, false);
        buf[(x, y)]
            .set_char(shape.render(ctx.use_unicode))
            .set_style(Style::default().fg(agent_color));

        // 2. Render status indicator ABOVE agent (y-1)
        if agent.status != AgentStatus::Active {
            let indicator = STATUS_INDICATORS.get(agent.status);
            let indicator_y = y.saturating_sub(1);

            if indicator_y > area.y {
                let status_color = ctx.scheme.status_colors.get(agent.status);
                buf[(x, indicator_y)]
                    .set_char(indicator.render(ctx.use_unicode))
                    .set_style(Style::default().fg(status_color));

                self.rendered_indicators.push(Rect::new(x, indicator_y, 1, 1));
            }
        }

        // 3. Render label (avoiding indicator position)
        self.render_label_avoiding_indicator(buf, agent, agent_pos, area, ctx);
    }

    fn render_label_avoiding_indicator(
        &mut self,
        buf: &mut Buffer,
        agent: &Agent,
        agent_pos: (u16, u16),
        area: Rect,
        ctx: &RenderContext,
    ) {
        let (x, y) = agent_pos;
        let label = &agent.id;
        let label_len = label.len().min(12) as u16;

        // Try positions: right, left, below (NOT above - indicator is there)
        let positions = [
            (x + 2, y, "right"),
            (x.saturating_sub(label_len + 1), y, "left"),
            (x.saturating_sub(label_len / 2), y + 1, "below"),
        ];

        for (lx, ly, _pos_name) in positions {
            let label_rect = Rect::new(lx, ly, label_len, 1);
            if self.can_place_label(&label_rect, area) {
                self.render_label_at(buf, label, label_rect, agent.color_index, ctx);
                self.rendered_labels.push(label_rect);
                return;
            }
        }

        // Fallback: truncate and render to the right anyway
        let truncated = if label.len() > 6 {
            format!("{}...", &label[..3])
        } else {
            label.clone()
        };
        let lx = x + 2;
        if lx + truncated.len() as u16 <= area.x + area.width - 1 {
            self.render_label_at(buf, &truncated, Rect::new(lx, y, truncated.len() as u16, 1), agent.color_index, ctx);
        }
    }

    fn can_place_label(&self, rect: &Rect, area: Rect) -> bool {
        // Check bounds
        if rect.x < area.x + 1 || rect.x + rect.width > area.x + area.width - 1 {
            return false;
        }
        if rect.y < area.y + 1 || rect.y + rect.height > area.y + area.height - 1 {
            return false;
        }
        // Check collision with existing labels
        if self.rendered_labels.iter().any(|r| rects_intersect(r, rect)) {
            return false;
        }
        // Check collision with indicators
        if self.rendered_indicators.iter().any(|r| rects_intersect(r, rect)) {
            return false;
        }
        true
    }

    fn render_label_at(
        &self,
        buf: &mut Buffer,
        label: &str,
        rect: Rect,
        color_index: usize,
        ctx: &RenderContext,
    ) {
        let color = dim_color(
            ctx.scheme.agent_colors[color_index % ctx.scheme.agent_colors.len()],
            0.8, // 80% brightness (up from 60%)
        );
        let style = Style::default().fg(color);

        for (i, ch) in label.chars().take(rect.width as usize).enumerate() {
            buf[(rect.x + i as u16, rect.y)].set_char(ch).set_style(style);
        }
    }
}

fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.width <= b.x
      || b.x + b.width <= a.x
      || a.y + a.height <= b.y
      || b.y + b.height <= a.y)
}
```

### 1.6 Remove/Reduce Visual Noise

**1.6.1 Selective Pulsing:**
```rust
// File: /Users/aaditshah/Documents/hive/src/state/agent.rs

impl Agent {
    pub fn should_pulse(&self, motion_pref: &MotionPreference) -> bool {
        if motion_pref.reduced_motion {
            return false;
        }
        // Only pulse when actively working at high intensity
        self.status == AgentStatus::Active && self.intensity > 0.6
    }

    pub fn pulse_brightness(&self, motion_pref: &MotionPreference) -> f32 {
        if self.should_pulse(motion_pref) {
            let base = 0.85;
            let variation = 0.15;
            base + variation * (self.pulse_phase.sin() * 0.5 + 0.5)
        } else {
            // Static brightness based on intensity
            0.6 + self.intensity * 0.4
        }
    }
}
```

**1.6.2 Distinct Glow:**
```rust
// File: /Users/aaditshah/Documents/hive/src/render/agent.rs

// Replace dot glow with subtle line brackets
fn render_agent_glow(agent: &Agent, x: u16, y: u16, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
    if agent.intensity > 0.6 && !ctx.is_selected(&agent.id) {
        let base_color = ctx.scheme.agent_colors[agent.color_index % ctx.scheme.agent_colors.len()];
        let glow_color = dim_color(base_color, 0.4);
        let glow_style = Style::default().fg(glow_color);

        // Left bracket
        if x > area.x + 1 {
            buf[(x - 1, y)].set_char('[').set_style(glow_style);
        }
        // Right bracket
        if x < area.x + area.width - 2 {
            buf[(x + 1, y)].set_char(']').set_style(glow_style);
        }
    }
}
```

**1.6.3 Faster Heatmap Decay:**
```rust
// File: /Users/aaditshah/Documents/hive/src/render/heatmap.rs

/// Configurable heatmap constants
pub struct HeatmapConfig {
    /// Heat decay rate per frame (default: 0.98, was 0.995)
    pub decay_rate: f32,
    /// Minimum heat threshold before clearing (default: 0.02, was 0.01)
    pub heat_threshold: f32,
    /// Maximum heat value
    pub max_heat: f32,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.98,
            heat_threshold: 0.02,
            max_heat: 1.0,
        }
    }
}
```

### 1.7 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/render/layers.rs` | NEW - Layer definitions and render order |
| `src/render/colors.rs` | NEW - Extracted color module |
| `src/render/symbols.rs` | NEW - Unicode/ASCII symbol definitions |
| `src/render/mod.rs` | Add new modules, new color palette |
| `src/animation/easing.rs` | NEW - Easing functions |
| `src/state/agent.rs` | Selective pulsing |
| `src/render/agent.rs` | Label collision, glow effect, status positioning |
| `src/render/heatmap.rs` | Configurable decay rate |
| `src/terminal/capabilities.rs` | NEW - Terminal detection |
| `src/render/error_recovery.rs` | NEW - Safe rendering |

**Estimated Complexity:** 5-7 days

**Dependencies:** None (foundation phase)

---

## Phase 2: Spatial Organization

**Goal:** Reduce congestion and make semantic zones visible.

### 2.1 Zone Visualization (Derived from SemanticPositioner)

```rust
// File: /Users/aaditshah/Documents/hive/src/render/zones.rs (NEW)

pub struct ZoneRenderer;

impl ZoneRenderer {
    /// Render zones derived from SemanticPositioner clusters
    pub fn render_zones(
        positioner: &SemanticPositioner,
        area: Rect,
        buf: &mut Buffer,
        ctx: &RenderContext,
    ) {
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);

        // Derive zones from actual concept clusters in positioner
        for cluster in &positioner.concept_clusters {
            let (x, y) = cluster.center.to_terminal(inner_width, inner_height);
            let draw_x = area.x + 1 + x;
            let draw_y = area.y + 1 + y;

            // Draw subtle zone label
            let label_style = Style::default()
                .fg(Color::Rgb(70, 70, 90))
                .add_modifier(Modifier::DIM);

            // Derive short label from cluster keywords
            let label = Self::derive_label(&cluster.keywords);

            // Position label above cluster center
            let label_y = draw_y.saturating_sub(3);
            let label_x = draw_x.saturating_sub(label.len() as u16 / 2);

            for (i, ch) in label.chars().enumerate() {
                let lx = label_x + i as u16;
                if lx > area.x && lx < area.x + area.width - 1
                   && label_y > area.y && label_y < area.y + area.height - 1 {
                    let cell = &mut buf[(lx, label_y)];
                    if cell.symbol() == " " {
                        cell.set_char(ch).set_style(label_style);
                    }
                }
            }
        }
    }

    /// Derive a short label from cluster keywords
    fn derive_label(keywords: &[String]) -> String {
        // Use first keyword, uppercase, max 6 chars
        keywords.first()
            .map(|k| k.to_uppercase().chars().take(6).collect())
            .unwrap_or_else(|| "ZONE".to_string())
    }
}

/// Optional grid mode (toggled with 'g' key)
pub fn render_grid(area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
    let grid_style = Style::default().fg(Color::Rgb(35, 35, 45));
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);

    // Vertical lines at 25%, 50%, 75%
    for pct in [0.25, 0.5, 0.75] {
        let x = area.x + 1 + (inner_width as f32 * pct) as u16;
        for y in (area.y + 1)..(area.y + area.height - 1) {
            let cell = &mut buf[(x, y)];
            if cell.symbol() == " " {
                let ch = CHROME_CHARS.border_v.render(ctx.use_unicode);
                cell.set_char(ch).set_style(grid_style);
            }
        }
    }

    // Horizontal lines
    for pct in [0.25, 0.5, 0.75] {
        let y = area.y + 1 + (inner_height as f32 * pct) as u16;
        for x in (area.x + 1)..(area.x + area.width - 1) {
            let cell = &mut buf[(x, y)];
            if cell.symbol() == " " {
                let ch = CHROME_CHARS.border_h.render(ctx.use_unicode);
                cell.set_char(ch).set_style(grid_style);
            }
        }
    }
}
```

### 2.2 Spatial Hash for Collision Detection

```rust
// File: /Users/aaditshah/Documents/hive/src/positioning/spatial_hash.rs (NEW)

use std::collections::HashMap;

/// Spatial hash grid for O(1) average collision detection
/// Instead of O(n^2) checking all pairs
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
    grid_width: i32,
    grid_height: i32,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            grid_width: (1.0 / cell_size).ceil() as i32,
            grid_height: (1.0 / cell_size).ceil() as i32,
        }
    }

    /// Clear and rebuild with current positions
    pub fn rebuild(&mut self, positions: &[Position]) {
        self.cells.clear();

        for (i, pos) in positions.iter().enumerate() {
            let cell = self.position_to_cell(pos);
            self.cells.entry(cell).or_insert_with(Vec::new).push(i);
        }
    }

    fn position_to_cell(&self, pos: &Position) -> (i32, i32) {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cy = (pos.y / self.cell_size).floor() as i32;
        (cx.clamp(0, self.grid_width - 1), cy.clamp(0, self.grid_height - 1))
    }

    /// Get indices of agents that might collide with agent at given position
    /// Only checks current cell and 8 neighbors
    pub fn get_nearby(&self, pos: &Position) -> Vec<usize> {
        let (cx, cy) = self.position_to_cell(pos);
        let mut nearby = Vec::new();

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
}

/// Collision avoidance using spatial hash
pub struct CollisionAvoidance {
    spatial_hash: SpatialHash,
    min_distance: f32,
    separation_force: f32,
}

impl CollisionAvoidance {
    pub fn new() -> Self {
        Self {
            // Cell size should be >= 2x min_distance for correct neighbor detection
            spatial_hash: SpatialHash::new(0.16),
            min_distance: 0.08,       // Configurable: MIN_AGENT_DISTANCE
            separation_force: 0.5,    // Configurable: SEPARATION_FORCE
        }
    }

    /// Apply separation to all agents in O(n) average time
    pub fn apply_separation(&mut self, positions: &mut [Position]) {
        // Rebuild spatial hash
        self.spatial_hash.rebuild(positions);

        // Calculate forces for each agent
        let forces: Vec<(f32, f32)> = positions.iter().enumerate().map(|(i, pos)| {
            let mut force_x = 0.0;
            let mut force_y = 0.0;

            // Only check nearby agents
            for j in self.spatial_hash.get_nearby(pos) {
                if j == i {
                    continue;
                }

                let other = &positions[j];
                let dx = pos.x - other.x;
                let dy = pos.y - other.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < self.min_distance && dist > 0.001 {
                    let strength = (self.min_distance - dist) / self.min_distance;
                    force_x += (dx / dist) * strength * self.separation_force;
                    force_y += (dy / dist) * strength * self.separation_force;
                }
            }

            (force_x, force_y)
        }).collect();

        // Apply forces
        for (i, (fx, fy)) in forces.into_iter().enumerate() {
            positions[i].x = (positions[i].x + fx).clamp(0.05, 0.95);
            positions[i].y = (positions[i].y + fy).clamp(0.05, 0.95);
        }
    }
}
```

### 2.3 Improved Semantic Positioning

```rust
// File: /Users/aaditshah/Documents/hive/src/positioning/semantic.rs

// Increase cluster radii
self.concept_clusters.push(ConceptCluster {
    center: Position::new(0.2, 0.2),
    keywords: vec!["frontend", "ui", "component", "style"],
    radius: 0.20,  // Was 0.15
});

// Use softer clamping
impl Position {
    pub fn clamp(&self) -> Self {
        Self {
            x: self.x.clamp(0.08, 0.92),  // Was 0.05/0.95
            y: self.y.clamp(0.08, 0.92),
        }
    }
}
```

### 2.4 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/render/zones.rs` | NEW - Zone rendering derived from positioner |
| `src/positioning/spatial_hash.rs` | NEW - Spatial hash for O(n) collision |
| `src/render/mod.rs` | Add zones module |
| `src/positioning/mod.rs` | Add spatial hash, use for collision |
| `src/positioning/semantic.rs` | Larger radii, softer clamping |
| `src/state/field.rs` | Use spatial hash in update loop |

**Estimated Complexity:** 3-4 days

**Dependencies:** Phase 1 (layers, labels need collision too)

---

## Phase 3: Display Modes (Consolidating Toggles)

**Goal:** Replace toggle explosion with 3 clear display modes.

### 3.1 Display Mode System

```rust
// File: /Users/aaditshah/Documents/hive/src/state/display_mode.rs (NEW)

/// Three consolidated display modes instead of 6+ individual toggles
#[derive(Clone, Copy, PartialEq, Default)]
pub enum DisplayMode {
    /// Clean view: agents + labels only
    Minimal,
    /// Default view: agents + labels + connections + activity log
    #[default]
    Standard,
    /// Full debug view: everything including grid, heatmap, trails
    Debug,
}

impl DisplayMode {
    pub fn show_grid(&self) -> bool {
        matches!(self, DisplayMode::Debug)
    }

    pub fn show_heatmap(&self) -> bool {
        matches!(self, DisplayMode::Debug)
    }

    pub fn show_trails(&self) -> bool {
        matches!(self, DisplayMode::Standard | DisplayMode::Debug)
    }

    pub fn show_landmarks(&self) -> bool {
        matches!(self, DisplayMode::Debug)
    }

    pub fn show_connections(&self) -> bool {
        matches!(self, DisplayMode::Standard | DisplayMode::Debug)
    }

    pub fn show_activity_log(&self) -> bool {
        matches!(self, DisplayMode::Standard | DisplayMode::Debug)
    }

    pub fn show_zone_labels(&self) -> bool {
        matches!(self, DisplayMode::Standard | DisplayMode::Debug)
    }

    pub fn show_event_flashes(&self) -> bool {
        matches!(self, DisplayMode::Standard | DisplayMode::Debug)
    }

    /// Cycle to next mode
    pub fn cycle(&self) -> Self {
        match self {
            DisplayMode::Minimal => DisplayMode::Standard,
            DisplayMode::Standard => DisplayMode::Debug,
            DisplayMode::Debug => DisplayMode::Minimal,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DisplayMode::Minimal => "Minimal",
            DisplayMode::Standard => "Standard",
            DisplayMode::Debug => "Debug",
        }
    }
}

/// Extended state for additional controls (not part of mode)
pub struct DisplayOptions {
    pub mode: DisplayMode,
    pub show_sidebar: bool,      // Toggle independently
    pub show_help: bool,         // Toggle independently
    pub color_mode: ColorMode,   // Accessibility
    pub reduced_motion: bool,    // Accessibility
}
```

### 3.2 Updated Input Handler

```rust
// File: /Users/aaditshah/Documents/hive/src/input/handler.rs

pub fn handle_key_event(key: KeyEvent, state: &mut AppState) -> Option<Action> {
    match key.code {
        // Mode cycling (single key instead of many)
        KeyCode::Char('m') => Some(Action::CycleDisplayMode),

        // Independent toggles (reduced from 6 to 2)
        KeyCode::Char('s') => Some(Action::ToggleSidebar),
        KeyCode::Char('?') => Some(Action::ToggleHelp),

        // Direct mode selection (optional shortcuts)
        KeyCode::Char('1') => Some(Action::SetDisplayMode(DisplayMode::Minimal)),
        KeyCode::Char('2') => Some(Action::SetDisplayMode(DisplayMode::Standard)),
        KeyCode::Char('3') => Some(Action::SetDisplayMode(DisplayMode::Debug)),

        // Navigation
        KeyCode::Tab => Some(Action::SelectNextAgent),
        KeyCode::BackTab => Some(Action::SelectPrevAgent),
        KeyCode::Enter => Some(Action::FocusSelectedAgent),
        KeyCode::Esc => {
            if state.selected_agent.is_some() {
                Some(Action::ClearSelection)
            } else if state.filter_mode {
                Some(Action::ExitFilterMode)
            } else {
                Some(Action::Quit)
            }
        }

        // Filtering
        KeyCode::Char('/') => Some(Action::EnterFilterMode),
        KeyCode::Char('0') => Some(Action::ClearFilter),

        // Playback
        KeyCode::Char(' ') => Some(Action::TogglePause),
        KeyCode::Char('+') | KeyCode::Char('=') => Some(Action::SpeedUp),
        KeyCode::Char('-') => Some(Action::SlowDown),

        // Accessibility
        KeyCode::F1 => Some(Action::ToggleHighContrast),
        KeyCode::F2 => Some(Action::ToggleReducedMotion),
        KeyCode::F3 => Some(Action::ToggleMonochrome),

        _ => None,
    }
}
```

### 3.3 Status Bar with Mode and Filter Indicator

```rust
// File: /Users/aaditshah/Documents/hive/src/render/ui.rs

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg_style = Style::default().bg(Color::Rgb(25, 28, 35));

        // Fill background
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_char(' ').set_style(bg_style);
        }

        let mut x = area.x + 1;

        // Agent count
        let count_text = format!("{} agents", self.agent_count);
        let count_style = bg_style.fg(Color::Rgb(150, 150, 160));
        for ch in count_text.chars() {
            buf[(x, area.y)].set_char(ch).set_style(count_style);
            x += 1;
        }
        x += 2;

        // FILTER INDICATOR (new)
        if self.filter_active {
            let filter_style = bg_style
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD);
            let filter_text = format!("[FILTER: {}]", self.filter_summary);
            for ch in filter_text.chars() {
                buf[(x, area.y)].set_char(ch).set_style(filter_style);
                x += 1;
            }
            x += 2;
        }

        // Display mode indicator (center)
        let mode_x = area.x + area.width / 2 - 5;
        let mode_style = bg_style
            .fg(Color::Rgb(100, 200, 150))
            .add_modifier(Modifier::BOLD);
        let mode_text = format!("[{}]", self.display_mode.name());
        for (i, ch) in mode_text.chars().enumerate() {
            buf[(mode_x + i as u16, area.y)].set_char(ch).set_style(mode_style);
        }

        // Sidebar indicator
        let sidebar_x = area.x + area.width - 20;
        let sidebar_style = if self.show_sidebar {
            bg_style.fg(Color::Rgb(100, 200, 150))
        } else {
            bg_style.fg(Color::Rgb(60, 60, 70))
        };
        buf[(sidebar_x, area.y)].set_char('S').set_style(sidebar_style);

        // FPS indicator (dimmed, right side)
        let fps_x = area.x + area.width - 10;
        let fps_text = format!("{}fps", self.fps);
        let fps_style = bg_style.fg(Color::Rgb(50, 50, 60));
        for (i, ch) in fps_text.chars().enumerate() {
            buf[(fps_x + i as u16, area.y)].set_char(ch).set_style(fps_style);
        }

        // Help hint (far right)
        let help_text = "?=help";
        let help_x = area.x + area.width - help_text.len() as u16 - 1;
        let help_style = bg_style.fg(Color::Rgb(80, 80, 90));
        for (i, ch) in help_text.chars().enumerate() {
            buf[(help_x + i as u16, area.y)].set_char(ch).set_style(help_style);
        }
    }
}
```

### 3.4 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/state/display_mode.rs` | NEW - Display mode system |
| `src/input/handler.rs` | Updated key handling |
| `src/render/ui.rs` | Mode indicator, filter indicator |
| `src/app.rs` | DisplayOptions state |

**Estimated Complexity:** 2-3 days

**Dependencies:** Phase 1 (layers)

---

## Phase 4: Unified Animation System

**Goal:** Consistent animation handling with reduced visual competition.

### 4.1 Unified Animation Trait

```rust
// File: /Users/aaditshah/Documents/hive/src/animation/trait.rs (NEW)

use std::time::Instant;

/// Unified animation trait for all animated elements
pub trait Animated {
    /// Update animation state based on elapsed time
    fn tick(&mut self, delta: f32);

    /// Get current visual intensity (0.0 - 1.0)
    fn intensity(&self) -> f32;

    /// Get current opacity (0.0 - 1.0)
    fn opacity(&self) -> f32;

    /// Is animation complete?
    fn is_complete(&self) -> bool;

    /// Reset animation to initial state
    fn reset(&mut self);
}

/// Animation state machine
#[derive(Clone)]
pub struct AnimationState {
    pub style: AnimationStyle,
    pub phase: f32,
    pub started_at: Instant,
    pub duration: Option<f32>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AnimationStyle {
    /// No animation
    Static,
    /// Smooth sine wave pulse
    Pulse { frequency: f32, amplitude: f32 },
    /// Gentle breathing effect
    Breathe { frequency: f32 },
    /// On/off blinking
    Blink { interval: f32 },
    /// Rapid brightness variation
    Flicker { intensity: f32 },
    /// Fade in then out
    Flash { duration: f32 },
    /// Fade out only
    FadeOut { duration: f32 },
}

impl AnimationState {
    pub fn new(style: AnimationStyle) -> Self {
        Self {
            style,
            phase: 0.0,
            started_at: Instant::now(),
            duration: None,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        match self.style {
            AnimationStyle::Static => {}
            AnimationStyle::Pulse { frequency, .. } => {
                self.phase += delta * frequency * std::f32::consts::TAU;
                self.phase %= std::f32::consts::TAU;
            }
            AnimationStyle::Breathe { frequency } => {
                self.phase += delta * frequency * std::f32::consts::TAU;
                self.phase %= std::f32::consts::TAU;
            }
            AnimationStyle::Blink { interval } => {
                self.phase += delta;
                self.phase %= interval * 2.0;
            }
            AnimationStyle::Flicker { .. } => {
                self.phase = rand::random::<f32>();
            }
            AnimationStyle::Flash { .. } | AnimationStyle::FadeOut { .. } => {
                // Time-based, no phase update needed
            }
        }
    }

    pub fn brightness(&self) -> f32 {
        match self.style {
            AnimationStyle::Static => 1.0,
            AnimationStyle::Pulse { amplitude, .. } => {
                1.0 - amplitude + amplitude * (self.phase.sin() * 0.5 + 0.5)
            }
            AnimationStyle::Breathe { .. } => {
                0.7 + 0.3 * (self.phase.sin() * 0.5 + 0.5)
            }
            AnimationStyle::Blink { interval } => {
                if self.phase < interval { 1.0 } else { 0.3 }
            }
            AnimationStyle::Flicker { intensity } => {
                0.5 + intensity * 0.5 * self.phase
            }
            AnimationStyle::Flash { duration } => {
                let t = self.started_at.elapsed().as_secs_f32() / duration;
                if t < 0.3 {
                    easing::ease_out_cubic(t / 0.3)
                } else if t < 0.7 {
                    1.0
                } else {
                    easing::ease_in_cubic((1.0 - t) / 0.3)
                }
            }
            AnimationStyle::FadeOut { duration } => {
                let t = self.started_at.elapsed().as_secs_f32() / duration;
                (1.0 - t).max(0.0)
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        match self.style {
            AnimationStyle::Flash { duration } | AnimationStyle::FadeOut { duration } => {
                self.started_at.elapsed().as_secs_f32() >= duration
            }
            _ => false,
        }
    }
}
```

### 4.2 Animation Priority System

```rust
// File: /Users/aaditshah/Documents/hive/src/animation/priority.rs (NEW)

/// Prevent competing animations by prioritizing
pub struct AnimationPriority {
    /// Maximum simultaneous event flashes
    pub max_flashes: usize,
    /// Maximum pulsing agents
    pub max_pulsing: usize,
    /// Activity log suppresses event flashes for same agent
    pub log_suppresses_flash: bool,
}

impl Default for AnimationPriority {
    fn default() -> Self {
        Self {
            max_flashes: 3,
            max_pulsing: 5,
            log_suppresses_flash: true,
        }
    }
}

impl AnimationPriority {
    /// Filter agents to show only highest priority animations
    pub fn filter_pulsing_agents<'a>(&self, agents: &'a [&Agent]) -> Vec<&'a Agent> {
        let mut pulsing: Vec<_> = agents.iter()
            .filter(|a| a.should_pulse(&MotionPreference::default()))
            .copied()
            .collect();

        // Sort by intensity (highest first)
        pulsing.sort_by(|a, b| b.intensity.partial_cmp(&a.intensity).unwrap());

        // Keep only top N
        pulsing.truncate(self.max_pulsing);
        pulsing
    }

    /// Filter flashes to avoid visual overload
    pub fn filter_flashes(&self, flashes: &mut Vec<EventFlash>, log: &ActivityLog) {
        // Remove flashes for agents already in recent log
        if self.log_suppresses_flash {
            flashes.retain(|f| {
                !log.has_recent_entry(&f.agent_id, std::time::Duration::from_secs(2))
            });
        }

        // Keep only most recent N flashes
        flashes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        flashes.truncate(self.max_flashes);
    }
}
```

### 4.3 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/animation/trait.rs` | NEW - Unified animation trait |
| `src/animation/priority.rs` | NEW - Animation priority |
| `src/animation/mod.rs` | Add new modules |
| `src/render/agent.rs` | Use animation trait |
| `src/render/flash.rs` | Use animation trait |
| `src/render/connections.rs` | Use animation trait |

**Estimated Complexity:** 3-4 days

**Dependencies:** Phase 1 (easing functions)

---

## Phase 5: Information Architecture & Interactivity

**Goal:** Present information in clear layers with on-demand detail.

### 5.1 Activity Log Strip

```rust
// File: /Users/aaditshah/Documents/hive/src/render/activity_log.rs (NEW)

const MAX_LOG_ENTRIES: usize = 100;
const VISIBLE_ENTRIES: usize = 3;

pub struct ActivityLog {
    entries: VecDeque<LogEntry>,
}

pub struct LogEntry {
    pub timestamp: Instant,
    pub agent_id: String,
    pub message: String,
    pub entry_type: LogEntryType,
}

pub enum LogEntryType {
    StatusChange,
    Connection,
    FocusChange,
    Error,
}

impl ActivityLog {
    pub fn add(&mut self, entry: LogEntry) {
        self.entries.push_back(entry);
        while self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.pop_front();
        }
    }

    pub fn has_recent_entry(&self, agent_id: &str, within: std::time::Duration) -> bool {
        self.entries.iter().rev()
            .take(10)
            .any(|e| e.agent_id == agent_id && e.timestamp.elapsed() < within)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        // Render in a strip above the status bar
        let log_height = VISIBLE_ENTRIES as u16;
        let log_area = Rect::new(
            area.x,
            area.y + area.height - 1 - log_height,
            area.width,
            log_height,
        );

        // Semi-transparent background
        let bg_style = Style::default().bg(Color::Rgb(20, 22, 28));
        for y in log_area.y..log_area.y + log_area.height {
            for x in log_area.x..log_area.x + log_area.width {
                buf[(x, y)].set_style(bg_style);
            }
        }

        // Render recent entries
        let recent: Vec<_> = self.entries.iter().rev().take(VISIBLE_ENTRIES).collect();
        for (i, entry) in recent.iter().rev().enumerate() {
            let y = log_area.y + i as u16;
            self.render_entry(entry, log_area.x + 1, y, log_area.width - 2, buf, ctx);
        }
    }

    fn render_entry(&self, entry: &LogEntry, x: u16, y: u16, width: u16, buf: &mut Buffer, ctx: &RenderContext) {
        let age = entry.timestamp.elapsed().as_secs();
        let opacity = if age < 5 { 1.0 } else { 0.7 };

        let type_color = match entry.entry_type {
            LogEntryType::StatusChange => Color::Rgb(100, 200, 150),
            LogEntryType::Connection => Color::Rgb(100, 150, 255),
            LogEntryType::FocusChange => Color::Rgb(200, 200, 100),
            LogEntryType::Error => Color::Rgb(255, 100, 100),
        };

        // Format: "[agent] message"
        let mut cx = x;

        // Agent name
        let agent_style = Style::default().fg(dim_color(type_color, opacity));
        let agent_text = format!("[{}] ", &entry.agent_id[..entry.agent_id.len().min(10)]);
        for ch in agent_text.chars() {
            if cx >= x + width { break; }
            buf[(cx, y)].set_char(ch).set_style(agent_style);
            cx += 1;
        }

        // Message
        let msg_style = Style::default().fg(dim_color(Color::Rgb(180, 180, 190), opacity));
        for ch in entry.message.chars() {
            if cx >= x + width { break; }
            buf[(cx, y)].set_char(ch).set_style(msg_style);
            cx += 1;
        }
    }
}
```

### 5.2 Mouse Hover/Click with Larger Targets

```rust
// File: /Users/aaditshah/Documents/hive/src/input/mouse.rs

impl MouseHandler {
    fn find_agent_at(
        &self,
        x: u16,
        y: u16,
        field_area: Rect,
        agents: &HashMap<String, Agent>,
    ) -> Option<String> {
        let inner_width = field_area.width.saturating_sub(2);
        let inner_height = field_area.height.saturating_sub(2);

        for (id, agent) in agents {
            let (ax, ay) = agent.position.to_terminal(inner_width, inner_height);
            let draw_x = field_area.x + 1 + ax;
            let draw_y = field_area.y + 1 + ay;

            // LARGER click target: 3x2 instead of 2x1
            let dx = (x as i32 - draw_x as i32).abs();
            let dy = (y as i32 - draw_y as i32).abs();

            if dx <= 3 && dy <= 2 {
                return Some(id.clone());
            }
        }
        None
    }
}
```

### 5.3 Empty State Designs

```rust
// File: /Users/aaditshah/Documents/hive/src/render/empty_state.rs (NEW)

/// Render appropriate empty state message
pub fn render_empty_state(
    area: Rect,
    buf: &mut Buffer,
    state: EmptyStateType,
    ctx: &RenderContext,
) {
    let (title, message, hint) = match state {
        EmptyStateType::NoAgents => (
            "No Agents Connected",
            "Waiting for agents to join the hive...",
            "Agents will appear here when they connect.",
        ),
        EmptyStateType::AllFiltered => (
            "No Matching Agents",
            "All agents are hidden by the current filter.",
            "Press '0' to clear filter, or '/' to modify.",
        ),
        EmptyStateType::ConnectionLost => (
            "Connection Lost",
            "Unable to receive agent updates.",
            "Check your connection and press 'r' to retry.",
        ),
    };

    let center_x = area.x + area.width / 2;
    let center_y = area.y + area.height / 2;

    // Title
    let title_style = Style::default()
        .fg(Color::Rgb(150, 150, 160))
        .add_modifier(Modifier::BOLD);
    render_centered_text(buf, title, center_x, center_y - 2, title_style);

    // Message
    let msg_style = Style::default().fg(Color::Rgb(100, 100, 110));
    render_centered_text(buf, message, center_x, center_y, msg_style);

    // Hint
    let hint_style = Style::default()
        .fg(Color::Rgb(80, 100, 120))
        .add_modifier(Modifier::DIM);
    render_centered_text(buf, hint, center_x, center_y + 2, hint_style);

    // Animated indicator (subtle)
    if !ctx.motion_pref.reduced_motion {
        let phase = (ctx.frame_count as f32 * 0.05).sin();
        let dot_y = center_y + 4;
        let dots = if phase > 0.3 { "..." } else if phase > -0.3 { ".." } else { "." };
        let dot_style = Style::default().fg(Color::Rgb(60, 60, 70));
        render_centered_text(buf, dots, center_x, dot_y, dot_style);
    }
}

fn render_centered_text(buf: &mut Buffer, text: &str, center_x: u16, y: u16, style: Style) {
    let start_x = center_x.saturating_sub(text.len() as u16 / 2);
    for (i, ch) in text.chars().enumerate() {
        buf[(start_x + i as u16, y)].set_char(ch).set_style(style);
    }
}

pub enum EmptyStateType {
    NoAgents,
    AllFiltered,
    ConnectionLost,
}
```

### 5.4 Help Overlay Content

```rust
// File: /Users/aaditshah/Documents/hive/src/render/help.rs (NEW)

/// Complete help overlay with all shortcuts
pub struct HelpOverlay;

impl HelpOverlay {
    pub fn render(area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        // Centered panel
        let panel_width = 50u16.min(area.width - 4);
        let panel_height = 22u16.min(area.height - 4);
        let panel_x = area.x + (area.width - panel_width) / 2;
        let panel_y = area.y + (area.height - panel_height) / 2;
        let panel = Rect::new(panel_x, panel_y, panel_width, panel_height);

        // Background with border
        let bg_style = Style::default().bg(Color::Rgb(30, 32, 40));
        let border_style = Style::default().fg(Color::Rgb(80, 100, 120));
        render_bordered_box(buf, panel, bg_style, border_style, ctx);

        let mut y = panel_y + 1;
        let x = panel_x + 2;

        // Title
        let title = "KEYBOARD SHORTCUTS";
        let title_style = Style::default()
            .fg(Color::Rgb(100, 200, 150))
            .add_modifier(Modifier::BOLD);
        render_text(buf, title, panel_x + (panel_width - title.len() as u16) / 2, y, title_style);
        y += 2;

        // Sections
        let sections = [
            ("DISPLAY", vec![
                ("m", "Cycle display mode (Minimal/Standard/Debug)"),
                ("1/2/3", "Set display mode directly"),
                ("s", "Toggle agent sidebar"),
            ]),
            ("NAVIGATION", vec![
                ("Tab", "Select next agent"),
                ("Shift+Tab", "Select previous agent"),
                ("Enter", "Focus selected agent"),
                ("Esc", "Clear selection / Exit"),
            ]),
            ("FILTERING", vec![
                ("/", "Enter filter mode (type to search)"),
                ("0", "Clear all filters"),
            ]),
            ("PLAYBACK", vec![
                ("Space", "Pause / Resume"),
                ("+/-", "Speed up / Slow down"),
            ]),
            ("ACCESSIBILITY", vec![
                ("F1", "Toggle high contrast"),
                ("F2", "Toggle reduced motion"),
                ("F3", "Toggle monochrome"),
            ]),
        ];

        let key_style = Style::default().fg(Color::Rgb(200, 200, 100));
        let desc_style = Style::default().fg(Color::Rgb(180, 180, 190));
        let section_style = Style::default()
            .fg(Color::Rgb(100, 150, 200))
            .add_modifier(Modifier::BOLD);

        for (section_name, shortcuts) in sections {
            if y >= panel_y + panel_height - 2 { break; }

            render_text(buf, section_name, x, y, section_style);
            y += 1;

            for (key, desc) in shortcuts {
                if y >= panel_y + panel_height - 2 { break; }

                let key_text = format!("{:>10}", key);
                render_text(buf, &key_text, x, y, key_style);
                render_text(buf, " - ", x + 10, y, desc_style);
                render_text(buf, desc, x + 13, y, desc_style);
                y += 1;
            }
            y += 1;
        }

        // Footer
        let footer = "Press ? or Esc to close";
        let footer_style = Style::default().fg(Color::Rgb(100, 100, 110));
        render_text(buf, footer, panel_x + (panel_width - footer.len() as u16) / 2, panel_y + panel_height - 1, footer_style);
    }
}

fn render_text(buf: &mut Buffer, text: &str, x: u16, y: u16, style: Style) {
    for (i, ch) in text.chars().enumerate() {
        if x + i as u16 < buf.area().width {
            buf[(x + i as u16, y)].set_char(ch).set_style(style);
        }
    }
}
```

### 5.5 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/render/activity_log.rs` | NEW - Event log strip |
| `src/render/empty_state.rs` | NEW - Empty state designs |
| `src/render/help.rs` | NEW - Help overlay with content |
| `src/input/mouse.rs` | Larger click targets |
| `src/render/mod.rs` | Add new modules |
| `src/app.rs` | State for new UI elements |

**Estimated Complexity:** 4-5 days

**Dependencies:** Phases 1-4

---

## Phase 6: Performance & Polish

**Goal:** Ensure smooth performance and polished experience.

### 6.1 Fixed Adaptive Frame Rate Logic

```rust
// File: /Users/aaditshah/Documents/hive/src/app.rs

/// Adaptive frame rate - FIXED logic
/// Low activity = lower FPS (save CPU)
/// High activity = higher FPS (smooth animation)
pub struct FrameRateController {
    target_fps: u32,
    min_fps: u32,
    max_fps: u32,
    last_frame_times: VecDeque<Duration>,
}

impl FrameRateController {
    pub fn new() -> Self {
        Self {
            target_fps: 30,
            min_fps: 15,
            max_fps: 60,
            last_frame_times: VecDeque::with_capacity(10),
        }
    }

    /// Calculate optimal FPS based on activity and performance
    /// FIXED: Higher activity = higher FPS (more CPU when needed for smooth animation)
    /// Lower activity = lower FPS (save CPU when idle)
    pub fn calculate_fps(&mut self, activity_level: f32, last_frame_time: Duration) -> u32 {
        self.last_frame_times.push_back(last_frame_time);
        if self.last_frame_times.len() > 10 {
            self.last_frame_times.pop_front();
        }

        // Target FPS based on activity
        // activity_level 0.0 -> min_fps (idle, save CPU)
        // activity_level 1.0 -> max_fps (busy, need smooth animation)
        let activity_fps = self.min_fps +
            ((self.max_fps - self.min_fps) as f32 * activity_level) as u32;

        // Check if we can sustain the target
        let avg_frame_time = self.last_frame_times.iter()
            .map(|d| d.as_millis() as f32)
            .sum::<f32>() / self.last_frame_times.len() as f32;

        let sustainable_fps = if avg_frame_time > 0.0 {
            (1000.0 / avg_frame_time) as u32
        } else {
            self.max_fps
        };

        // Use the lower of: desired FPS and sustainable FPS
        // This prevents frame drops by backing off when struggling
        activity_fps
            .min(sustainable_fps)
            .clamp(self.min_fps, self.max_fps)
    }
}
```

### 6.2 Adaptive Detail Level

```rust
// File: /Users/aaditshah/Documents/hive/src/render/adaptive.rs (NEW)

/// Reduce detail when many agents are visible
pub struct AdaptiveDetail {
    agent_count_thresholds: [usize; 3],
}

impl Default for AdaptiveDetail {
    fn default() -> Self {
        Self {
            agent_count_thresholds: [10, 25, 40],
        }
    }
}

impl AdaptiveDetail {
    pub fn get_detail_level(&self, agent_count: usize) -> DetailLevel {
        if agent_count <= self.agent_count_thresholds[0] {
            DetailLevel::Full
        } else if agent_count <= self.agent_count_thresholds[1] {
            DetailLevel::Reduced
        } else if agent_count <= self.agent_count_thresholds[2] {
            DetailLevel::Minimal
        } else {
            DetailLevel::Critical
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum DetailLevel {
    /// All effects: labels, trails, glow, animations
    Full,
    /// Labels for selected/hovered only, shorter trails
    Reduced,
    /// No labels except selected, no trails, no glow
    Minimal,
    /// Agents only, no decorations
    Critical,
}

impl DetailLevel {
    pub fn show_all_labels(&self) -> bool {
        matches!(self, DetailLevel::Full)
    }

    pub fn show_trails(&self) -> bool {
        matches!(self, DetailLevel::Full | DetailLevel::Reduced)
    }

    pub fn max_trail_length(&self) -> usize {
        match self {
            DetailLevel::Full => 20,
            DetailLevel::Reduced => 10,
            _ => 0,
        }
    }

    pub fn show_glow(&self) -> bool {
        matches!(self, DetailLevel::Full)
    }
}
```

### 6.3 First-Run Experience & Contextual Hints

```rust
// File: /Users/aaditshah/Documents/hive/src/render/hints.rs (NEW)

/// Contextual hints for first-time users
pub struct ContextualHints {
    hints_shown: HashSet<String>,
    hint_cooldown: HashMap<String, Instant>,
    first_run: bool,
}

impl ContextualHints {
    pub fn new(first_run: bool) -> Self {
        Self {
            hints_shown: HashSet::new(),
            hint_cooldown: HashMap::new(),
            first_run,
        }
    }

    /// Get hint to show based on current context
    pub fn get_hint(&mut self, context: &HintContext) -> Option<&'static str> {
        if !self.first_run {
            return None;
        }

        let (hint_key, hint_text) = match context {
            HintContext::NoSelection if !self.hints_shown.contains("select") => {
                ("select", "Tip: Press Tab to select an agent, or click on one")
            }
            HintContext::AgentSelected if !self.hints_shown.contains("detail") => {
                ("detail", "Tip: Press Enter to see agent details")
            }
            HintContext::ManyAgents if !self.hints_shown.contains("filter") => {
                ("filter", "Tip: Press / to filter agents by name")
            }
            HintContext::ErrorVisible if !self.hints_shown.contains("error") => {
                ("error", "Tip: Red agents have errors - select to see details")
            }
            HintContext::HighActivity if !self.hints_shown.contains("pause") => {
                ("pause", "Tip: Press Space to pause and examine the field")
            }
            _ => return None,
        };

        // Check cooldown (don't spam hints)
        if let Some(last_shown) = self.hint_cooldown.get(hint_key) {
            if last_shown.elapsed() < Duration::from_secs(30) {
                return None;
            }
        }

        self.hints_shown.insert(hint_key.to_string());
        self.hint_cooldown.insert(hint_key.to_string(), Instant::now());

        Some(hint_text)
    }

    pub fn dismiss_hints(&mut self) {
        self.first_run = false;
    }
}

pub enum HintContext {
    NoSelection,
    AgentSelected,
    ManyAgents,
    ErrorVisible,
    HighActivity,
}

/// Render hint at bottom of field
pub fn render_hint(hint: &str, area: Rect, buf: &mut Buffer) {
    let hint_y = area.y + area.height - 2;
    let hint_x = area.x + 2;

    let bg_style = Style::default().bg(Color::Rgb(40, 50, 60));
    let text_style = Style::default()
        .fg(Color::Rgb(200, 220, 255))
        .bg(Color::Rgb(40, 50, 60));

    // Background
    for x in hint_x..hint_x + hint.len() as u16 + 4 {
        buf[(x, hint_y)].set_style(bg_style);
    }

    // Text
    buf[(hint_x, hint_y)].set_char(' ').set_style(bg_style);
    buf[(hint_x + 1, hint_y)].set_char('i').set_style(
        text_style.fg(Color::Rgb(100, 150, 255))
    );
    buf[(hint_x + 2, hint_y)].set_char(' ').set_style(bg_style);

    for (i, ch) in hint.chars().enumerate() {
        buf[(hint_x + 3 + i as u16, hint_y)].set_char(ch).set_style(text_style);
    }
}
```

### 6.4 Configuration Support for Magic Numbers

```rust
// File: /Users/aaditshah/Documents/hive/src/config.rs (NEW)

use serde::{Deserialize, Serialize};

/// User-configurable settings (loaded from config file or environment)
#[derive(Clone, Deserialize, Serialize)]
pub struct HiveConfig {
    // Animation
    pub pulse_frequency: f32,
    pub pulse_amplitude: f32,
    pub breathe_frequency: f32,
    pub blink_interval: f32,

    // Collision
    pub min_agent_distance: f32,
    pub separation_force: f32,

    // Heatmap
    pub heatmap_decay_rate: f32,
    pub heatmap_threshold: f32,

    // Frame rate
    pub min_fps: u32,
    pub max_fps: u32,
    pub target_fps: u32,

    // UI
    pub sidebar_width: u16,
    pub activity_log_entries: usize,
    pub label_max_length: usize,

    // Accessibility
    pub default_color_mode: String,
    pub default_reduced_motion: bool,

    // Demo
    pub connection_probability: f32,
    pub connection_duration_secs: f32,
    pub swarm_interval_secs: f32,
}

impl Default for HiveConfig {
    fn default() -> Self {
        Self {
            pulse_frequency: 1.5,
            pulse_amplitude: 0.15,
            breathe_frequency: 0.5,
            blink_interval: 2.0,

            min_agent_distance: 0.08,
            separation_force: 0.5,

            heatmap_decay_rate: 0.98,
            heatmap_threshold: 0.02,

            min_fps: 15,
            max_fps: 60,
            target_fps: 30,

            sidebar_width: 24,
            activity_log_entries: 3,
            label_max_length: 12,

            default_color_mode: "normal".to_string(),
            default_reduced_motion: false,

            connection_probability: 0.15,
            connection_duration_secs: 3.0,
            swarm_interval_secs: 90.0,
        }
    }
}

impl HiveConfig {
    pub fn load() -> Self {
        // Try to load from file, fall back to defaults
        let config_path = dirs::config_dir()
            .map(|p| p.join("hive").join("config.toml"));

        if let Some(path) = config_path {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str(&contents) {
                    return config;
                }
            }
        }

        // Also check environment variables
        let mut config = Self::default();

        if let Ok(v) = std::env::var("HIVE_MIN_FPS") {
            config.min_fps = v.parse().unwrap_or(config.min_fps);
        }
        if let Ok(v) = std::env::var("HIVE_MAX_FPS") {
            config.max_fps = v.parse().unwrap_or(config.max_fps);
        }
        if let Ok(v) = std::env::var("HIVE_REDUCED_MOTION") {
            config.default_reduced_motion = v == "1" || v.to_lowercase() == "true";
        }

        config
    }
}
```

### 6.5 Integration Test Suite Notes

```rust
// File: /Users/aaditshah/Documents/hive/tests/integration/README.md content:
//
// Integration tests should cover:
// 1. Rendering with 0, 1, 10, 50 agents
// 2. All display modes
// 3. Filter application and clearing
// 4. Terminal resize handling
// 5. Mouse interaction (if supported)
// 6. Accessibility modes
// 7. Empty states
// 8. Error recovery
// 9. Unicode/ASCII fallback
// 10. Color mode switching
```

### 6.6 Implementation Details

**Files to Modify/Create:**
| File | Changes |
|------|---------|
| `src/app.rs` | Fixed adaptive frame rate |
| `src/render/adaptive.rs` | NEW - Adaptive detail levels |
| `src/render/hints.rs` | NEW - Contextual hints |
| `src/config.rs` | NEW - Configuration support |
| `tests/integration/` | NEW - Integration test structure |

**Estimated Complexity:** 4-5 days

**Dependencies:** Phases 1-5

---

## Demo Mode Polish (Phase 6 Continued)

### 6.7 Better Agent Personalities

```rust
// File: /Users/aaditshah/Documents/hive/src/demo.rs

/// Demo agent archetypes with distinct personalities
const DEMO_AGENTS: [DemoAgent; 6] = [
    DemoAgent {
        id: "Atlas",
        role: "Navigator",
        preferred_zones: &["api", "endpoint", "route"],
        behavior: Behavior::Methodical,
        messages: &[
            "Mapping API endpoints",
            "Tracing request flow",
            "Documenting routes",
        ],
    },
    DemoAgent {
        id: "Echo",
        role: "Investigator",
        preferred_zones: &["test", "debug", "error"],
        behavior: Behavior::Thorough,
        messages: &[
            "Running test suite",
            "Analyzing failures",
            "Checking edge cases",
        ],
    },
    DemoAgent {
        id: "Forge",
        role: "Builder",
        preferred_zones: &["component", "ui", "style"],
        behavior: Behavior::Creative,
        messages: &[
            "Building component",
            "Styling interface",
            "Creating layout",
        ],
    },
    DemoAgent {
        id: "Cipher",
        role: "Security",
        preferred_zones: &["auth", "jwt", "permission"],
        behavior: Behavior::Careful,
        messages: &[
            "Auditing permissions",
            "Validating tokens",
            "Checking access control",
        ],
    },
    DemoAgent {
        id: "Quantum",
        role: "Optimizer",
        preferred_zones: &["cache", "performance", "query"],
        behavior: Behavior::Fast,
        messages: &[
            "Optimizing query",
            "Caching results",
            "Reducing latency",
        ],
    },
    DemoAgent {
        id: "Nexus",
        role: "Coordinator",
        preferred_zones: &["core", "main", "config"],
        behavior: Behavior::Collaborative,
        messages: &[
            "Coordinating tasks",
            "Syncing state",
            "Managing workflow",
        ],
    },
];
```

### 6.8 Tutorial (Demo Mode Only)

```rust
// File: /Users/aaditshah/Documents/hive/src/render/tutorial.rs

const TUTORIAL_STEPS: [TutorialStep; 5] = [
    TutorialStep {
        message: "Welcome to Hive! You're watching AI agents collaborate in real-time.",
        highlight: None,
        duration: Duration::from_secs(5),
    },
    TutorialStep {
        message: "Each symbol is an agent. Colors show identity, shapes show type.",
        highlight: Some(Highlight::Agents),
        duration: Duration::from_secs(5),
    },
    TutorialStep {
        message: "Lines between agents show communication. Watch for patterns!",
        highlight: Some(Highlight::Connections),
        duration: Duration::from_secs(5),
    },
    TutorialStep {
        message: "Press 'm' to cycle display modes: Minimal, Standard, Debug.",
        highlight: Some(Highlight::StatusBar),
        duration: Duration::from_secs(5),
    },
    TutorialStep {
        message: "Press ? for all controls. Press any key to dismiss this tutorial.",
        highlight: None,
        duration: Duration::from_secs(8),
    },
];
```

---

## Success Criteria

### Phase 1: Foundation
- [ ] Layer-based rendering established and working
- [ ] All 8 agent colors pass WCAG AA contrast ratio against background
- [ ] Each status has a unique combination of color + shape + animation
- [ ] All symbols have Unicode codepoints AND ASCII fallbacks
- [ ] Labels and status indicators don't overlap (positioned correctly)
- [ ] Color module extracted and configurable
- [ ] Easing functions implemented and used
- [ ] Terminal capabilities detected correctly
- [ ] Rendering errors caught and handled gracefully

### Phase 2: Spatial Organization
- [ ] Zone labels derived from SemanticPositioner clusters
- [ ] Spatial hash implemented for O(n) collision detection
- [ ] No two agents overlap (minimum 3 character separation)
- [ ] Works with 40+ agents without performance degradation
- [ ] Agents near edges are still fully visible with labels
- [ ] Grid toggle works and aids spatial orientation

### Phase 3: Display Modes
- [ ] 3 display modes (Minimal, Standard, Debug) replace 6+ toggles
- [ ] Mode indicator visible in status bar
- [ ] Filter indicator visible when filter active
- [ ] Mode cycling works with 'm' key
- [ ] Direct mode selection works with 1/2/3 keys

### Phase 4: Animation
- [ ] Unified animation trait implemented
- [ ] Animation priority prevents visual competition
- [ ] Position transitions feel smooth (eased, no teleporting)
- [ ] Only highest-priority animations shown when many active
- [ ] Reduced motion mode disables all animations

### Phase 5: Information & Interactivity
- [ ] Activity log shows last 3 events legibly
- [ ] Mouse targets are 3x2 cells (easier to click)
- [ ] Empty states display appropriate messages
- [ ] Help overlay shows all keyboard shortcuts organized by category
- [ ] Tab cycles through agents
- [ ] Filter by name works
- [ ] All UI elements respect terminal size changes

### Phase 6: Performance & Polish
- [ ] Adaptive FPS: low activity = low FPS, high activity = high FPS
- [ ] Detail level reduces automatically with many agents
- [ ] First-run hints appear for new users
- [ ] Configuration file supported for magic numbers
- [ ] FPS stays above 30 during high activity with 40 agents
- [ ] Demo mode tutorial works
- [ ] Demo agent personalities are distinct

### Accessibility
- [ ] High contrast mode available (F1)
- [ ] Reduced motion mode available (F2)
- [ ] Monochrome mode available (F3)
- [ ] Screen reader summary generation implemented
- [ ] All interactions work via keyboard only

### Error Handling
- [ ] Small terminal shows minimal mode with summary
- [ ] Rendering errors show fallback UI
- [ ] Mouse errors fall back to keyboard navigation
- [ ] Unicode failures fall back to ASCII

---

## Appendix A: Symbol Reference

### Agent Shapes (Unicode / ASCII)
| Index | Name | Unicode | Codepoint | ASCII |
|-------|------|---------|-----------|-------|
| 0 | Diamond | ◆ | U+25C6 | < |
| 1 | Triangle Up | ▲ | U+25B2 | ^ |
| 2 | Square | ■ | U+25A0 | # |
| 3 | Triangle Down | ▼ | U+25BC | v |
| 4 | Pentagon | ⬟ | U+2B1F | * |
| 5 | Hexagon | ⬢ | U+2B22 | H |
| 6 | Star | ★ | U+2605 | * |
| 7 | Club | ♣ | U+2663 | & |

### Status Indicators (Unicode / ASCII)
| Status | Unicode | Codepoint | ASCII |
|--------|---------|-----------|-------|
| Active | • | U+2022 | * |
| Thinking | … | U+2026 | . |
| Waiting | ⧖ | U+29D6 | ~ |
| Idle | – | U+2013 | - |
| Error | ❗ | U+2757 | ! |

### Line Characters (Unicode / ASCII)
| Type | Unicode | Codepoint | ASCII |
|------|---------|-----------|-------|
| Horizontal | ─ | U+2500 | - |
| Vertical | │ | U+2502 | \| |
| Cross | ┼ | U+253C | + |
| Dot | · | U+00B7 | . |

---

## Appendix B: Color Reference

### Okabe-Ito Palette (Agent Colors)
| Index | Name | RGB | Hex |
|-------|------|-----|-----|
| 0 | Orange | (230, 159, 0) | #E69F00 |
| 1 | Sky Blue | (86, 180, 233) | #56B4E9 |
| 2 | Bluish Green | (0, 158, 115) | #009E73 |
| 3 | Yellow | (240, 228, 66) | #F0E442 |
| 4 | Blue | (0, 114, 178) | #0072B2 |
| 5 | Vermillion | (213, 94, 0) | #D55E00 |
| 6 | Reddish Purple | (204, 121, 167) | #CC79A7 |
| 7 | Gray | (170, 170, 170) | #AAAAAA |

### Status Colors
| Status | RGB | Hex |
|--------|-----|-----|
| Active | (0, 200, 100) | #00C864 |
| Thinking | (100, 150, 255) | #6496FF |
| Waiting | (255, 200, 80) | #FFC850 |
| Idle | (100, 100, 100) | #646464 |
| Error | (255, 80, 80) | #FF5050 |

---

## Appendix C: Keyboard Shortcut Reference

| Key | Action | Notes |
|-----|--------|-------|
| `q`, `Esc` | Quit (or clear selection) | Esc prioritizes clearing |
| `m` | Cycle display mode | Minimal -> Standard -> Debug |
| `1` | Set Minimal mode | Direct selection |
| `2` | Set Standard mode | Direct selection |
| `3` | Set Debug mode | Direct selection |
| `s` | Toggle sidebar | Independent of mode |
| `?` | Toggle help | Shows all shortcuts |
| `Tab` | Select next agent | |
| `Shift+Tab` | Select previous agent | |
| `Enter` | Focus selected agent | Opens detail panel |
| `/` | Enter filter mode | Type to search |
| `0` | Clear filter | |
| `Space` | Pause/Resume | |
| `+`/`-` | Speed up/down | |
| `F1` | Toggle high contrast | Accessibility |
| `F2` | Toggle reduced motion | Accessibility |
| `F3` | Toggle monochrome | Accessibility |

---

## Appendix D: Revised Complexity Estimates

| Phase | Description | Days | Cumulative |
|-------|-------------|------|------------|
| 1 | Foundation (Layers, Colors, Symbols, Easing, Error Handling) | 5-7 | 5-7 |
| 2 | Spatial Organization (Zones, Spatial Hash, Collision) | 3-4 | 8-11 |
| 3 | Display Modes (Mode System, Status Bar) | 2-3 | 10-14 |
| 4 | Animation (Unified Trait, Priority) | 3-4 | 13-18 |
| 5 | Information & Interactivity (Log, Help, Empty States, Mouse) | 4-5 | 17-23 |
| 6 | Performance & Polish (FPS, Adaptive Detail, Hints, Config, Demo) | 5-7 | 22-30 |
| | **Buffer for integration testing and bug fixes** | 4-6 | 26-36 |
| | **Total Estimate** | **26-36 days** | |

---

## Appendix E: File Structure After Implementation

```
src/
  main.rs
  app.rs                    # Core state, resize handling, frame rate
  config.rs                 # NEW - Configuration loading
  demo.rs                   # MODIFIED - Agent personalities, scenarios

  accessibility/            # NEW directory
    mod.rs
    screen_reader.rs        # NEW - Screen reader support
    motion.rs               # NEW - Reduced motion
    contrast.rs             # NEW - High contrast mode
    monochrome.rs           # NEW - Monochrome mode

  animation/
    mod.rs
    easing.rs               # NEW - Easing functions (moved to Phase 1)
    trait.rs                # NEW - Unified animation trait
    priority.rs             # NEW - Animation priority system
    pulse.rs                # MODIFIED - Use animation trait

  event/
    mod.rs
    types.rs
    watcher.rs
    queue.rs

  input/
    mod.rs
    handler.rs              # MODIFIED - Mode-based handling
    mouse.rs                # NEW - Mouse interaction

  positioning/
    mod.rs                  # MODIFIED - Use spatial hash
    semantic.rs             # MODIFIED - Larger clusters
    interpolation.rs        # MODIFIED - Eased lerp
    spatial_hash.rs         # NEW - O(n) collision detection

  render/
    mod.rs                  # MODIFIED - Add all new modules
    layers.rs               # NEW - Layer definitions
    colors.rs               # NEW - Extracted color module
    symbols.rs              # NEW - Unicode/ASCII symbols
    field.rs                # MODIFIED - Layer-based rendering
    agent.rs                # MODIFIED - Label/indicator positioning
    heatmap.rs              # MODIFIED - Configurable decay
    trails.rs
    connections.rs          # MODIFIED - Use animation trait
    ui.rs                   # MODIFIED - Mode indicator, filter indicator
    zones.rs                # NEW - Derived from positioner
    flash.rs                # NEW - Event flash effects
    activity_log.rs         # NEW - Event log strip
    sidebar.rs              # NEW - Agent list
    empty_state.rs          # NEW - Empty state designs
    help.rs                 # NEW - Help overlay content
    hints.rs                # NEW - Contextual hints
    tutorial.rs             # NEW - Demo tutorial
    adaptive.rs             # NEW - Adaptive detail levels
    error_recovery.rs       # NEW - Safe rendering

  state/
    mod.rs
    agent.rs                # MODIFIED - Animation styles
    field.rs                # MODIFIED - Use spatial hash
    history.rs
    filter.rs               # NEW - Filtering with persistence
    display_mode.rs         # NEW - 3 display modes

  terminal/                 # NEW directory
    mod.rs
    capabilities.rs         # NEW - Terminal detection

tests/
  integration/              # NEW - Integration test suite
    mod.rs
    rendering_tests.rs
    interaction_tests.rs
    accessibility_tests.rs
```

---

*Document Version: 2.0 (FINAL)*
*Last Updated: 2026-02-02*
*Revision based on: Technical Review + UX Review*
*Author: Lead Architect*
