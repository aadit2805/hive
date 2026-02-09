use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::positioning::Position;

/// Heat map grid resolution (cells per terminal character)
const CELL_SIZE: u16 = 2;

/// Default heat decay rate per frame (0.98 for faster decay, was 0.995)
const DEFAULT_DECAY_RATE: f32 = 0.98;

/// Heat accumulation rate when agent is present
const ACCUMULATION_RATE: f32 = 0.05;

/// Default minimum heat threshold before clearing
const DEFAULT_HEAT_THRESHOLD: f32 = 0.02;

/// Configuration for heatmap behavior
#[derive(Debug, Clone)]
pub struct HeatmapConfig {
    /// Heat decay rate per frame (default: 0.98, lower = faster decay)
    pub decay_rate: f32,
    /// Minimum heat threshold before clearing (default: 0.02)
    pub heat_threshold: f32,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            decay_rate: DEFAULT_DECAY_RATE,
            heat_threshold: DEFAULT_HEAT_THRESHOLD,
        }
    }
}

impl HeatmapConfig {
    /// Create a new config with custom decay rate
    pub fn with_decay_rate(mut self, decay_rate: f32) -> Self {
        self.decay_rate = decay_rate.clamp(0.9, 0.999);
        self
    }

    /// Create a new config with custom heat threshold
    pub fn with_heat_threshold(mut self, threshold: f32) -> Self {
        self.heat_threshold = threshold.clamp(0.001, 0.1);
        self
    }
}

/// Heat map for visualizing agent activity over time
pub struct HeatMap {
    grid: Vec<Vec<f32>>,
    width: usize,
    height: usize,
    config: HeatmapConfig,
}

impl HeatMap {
    /// Create a new heatmap with default configuration
    pub fn new(width: u16, height: u16) -> Self {
        Self::with_config(width, height, HeatmapConfig::default())
    }

    /// Create a new heatmap with custom configuration
    pub fn with_config(width: u16, height: u16, config: HeatmapConfig) -> Self {
        let grid_width = (width / CELL_SIZE).max(1) as usize;
        let grid_height = (height / CELL_SIZE).max(1) as usize;

        Self {
            grid: vec![vec![0.0; grid_width]; grid_height],
            width: grid_width,
            height: grid_height,
            config,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &HeatmapConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: HeatmapConfig) {
        self.config = config;
    }

    /// Set the decay rate
    pub fn set_decay_rate(&mut self, decay_rate: f32) {
        self.config.decay_rate = decay_rate.clamp(0.9, 0.999);
    }

    /// Resize the heat map grid (preserves config)
    pub fn resize(&mut self, width: u16, height: u16) {
        let new_width = (width / CELL_SIZE).max(1) as usize;
        let new_height = (height / CELL_SIZE).max(1) as usize;

        if new_width != self.width || new_height != self.height {
            self.grid = vec![vec![0.0; new_width]; new_height];
            self.width = new_width;
            self.height = new_height;
        }
    }

    /// Add heat at a position with given intensity
    pub fn add_heat(&mut self, position: &Position, intensity: f32) {
        let x = (position.x * (self.width - 1) as f32) as usize;
        let y = (position.y * (self.height - 1) as f32) as usize;

        if x < self.width && y < self.height {
            self.grid[y][x] = (self.grid[y][x] + intensity * ACCUMULATION_RATE).min(1.0);

            // Add some spread to adjacent cells
            let spread = intensity * ACCUMULATION_RATE * 0.3;
            if x > 0 {
                self.grid[y][x - 1] = (self.grid[y][x - 1] + spread).min(1.0);
            }
            if x < self.width - 1 {
                self.grid[y][x + 1] = (self.grid[y][x + 1] + spread).min(1.0);
            }
            if y > 0 {
                self.grid[y - 1][x] = (self.grid[y - 1][x] + spread).min(1.0);
            }
            if y < self.height - 1 {
                self.grid[y + 1][x] = (self.grid[y + 1][x] + spread).min(1.0);
            }
        }
    }

    /// Decay all heat values using configured decay rate
    pub fn decay(&mut self) {
        let decay_rate = self.config.decay_rate;
        let threshold = self.config.heat_threshold;
        for row in &mut self.grid {
            for cell in row {
                *cell *= decay_rate;
                if *cell < threshold {
                    *cell = 0.0;
                }
            }
        }
    }

    /// Get heat value at a normalized position
    pub fn get_heat(&self, position: &Position) -> f32 {
        let x = (position.x * (self.width - 1) as f32) as usize;
        let y = (position.y * (self.height - 1) as f32) as usize;

        if x < self.width && y < self.height {
            self.grid[y][x]
        } else {
            0.0
        }
    }

    /// Clear all heat
    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                *cell = 0.0;
            }
        }
    }
}

/// Widget for rendering the heat map
pub struct HeatMapWidget<'a> {
    heatmap: &'a HeatMap,
}

impl<'a> HeatMapWidget<'a> {
    pub fn new(heatmap: &'a HeatMap) -> Self {
        Self { heatmap }
    }
}

impl Widget for HeatMapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_x = area.x + 1;
        let inner_y = area.y + 1;
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);

        for screen_y in 0..inner_height {
            for screen_x in 0..inner_width {
                let norm_x = screen_x as f32 / inner_width as f32;
                let norm_y = screen_y as f32 / inner_height as f32;

                let pos = Position::new(norm_x, norm_y);
                let heat = self.heatmap.get_heat(&pos);

                if heat > 0.05 {
                    let color = heat_to_color(heat);
                    let style = Style::default().bg(color);

                    let x = inner_x + screen_x;
                    let y = inner_y + screen_y;

                    // Only modify background if cell is otherwise empty
                    let cell = &mut buf[(x, y)];
                    if cell.symbol() == " " {
                        cell.set_style(style);
                    }
                }
            }
        }
    }
}

/// Convert heat value (0.0-1.0) to a color
fn heat_to_color(heat: f32) -> Color {
    let heat = heat.clamp(0.0, 1.0);

    // Color gradient: dark blue -> blue -> cyan -> yellow -> orange -> red
    if heat < 0.2 {
        let t = heat / 0.2;
        Color::Rgb(
            0,
            0,
            (20.0 + t * 40.0) as u8,
        )
    } else if heat < 0.4 {
        let t = (heat - 0.2) / 0.2;
        Color::Rgb(
            0,
            (t * 60.0) as u8,
            (60.0 + t * 40.0) as u8,
        )
    } else if heat < 0.6 {
        let t = (heat - 0.4) / 0.2;
        Color::Rgb(
            (t * 100.0) as u8,
            (60.0 + t * 40.0) as u8,
            (100.0 - t * 50.0) as u8,
        )
    } else if heat < 0.8 {
        let t = (heat - 0.6) / 0.2;
        Color::Rgb(
            (100.0 + t * 100.0) as u8,
            (100.0 - t * 30.0) as u8,
            (50.0 - t * 50.0) as u8,
        )
    } else {
        let t = (heat - 0.8) / 0.2;
        Color::Rgb(
            (200.0 + t * 55.0) as u8,
            (70.0 - t * 70.0) as u8,
            0,
        )
    }
}

/// Render the heat map as background
pub fn render_heatmap(heatmap: &HeatMap, area: Rect, buf: &mut Buffer) {
    HeatMapWidget::new(heatmap).render(area, buf);
}
