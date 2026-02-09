//! Symbol system with Unicode and ASCII fallbacks
//!
//! This module provides a unified symbol system that supports both Unicode
//! characters for modern terminals and ASCII fallbacks for limited environments.

use crate::event::AgentStatus;

/// Symbol with Unicode and ASCII fallback
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    /// Unicode character for modern terminals
    pub unicode: char,
    /// ASCII fallback for limited terminals
    pub ascii: char,
    /// Human-readable name for the symbol
    pub name: &'static str,
}

impl Symbol {
    /// Create a new symbol with Unicode and ASCII variants
    pub const fn new(unicode: char, ascii: char, name: &'static str) -> Self {
        Self {
            unicode,
            ascii,
            name,
        }
    }

    /// Render the appropriate character based on Unicode support
    pub fn render(&self, use_unicode: bool) -> char {
        if use_unicode {
            self.unicode
        } else {
            self.ascii
        }
    }
}

/// Agent shape symbols (identity - based on shape_index)
/// These 8 shapes provide unique visual identities for agents
pub const AGENT_SHAPES: [Symbol; 8] = [
    Symbol::new('\u{25C6}', '<', "diamond"),       // U+25C6 Black Diamond (◆)
    Symbol::new('\u{25B2}', '^', "triangle_up"),   // U+25B2 Black Up Triangle (▲)
    Symbol::new('\u{25A0}', '#', "square"),        // U+25A0 Black Square (■)
    Symbol::new('\u{2B1F}', '*', "pentagon"),      // U+2B1F Black Pentagon (⬟)
    Symbol::new('\u{2B22}', 'H', "hexagon"),       // U+2B22 Black Hexagon (⬢)
    Symbol::new('\u{25CF}', 'O', "circle"),        // U+25CF Black Circle (●)
    Symbol::new('\u{2605}', '*', "star"),          // U+2605 Black Star (★)
    Symbol::new('\u{25BC}', 'v', "triangle_down"), // U+25BC Black Down Triangle (▼)
];

/// Status indicator symbols
pub struct StatusSymbols {
    pub active: Symbol,
    pub thinking: Symbol,
    pub waiting: Symbol,
    pub idle: Symbol,
    pub error: Symbol,
}

impl StatusSymbols {
    /// Get the symbol for a given agent status
    pub fn get(&self, status: &AgentStatus) -> &Symbol {
        match status {
            AgentStatus::Active => &self.active,
            AgentStatus::Thinking => &self.thinking,
            AgentStatus::Waiting => &self.waiting,
            AgentStatus::Idle => &self.idle,
            AgentStatus::Error => &self.error,
        }
    }
}

/// Status indicator symbols (single character for layout consistency)
pub const STATUS_INDICATORS: StatusSymbols = StatusSymbols {
    active: Symbol::new('\u{2022}', '*', "active"),     // U+2022 Bullet (•)
    thinking: Symbol::new('\u{2026}', '.', "thinking"), // U+2026 Ellipsis (…)
    waiting: Symbol::new('\u{29D6}', '~', "waiting"),   // U+29D6 Hourglass (⧖)
    idle: Symbol::new('\u{2013}', '-', "idle"),         // U+2013 En Dash (–)
    error: Symbol::new('\u{2757}', '!', "error"),       // U+2757 Exclamation (❗)
};

/// Trail character set for rendering agent movement trails
pub struct TrailCharset {
    pub recent: Symbol,
    pub medium: Symbol,
    pub faded: Symbol,
}

impl TrailCharset {
    /// Get trail symbol based on age (0.0 = recent, 1.0 = old)
    pub fn get_by_age(&self, age: f32) -> &Symbol {
        if age < 0.33 {
            &self.recent
        } else if age < 0.66 {
            &self.medium
        } else {
            &self.faded
        }
    }
}

/// Trail characters for movement visualization
pub const TRAIL_SYMBOLS: TrailCharset = TrailCharset {
    recent: Symbol::new('\u{2022}', 'o', "trail_recent"), // U+2022 Bullet (•)
    medium: Symbol::new('\u{00B7}', '.', "trail_medium"), // U+00B7 Middle Dot (·)
    faded: Symbol::new('\u{2219}', '.', "trail_faded"),   // U+2219 Bullet Operator (∙)
};

/// Line character set for connection rendering
pub struct LineCharset {
    pub horizontal: Symbol,
    pub vertical: Symbol,
    pub cross: Symbol,
    pub dot: Symbol,
    pub arrow_right: Symbol,
    pub arrow_left: Symbol,
    pub arrow_up: Symbol,
    pub arrow_down: Symbol,
}

/// Line characters for connections between agents
pub const LINE_CHARS: LineCharset = LineCharset {
    horizontal: Symbol::new('\u{2500}', '-', "h_line"),   // U+2500 Box Light Horizontal (─)
    vertical: Symbol::new('\u{2502}', '|', "v_line"),     // U+2502 Box Light Vertical (│)
    cross: Symbol::new('\u{253C}', '+', "cross"),         // U+253C Box Light Cross (┼)
    dot: Symbol::new('\u{00B7}', '.', "dot"),             // U+00B7 Middle Dot (·)
    arrow_right: Symbol::new('\u{25B6}', '>', "arrow_r"), // U+25B6 Right Triangle (▶)
    arrow_left: Symbol::new('\u{25C0}', '<', "arrow_l"),  // U+25C0 Left Triangle (◀)
    arrow_up: Symbol::new('\u{25B2}', '^', "arrow_u"),    // U+25B2 Up Triangle (▲)
    arrow_down: Symbol::new('\u{25BC}', 'v', "arrow_d"),  // U+25BC Down Triangle (▼)
};

/// Detect if the terminal supports Unicode characters
///
/// Checks environment variables for UTF-8 support indicators:
/// - LANG and LC_ALL for UTF-8 locale
/// - TERM_PROGRAM for known Unicode-capable terminals
pub fn detect_unicode() -> bool {
    // Check LANG environment variable
    if let Ok(lang) = std::env::var("LANG") {
        if lang.to_lowercase().contains("utf") {
            return true;
        }
    }

    // Check LC_ALL environment variable
    if let Ok(lc_all) = std::env::var("LC_ALL") {
        if lc_all.to_lowercase().contains("utf") {
            return true;
        }
    }

    // Check LC_CTYPE environment variable
    if let Ok(lc_ctype) = std::env::var("LC_CTYPE") {
        if lc_ctype.to_lowercase().contains("utf") {
            return true;
        }
    }

    // Check for known Unicode-capable terminals
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let unicode_terminals = [
            "iTerm.app",
            "Apple_Terminal",
            "vscode",
            "Hyper",
            "Alacritty",
            "kitty",
            "WezTerm",
        ];
        if unicode_terminals
            .iter()
            .any(|t| term_program.contains(t))
        {
            return true;
        }
    }

    // Check TERM for common Unicode-capable terminal types
    if let Ok(term) = std::env::var("TERM") {
        let unicode_terms = ["xterm", "screen", "tmux", "rxvt"];
        if unicode_terms.iter().any(|t| term.contains(t)) {
            return true;
        }
    }

    // Default to false if we can't determine Unicode support
    false
}

/// Get the agent shape symbol for a given shape index
pub fn get_agent_shape(shape_index: usize) -> &'static Symbol {
    &AGENT_SHAPES[shape_index % AGENT_SHAPES.len()]
}

/// Get the status indicator symbol for a given status
pub fn get_status_indicator(status: &AgentStatus) -> &'static Symbol {
    STATUS_INDICATORS.get(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_render() {
        let sym = Symbol::new('\u{25C6}', '<', "diamond");
        assert_eq!(sym.render(true), '\u{25C6}');
        assert_eq!(sym.render(false), '<');
    }

    #[test]
    fn test_agent_shapes_count() {
        assert_eq!(AGENT_SHAPES.len(), 8);
    }

    #[test]
    fn test_get_agent_shape_wraps() {
        // Test that shape index wraps around
        let shape0 = get_agent_shape(0);
        let shape8 = get_agent_shape(8);
        assert_eq!(shape0.name, shape8.name);
    }

    #[test]
    fn test_status_indicators() {
        assert_eq!(STATUS_INDICATORS.get(&AgentStatus::Active).name, "active");
        assert_eq!(STATUS_INDICATORS.get(&AgentStatus::Thinking).name, "thinking");
        assert_eq!(STATUS_INDICATORS.get(&AgentStatus::Waiting).name, "waiting");
        assert_eq!(STATUS_INDICATORS.get(&AgentStatus::Idle).name, "idle");
        assert_eq!(STATUS_INDICATORS.get(&AgentStatus::Error).name, "error");
    }

    #[test]
    fn test_trail_by_age() {
        assert_eq!(TRAIL_SYMBOLS.get_by_age(0.1).name, "trail_recent");
        assert_eq!(TRAIL_SYMBOLS.get_by_age(0.5).name, "trail_medium");
        assert_eq!(TRAIL_SYMBOLS.get_by_age(0.9).name, "trail_faded");
    }
}
