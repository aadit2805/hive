//! Layer-based rendering system for Hive visualization.
//!
//! Layers render in strict z-order: lower enum values render first (background),
//! higher values render last (foreground/overlays). This ensures consistent
//! visual hierarchy with proper element visibility.

use ratatui::{buffer::Buffer, layout::Rect};
use std::collections::HashMap;

use crate::event::LandmarkId;
use crate::positioning::Position;
use crate::state::field::{ActiveConnection, StoredLandmark};
use crate::state::{Agent, History};

use super::{
    agent::AgentsWidget, connections::ConnectionsWidget, display_mode::DisplayMode,
    field::FieldWidget, heatmap::HeatMapWidget, trails::TrailsWidget, ui::HelpOverlay,
    ui::StatusBar, ui::TimelineWidget, HeatMap,
};

/// Render layers in strict z-order.
///
/// Elements on higher layers (larger enum values) render on top of
/// elements on lower layers. The order is:
///
/// 1. Background - base layer with grid and zone fills
/// 2. Zones - semantic zone boundaries
/// 3. Grid - optional grid lines
/// 4. Heatmap - activity heat visualization
/// 5. Trails - agent movement history
/// 6. Connections - lines between communicating agents
/// 7. Flashes - temporary event indicators
/// 8. Agents - agent symbols (primary content)
/// 9. Labels - agent name labels
/// 10. StatusIndicators - status symbols above agents
/// 11. UI - status bar and chrome
/// 12. Overlays - tooltips, help panels, modals
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RenderLayer {
    /// Base background layer (field border, zone fills)
    Background = 0,
    /// Semantic zone boundaries and labels
    Zones = 1,
    /// Optional grid overlay
    Grid = 2,
    /// Activity heatmap visualization
    Heatmap = 3,
    /// Agent movement trails
    Trails = 4,
    /// Connection lines between agents
    Connections = 5,
    /// Temporary event flash indicators
    Flashes = 6,
    /// Agent symbols (primary visual elements)
    Agents = 7,
    /// Agent name labels
    Labels = 8,
    /// Status indicator symbols
    StatusIndicators = 9,
    /// UI chrome (status bar, etc.)
    UI = 10,
    /// Overlays (help, tooltips, modals)
    Overlays = 11,
}

impl RenderLayer {
    /// Get all layers in render order (background to foreground).
    pub const fn all() -> [RenderLayer; 12] {
        [
            RenderLayer::Background,
            RenderLayer::Zones,
            RenderLayer::Grid,
            RenderLayer::Heatmap,
            RenderLayer::Trails,
            RenderLayer::Connections,
            RenderLayer::Flashes,
            RenderLayer::Agents,
            RenderLayer::Labels,
            RenderLayer::StatusIndicators,
            RenderLayer::UI,
            RenderLayer::Overlays,
        ]
    }

    /// Get the layer's z-index value.
    pub const fn z_index(self) -> u8 {
        self as u8
    }

    /// Check if this layer should render above another.
    pub fn renders_above(self, other: RenderLayer) -> bool {
        self.z_index() > other.z_index()
    }
}

/// Configuration for which layers are enabled.
#[derive(Debug, Clone)]
pub struct LayerVisibility {
    enabled: [bool; 12],
}

impl Default for LayerVisibility {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerVisibility {
    /// Create new visibility config with all layers enabled by default.
    pub fn new() -> Self {
        Self {
            enabled: [true; 12],
        }
    }

    /// Check if a layer is visible.
    pub fn is_visible(&self, layer: RenderLayer) -> bool {
        self.enabled[layer.z_index() as usize]
    }

    /// Enable or disable a layer.
    pub fn set_visible(&mut self, layer: RenderLayer, visible: bool) {
        self.enabled[layer.z_index() as usize] = visible;
    }

    /// Toggle a layer's visibility.
    pub fn toggle(&mut self, layer: RenderLayer) {
        let idx = layer.z_index() as usize;
        self.enabled[idx] = !self.enabled[idx];
    }
}

/// Manages ordered layer rendering for the Hive visualization.
///
/// The LayerRenderer ensures all visual elements render in the correct
/// z-order, with background elements first and overlays last. This
/// prevents visual artifacts like agents being hidden behind heatmaps
/// or connections obscuring labels.
pub struct LayerRenderer<'a> {
    /// Render area for the field (excludes UI chrome)
    field_area: Rect,
    /// Full render area (includes UI chrome)
    full_area: Rect,
    /// Which layers are currently visible
    visibility: &'a LayerVisibility,
}

impl<'a> LayerRenderer<'a> {
    /// Create a new layer renderer.
    ///
    /// # Arguments
    /// * `full_area` - Complete render area including status bar
    /// * `field_area` - Field-only area (excludes status bar)
    /// * `visibility` - Configuration for which layers to render
    pub fn new(full_area: Rect, field_area: Rect, visibility: &'a LayerVisibility) -> Self {
        Self {
            field_area,
            full_area,
            visibility,
        }
    }

    /// Render all layers in order.
    ///
    /// This is the main entry point for layer-based rendering. It renders
    /// each enabled layer in z-order, ensuring proper visual hierarchy.
    pub fn render_all(
        &self,
        buf: &mut Buffer,
        state: &RenderState<'_>,
    ) {
        for layer in RenderLayer::all() {
            if self.visibility.is_visible(layer) {
                self.render_layer(layer, buf, state);
            }
        }
    }

    /// Render a single layer.
    fn render_layer(
        &self,
        layer: RenderLayer,
        buf: &mut Buffer,
        state: &RenderState<'_>,
    ) {
        match layer {
            RenderLayer::Background => self.render_background(buf, state),
            RenderLayer::Zones => self.render_zones(buf, state),
            RenderLayer::Grid => self.render_grid(buf, state),
            RenderLayer::Heatmap => self.render_heatmap(buf, state),
            RenderLayer::Trails => self.render_trails(buf, state),
            RenderLayer::Connections => self.render_connections(buf, state),
            RenderLayer::Flashes => self.render_flashes(buf, state),
            RenderLayer::Agents => self.render_agents(buf, state),
            RenderLayer::Labels => self.render_labels(buf, state),
            RenderLayer::StatusIndicators => self.render_status_indicators(buf, state),
            RenderLayer::UI => self.render_ui(buf, state),
            RenderLayer::Overlays => self.render_overlays(buf, state),
        }
    }

    /// Layer 0: Background (field border)
    fn render_background(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;
        FieldWidget::new(state.landmarks).render(self.field_area, buf);
    }

    /// Layer 1: Zones (semantic zone labels - currently part of field)
    fn render_zones(&self, _buf: &mut Buffer, _state: &RenderState<'_>) {
        // Zone labels are currently rendered as part of the FieldWidget.
        // Future enhancement: separate zone rendering for better control.
    }

    /// Layer 2: Grid (optional grid overlay)
    fn render_grid(&self, _buf: &mut Buffer, _state: &RenderState<'_>) {
        // Grid rendering is a future enhancement.
        // Placeholder for optional grid overlay.
    }

    /// Layer 3: Heatmap
    fn render_heatmap(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        if let Some(heatmap) = state.heatmap {
            use ratatui::widgets::Widget;
            HeatMapWidget::new(heatmap).render(self.field_area, buf);
        }
    }

    /// Layer 4: Trails
    fn render_trails(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;
        TrailsWidget::new(state.agents.to_vec()).render(self.field_area, buf);
    }

    /// Layer 5: Connections
    fn render_connections(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;
        let get_position = state.get_agent_position;
        ConnectionsWidget::new(state.connections, get_position).render(self.field_area, buf);
    }

    /// Layer 6: Event flashes
    fn render_flashes(&self, _buf: &mut Buffer, _state: &RenderState<'_>) {
        // Flash rendering is a future enhancement.
        // Will show temporary visual indicators for events.
    }

    /// Layer 7: Agents
    fn render_agents(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;
        AgentsWidget::new(state.agents.to_vec())
            .selected(state.selected_agent)
            .hovered(state.hovered_agent)
            .render(self.field_area, buf);
    }

    /// Layer 8: Labels (currently rendered with agents)
    fn render_labels(&self, _buf: &mut Buffer, _state: &RenderState<'_>) {
        // Agent labels are currently rendered as part of AgentsWidget.
        // Future enhancement: separate label layer for better positioning.
    }

    /// Layer 9: Status indicators (currently rendered with agents)
    fn render_status_indicators(&self, _buf: &mut Buffer, _state: &RenderState<'_>) {
        // Status indicators are currently rendered as part of agent symbols.
        // Future enhancement: separate status indicator layer.
    }

    /// Layer 10: UI chrome
    fn render_ui(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;

        // Status bar at bottom
        let status_area = Rect::new(
            self.full_area.x,
            self.full_area.y + self.full_area.height - 1,
            self.full_area.width,
            1,
        );

        StatusBar::new(state.agents)
            .paused(state.paused)
            .playback_speed(state.playback_speed)
            .replay_mode(state.history.replay_mode, state.history.position())
            .fps(state.fps)
            .display_mode(state.display_mode)
            .render(status_area, buf);

        // Timeline when in replay mode
        if state.history.replay_mode {
            let timeline_area = Rect::new(
                self.full_area.x,
                self.full_area.y + self.full_area.height - 2,
                self.full_area.width,
                1,
            );
            TimelineWidget::new(state.history).render(timeline_area, buf);
        }
    }

    /// Layer 11: Overlays (help, tooltips)
    fn render_overlays(&self, buf: &mut Buffer, state: &RenderState<'_>) {
        use ratatui::widgets::Widget;
        use ratatui::style::{Color, Modifier, Style};

        if state.show_help {
            HelpOverlay.render(self.full_area, buf);
        }

        // Render filter bar when filter mode is active or filter text exists
        if let Some(filter_text) = state.filter_text {
            self.render_filter_bar(buf, filter_text, state.filter_mode);
        }
    }

    /// Render the filter input bar at the top of the screen
    fn render_filter_bar(&self, buf: &mut Buffer, filter_text: &str, is_editing: bool) {
        use ratatui::style::{Color, Modifier, Style};

        // Filter bar at top of field area
        let bar_y = self.field_area.y;
        let bar_width = self.field_area.width.min(40);
        let bar_x = self.field_area.x + 1;

        // Background
        let bg_style = if is_editing {
            Style::default().bg(Color::Rgb(40, 40, 60))
        } else {
            Style::default().bg(Color::Rgb(30, 30, 45))
        };

        for x in bar_x..bar_x + bar_width {
            if x < buf.area.width && bar_y < buf.area.height {
                buf[(x, bar_y)].set_char(' ').set_style(bg_style);
            }
        }

        // Label
        let label = if is_editing { "Filter: " } else { "Filter: " };
        let label_style = Style::default()
            .fg(Color::Rgb(150, 200, 255))
            .add_modifier(Modifier::BOLD);

        let mut x = bar_x;
        for ch in label.chars() {
            if x < bar_x + bar_width && x < buf.area.width {
                buf[(x, bar_y)].set_char(ch).set_style(label_style);
                x += 1;
            }
        }

        // Filter text
        let text_style = Style::default().fg(Color::Rgb(220, 220, 240));
        for ch in filter_text.chars() {
            if x < bar_x + bar_width - 1 && x < buf.area.width {
                buf[(x, bar_y)].set_char(ch).set_style(text_style);
                x += 1;
            }
        }

        // Cursor when editing
        if is_editing && x < bar_x + bar_width - 1 && x < buf.area.width {
            let cursor_style = Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .add_modifier(Modifier::RAPID_BLINK);
            buf[(x, bar_y)].set_char('_').set_style(cursor_style);
        }

        // Hint at the end
        if !is_editing {
            let hint = " [0:clear]";
            let hint_style = Style::default().fg(Color::Rgb(100, 100, 120));
            let hint_x = bar_x + bar_width - hint.len() as u16;
            let mut hx = hint_x;
            for ch in hint.chars() {
                if hx < buf.area.width && bar_y < buf.area.height {
                    buf[(hx, bar_y)].set_char(ch).set_style(hint_style);
                    hx += 1;
                }
            }
        }
    }
}

/// State needed for rendering all layers.
///
/// This struct collects all the data needed to render the visualization,
/// passed to the LayerRenderer for organized rendering.
pub struct RenderState<'a> {
    /// All agents to render
    pub agents: &'a [&'a Agent],
    /// Currently selected agent ID
    pub selected_agent: Option<&'a str>,
    /// Currently hovered agent ID (for highlighting)
    pub hovered_agent: Option<&'a str>,
    /// Heatmap data (optional, based on display toggle)
    pub heatmap: Option<&'a HeatMap>,
    /// Active connections between agents
    pub connections: &'a [ActiveConnection],
    /// Function to get agent position by ID
    pub get_agent_position: &'a dyn Fn(&str) -> Option<Position>,
    /// Landmarks on the field
    pub landmarks: &'a HashMap<LandmarkId, StoredLandmark>,
    /// History for replay mode
    pub history: &'a History,
    /// Whether simulation is paused
    pub paused: bool,
    /// Playback speed multiplier
    pub playback_speed: f32,
    /// Whether help overlay is shown
    pub show_help: bool,
    /// Current frames per second
    pub fps: u32,
    /// Current display mode
    pub display_mode: DisplayMode,
    /// Current filter text (None if not filtering)
    pub filter_text: Option<&'a str>,
    /// Whether filter mode is active (typing)
    pub filter_mode: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        assert!(RenderLayer::Background < RenderLayer::Agents);
        assert!(RenderLayer::Agents < RenderLayer::UI);
        assert!(RenderLayer::UI < RenderLayer::Overlays);
    }

    #[test]
    fn test_layer_z_index() {
        assert_eq!(RenderLayer::Background.z_index(), 0);
        assert_eq!(RenderLayer::Overlays.z_index(), 11);
    }

    #[test]
    fn test_renders_above() {
        assert!(RenderLayer::Overlays.renders_above(RenderLayer::UI));
        assert!(RenderLayer::Agents.renders_above(RenderLayer::Heatmap));
        assert!(!RenderLayer::Background.renders_above(RenderLayer::Agents));
    }

    #[test]
    fn test_layer_visibility() {
        let mut visibility = LayerVisibility::new();

        // All layers visible by default
        assert!(visibility.is_visible(RenderLayer::Heatmap));
        assert!(visibility.is_visible(RenderLayer::Trails));

        // Toggle heatmap off
        visibility.set_visible(RenderLayer::Heatmap, false);
        assert!(!visibility.is_visible(RenderLayer::Heatmap));

        // Toggle trails
        visibility.toggle(RenderLayer::Trails);
        assert!(!visibility.is_visible(RenderLayer::Trails));
        visibility.toggle(RenderLayer::Trails);
        assert!(visibility.is_visible(RenderLayer::Trails));
    }

    #[test]
    fn test_all_layers_in_order() {
        let layers = RenderLayer::all();
        assert_eq!(layers.len(), 12);
        assert_eq!(layers[0], RenderLayer::Background);
        assert_eq!(layers[11], RenderLayer::Overlays);

        // Verify monotonic ordering
        for i in 1..layers.len() {
            assert!(layers[i] > layers[i - 1]);
        }
    }
}
