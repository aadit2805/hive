pub mod activity_log;
pub mod agent;
pub mod agent_panel;
pub mod colors;
pub mod connections;
pub mod display_mode;
pub mod field;
pub mod heatmap;
pub mod layers;
pub mod symbols;
pub mod trails;
pub mod ui;

use ratatui::style::Color;

pub use activity_log::{ActivityEntry, ActivityLog, ActivityLogWidget};
pub use agent::render_agents;
pub use agent_panel::AgentPanel;
pub use connections::render_connections;
pub use display_mode::DisplayMode;
pub use field::render_field;
pub use heatmap::{HeatMap, HeatmapConfig};
pub use layers::{LayerRenderer, LayerVisibility, RenderLayer, RenderState};
pub use trails::render_trails;
pub use ui::{render_ui, EmptyStateType, EmptyStateWidget};

// Re-export colors module items for backward compatibility
pub use colors::{
    AGENT_COLORS, STATUS_COLORS, StatusColors, ColorMode,
    dim_color, get_agent_color,
};

// Re-export symbols module items
pub use symbols::{
    Symbol, AGENT_SHAPES, STATUS_INDICATORS, TRAIL_SYMBOLS, LINE_CHARS,
    detect_unicode, get_agent_shape, get_status_indicator,
};

/// Get color for an agent based on index (backward compatibility alias)
pub fn agent_color(index: usize) -> ratatui::style::Color {
    get_agent_color(index)
}

/// Interpolate between two colors
pub fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    let (r1, g1, b1) = match from {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    };

    let (r2, g2, b2) = match to {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (255.0, 255.0, 255.0),
    };

    Color::Rgb(
        (r1 + (r2 - r1) * t) as u8,
        (g1 + (g2 - g1) * t) as u8,
        (b1 + (b2 - b1) * t) as u8,
    )
}
