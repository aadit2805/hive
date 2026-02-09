//! Color module for the Hive visualization.
//!
//! This module provides:
//! - Okabe-Ito colorblind-safe agent color palette
//! - Status colors for different agent states
//! - Color manipulation utilities
//! - Color mode support for different terminal capabilities

use ratatui::style::Color;

use crate::event::AgentStatus;

/// Color depth/mode for different terminal capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// Full 24-bit RGB color support
    #[default]
    TrueColor,
    /// 256 color mode (xterm-256color)
    Color256,
    /// Basic 16 ANSI colors
    Basic16,
    /// Monochrome mode (grayscale only)
    Monochrome,
}

/// Okabe-Ito colorblind-safe palette (8 colors)
///
/// This palette is designed to be distinguishable by people with
/// various forms of color vision deficiency.
///
/// Reference: https://jfly.uni-koeln.de/color/
pub const AGENT_COLORS: [Color; 8] = [
    Color::Rgb(0, 114, 178),    // Blue
    Color::Rgb(230, 159, 0),    // Orange
    Color::Rgb(0, 158, 115),    // Bluish Green
    Color::Rgb(240, 228, 66),   // Yellow
    Color::Rgb(86, 180, 233),   // Sky Blue
    Color::Rgb(213, 94, 0),     // Vermillion
    Color::Rgb(204, 121, 167),  // Reddish Purple
    Color::Rgb(136, 136, 136),  // Gray
];

/// 256-color fallback palette (using closest xterm-256 colors)
pub const AGENT_COLORS_256: [Color; 8] = [
    Color::Indexed(24),  // Blue (closest to 0, 114, 178)
    Color::Indexed(214), // Orange (closest to 230, 159, 0)
    Color::Indexed(36),  // Bluish Green (closest to 0, 158, 115)
    Color::Indexed(227), // Yellow (closest to 240, 228, 66)
    Color::Indexed(117), // Sky Blue (closest to 86, 180, 233)
    Color::Indexed(166), // Vermillion (closest to 213, 94, 0)
    Color::Indexed(175), // Reddish Purple (closest to 204, 121, 167)
    Color::Indexed(245), // Gray (closest to 136, 136, 136)
];

/// Basic 16-color fallback palette
pub const AGENT_COLORS_BASIC: [Color; 8] = [
    Color::Blue,
    Color::Yellow,
    Color::Green,
    Color::LightYellow,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    Color::Gray,
];

/// Monochrome palette (different grayscale levels)
pub const AGENT_COLORS_MONO: [Color; 8] = [
    Color::Rgb(255, 255, 255), // White
    Color::Rgb(220, 220, 220), // Light gray
    Color::Rgb(200, 200, 200), // Medium-light gray
    Color::Rgb(180, 180, 180), // Light-medium gray
    Color::Rgb(160, 160, 160), // Medium gray
    Color::Rgb(140, 140, 140), // Medium-dark gray
    Color::Rgb(120, 120, 120), // Dark gray
    Color::Rgb(100, 100, 100), // Darker gray
];

/// Status colors struct for different agent states
#[derive(Debug, Clone, Copy)]
pub struct StatusColors {
    /// Color for active/working agents
    pub active: Color,
    /// Color for thinking/processing agents
    pub thinking: Color,
    /// Color for waiting/blocked agents
    pub waiting: Color,
    /// Color for idle agents
    pub idle: Color,
    /// Color for agents in error state
    pub error: Color,
}

impl StatusColors {
    /// Get the color for a given agent status
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

/// Default status colors (TrueColor)
pub const STATUS_COLORS: StatusColors = StatusColors {
    active: Color::Rgb(0, 200, 100),     // Green - working
    thinking: Color::Rgb(100, 150, 255), // Blue - processing
    waiting: Color::Rgb(255, 200, 80),   // Amber - blocked
    idle: Color::Rgb(100, 100, 100),     // Gray - inactive
    error: Color::Rgb(255, 80, 80),      // Red - problem
};

/// Status colors for 256-color mode
pub const STATUS_COLORS_256: StatusColors = StatusColors {
    active: Color::Indexed(41),   // Green
    thinking: Color::Indexed(75), // Blue
    waiting: Color::Indexed(220), // Amber/Yellow
    idle: Color::Indexed(245),    // Gray
    error: Color::Indexed(196),   // Red
};

/// Status colors for basic 16-color mode
pub const STATUS_COLORS_BASIC: StatusColors = StatusColors {
    active: Color::Green,
    thinking: Color::Blue,
    waiting: Color::Yellow,
    idle: Color::Gray,
    error: Color::Red,
};

/// Status colors for monochrome mode
pub const STATUS_COLORS_MONO: StatusColors = StatusColors {
    active: Color::Rgb(255, 255, 255),   // Bright white
    thinking: Color::Rgb(200, 200, 200), // Light gray
    waiting: Color::Rgb(180, 180, 180),  // Medium gray (with blink)
    idle: Color::Rgb(100, 100, 100),     // Dark gray
    error: Color::Rgb(255, 255, 255),    // White (with rapid blink)
};

/// Dim a color by a factor (0.0 = black, 1.0 = unchanged)
///
/// # Arguments
/// * `color` - The color to dim
/// * `factor` - Dimming factor (0.0 to 1.0)
///
/// # Returns
/// The dimmed color. For non-RGB colors, returns the original color unchanged.
pub fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        // For non-RGB colors, return unchanged
        other => other,
    }
}

/// Get an agent color by index, wrapping around the palette
///
/// # Arguments
/// * `index` - The color index (will wrap around palette length)
///
/// # Returns
/// The color at the given index (modulo palette length)
pub fn get_agent_color(index: usize) -> Color {
    AGENT_COLORS[index % AGENT_COLORS.len()]
}

/// Get an agent color for a specific color mode
///
/// # Arguments
/// * `index` - The color index (will wrap around palette length)
/// * `mode` - The color mode to use
///
/// # Returns
/// The appropriate color for the given mode
pub fn get_agent_color_for_mode(index: usize, mode: ColorMode) -> Color {
    match mode {
        ColorMode::TrueColor => AGENT_COLORS[index % AGENT_COLORS.len()],
        ColorMode::Color256 => AGENT_COLORS_256[index % AGENT_COLORS_256.len()],
        ColorMode::Basic16 => AGENT_COLORS_BASIC[index % AGENT_COLORS_BASIC.len()],
        ColorMode::Monochrome => AGENT_COLORS_MONO[index % AGENT_COLORS_MONO.len()],
    }
}

/// Get status colors for a specific color mode
///
/// # Arguments
/// * `mode` - The color mode to use
///
/// # Returns
/// The appropriate StatusColors for the given mode
pub fn get_status_colors_for_mode(mode: ColorMode) -> &'static StatusColors {
    match mode {
        ColorMode::TrueColor => &STATUS_COLORS,
        ColorMode::Color256 => &STATUS_COLORS_256,
        ColorMode::Basic16 => &STATUS_COLORS_BASIC,
        ColorMode::Monochrome => &STATUS_COLORS_MONO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dim_color_rgb() {
        let color = Color::Rgb(100, 200, 50);
        let dimmed = dim_color(color, 0.5);
        assert_eq!(dimmed, Color::Rgb(50, 100, 25));
    }

    #[test]
    fn test_dim_color_full_brightness() {
        let color = Color::Rgb(100, 200, 50);
        let dimmed = dim_color(color, 1.0);
        assert_eq!(dimmed, Color::Rgb(100, 200, 50));
    }

    #[test]
    fn test_dim_color_zero() {
        let color = Color::Rgb(100, 200, 50);
        let dimmed = dim_color(color, 0.0);
        assert_eq!(dimmed, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_dim_color_non_rgb() {
        let color = Color::Blue;
        let dimmed = dim_color(color, 0.5);
        assert_eq!(dimmed, Color::Blue);
    }

    #[test]
    fn test_get_agent_color_wraps() {
        let color0 = get_agent_color(0);
        let color8 = get_agent_color(8);
        assert_eq!(color0, color8);
    }

    #[test]
    fn test_status_colors_get() {
        assert_eq!(STATUS_COLORS.get(AgentStatus::Active), STATUS_COLORS.active);
        assert_eq!(STATUS_COLORS.get(AgentStatus::Error), STATUS_COLORS.error);
    }

    #[test]
    fn test_color_mode_for_mode() {
        let true_color = get_agent_color_for_mode(0, ColorMode::TrueColor);
        let basic_color = get_agent_color_for_mode(0, ColorMode::Basic16);

        assert_eq!(true_color, AGENT_COLORS[0]);
        assert_eq!(basic_color, AGENT_COLORS_BASIC[0]);
    }
}
