//! Display mode presets for Hive visualization.
//!
//! Instead of managing individual layer toggles, users can select from
//! three preset display modes that configure all layers appropriately:
//!
//! - **Minimal**: Clean view with agents and labels only
//! - **Standard**: Balanced view with connections, trails, and activity
//! - **Debug**: Full diagnostic view showing all available information

use super::{LayerVisibility, RenderLayer};

/// Display mode presets for the visualization.
///
/// Each mode configures layer visibility for a specific use case:
/// - Minimal: Focus on agent positions and identity
/// - Standard: Balanced view for typical monitoring
/// - Debug: Full visibility for troubleshooting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Minimal mode: agents + labels only.
    /// Best for clean screenshots or when you need to focus on agent positions.
    Minimal,

    /// Standard mode (default): agents + connections + trails + activity.
    /// Balanced view suitable for most monitoring scenarios.
    #[default]
    Standard,

    /// Debug mode: everything visible.
    /// Shows heatmap, grid, trails, connections, landmarks - full diagnostic view.
    Debug,
}

impl DisplayMode {
    /// Get the layer visibility configuration for this display mode.
    ///
    /// Returns a `LayerVisibility` struct with appropriate layers enabled
    /// for the current mode.
    pub fn layer_visibility(&self) -> LayerVisibility {
        let mut visibility = LayerVisibility::new();

        // First, disable all optional layers
        // Background, Agents, Labels, StatusIndicators, UI, Overlays are always on
        visibility.set_visible(RenderLayer::Zones, false);
        visibility.set_visible(RenderLayer::Grid, false);
        visibility.set_visible(RenderLayer::Heatmap, false);
        visibility.set_visible(RenderLayer::Trails, false);
        visibility.set_visible(RenderLayer::Connections, false);
        visibility.set_visible(RenderLayer::Flashes, false);

        match self {
            DisplayMode::Minimal => {
                // Minimal: Only agents and labels (already disabled all optional layers)
                // Background, Agents, Labels, StatusIndicators, UI, Overlays remain on
            }

            DisplayMode::Standard => {
                // Standard: agents + connections + trails + activity indicators
                visibility.set_visible(RenderLayer::Trails, true);
                visibility.set_visible(RenderLayer::Connections, true);
                visibility.set_visible(RenderLayer::Flashes, true);
            }

            DisplayMode::Debug => {
                // Debug: everything visible
                visibility.set_visible(RenderLayer::Zones, true);
                visibility.set_visible(RenderLayer::Grid, true);
                visibility.set_visible(RenderLayer::Heatmap, true);
                visibility.set_visible(RenderLayer::Trails, true);
                visibility.set_visible(RenderLayer::Connections, true);
                visibility.set_visible(RenderLayer::Flashes, true);
            }
        }

        visibility
    }

    /// Cycle to the next display mode.
    ///
    /// Order: Minimal -> Standard -> Debug -> Minimal
    pub fn cycle(&self) -> DisplayMode {
        match self {
            DisplayMode::Minimal => DisplayMode::Standard,
            DisplayMode::Standard => DisplayMode::Debug,
            DisplayMode::Debug => DisplayMode::Minimal,
        }
    }

    /// Get the display name for this mode.
    pub fn name(&self) -> &'static str {
        match self {
            DisplayMode::Minimal => "Minimal",
            DisplayMode::Standard => "Standard",
            DisplayMode::Debug => "Debug",
        }
    }

    /// Get a short description of what this mode shows.
    pub fn description(&self) -> &'static str {
        match self {
            DisplayMode::Minimal => "agents + labels",
            DisplayMode::Standard => "agents + trails + connections",
            DisplayMode::Debug => "all layers visible",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_default() {
        assert_eq!(DisplayMode::default(), DisplayMode::Standard);
    }

    #[test]
    fn test_cycle_order() {
        assert_eq!(DisplayMode::Minimal.cycle(), DisplayMode::Standard);
        assert_eq!(DisplayMode::Standard.cycle(), DisplayMode::Debug);
        assert_eq!(DisplayMode::Debug.cycle(), DisplayMode::Minimal);
    }

    #[test]
    fn test_minimal_mode_layers() {
        let visibility = DisplayMode::Minimal.layer_visibility();

        // Should have agents visible
        assert!(visibility.is_visible(RenderLayer::Agents));
        assert!(visibility.is_visible(RenderLayer::Labels));
        assert!(visibility.is_visible(RenderLayer::UI));

        // Should not have optional diagnostic layers
        assert!(!visibility.is_visible(RenderLayer::Heatmap));
        assert!(!visibility.is_visible(RenderLayer::Trails));
        assert!(!visibility.is_visible(RenderLayer::Connections));
        assert!(!visibility.is_visible(RenderLayer::Zones));
    }

    #[test]
    fn test_standard_mode_layers() {
        let visibility = DisplayMode::Standard.layer_visibility();

        // Should have core layers
        assert!(visibility.is_visible(RenderLayer::Agents));
        assert!(visibility.is_visible(RenderLayer::Trails));
        assert!(visibility.is_visible(RenderLayer::Connections));

        // Should not have debug-only layers
        assert!(!visibility.is_visible(RenderLayer::Heatmap));
        assert!(!visibility.is_visible(RenderLayer::Grid));
        assert!(!visibility.is_visible(RenderLayer::Zones));
    }

    #[test]
    fn test_debug_mode_layers() {
        let visibility = DisplayMode::Debug.layer_visibility();

        // Should have everything visible
        assert!(visibility.is_visible(RenderLayer::Agents));
        assert!(visibility.is_visible(RenderLayer::Trails));
        assert!(visibility.is_visible(RenderLayer::Connections));
        assert!(visibility.is_visible(RenderLayer::Heatmap));
        assert!(visibility.is_visible(RenderLayer::Zones));
        assert!(visibility.is_visible(RenderLayer::Grid));
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(DisplayMode::Minimal.name(), "Minimal");
        assert_eq!(DisplayMode::Standard.name(), "Standard");
        assert_eq!(DisplayMode::Debug.name(), "Debug");
    }
}
