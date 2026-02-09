use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};
use std::time::{Duration, Instant};

use crate::state::Agent;

use super::colors::{dim_color, get_agent_color};

/// Trail symbols from newest to oldest
const TRAIL_SYMBOLS: [&str; 5] = ["•", "∙", "·", "˙", " "];

/// Maximum age for trail points before they're invisible
const MAX_TRAIL_AGE: Duration = Duration::from_secs(5);

/// Widget for rendering agent trails
pub struct TrailsWidget<'a> {
    agents: Vec<&'a Agent>,
}

impl<'a> TrailsWidget<'a> {
    pub fn new(agents: Vec<&'a Agent>) -> Self {
        Self { agents }
    }
}

impl Widget for TrailsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);
        let now = Instant::now();

        for agent in &self.agents {
            let base_color = get_agent_color(agent.color_index);

            for point in &agent.trail {
                let age = now.duration_since(point.timestamp);
                if age > MAX_TRAIL_AGE {
                    continue;
                }

                let age_factor = 1.0 - (age.as_secs_f32() / MAX_TRAIL_AGE.as_secs_f32());
                let symbol_index = ((1.0 - age_factor) * (TRAIL_SYMBOLS.len() - 1) as f32) as usize;
                let symbol = TRAIL_SYMBOLS[symbol_index.min(TRAIL_SYMBOLS.len() - 1)];

                if symbol == " " {
                    continue;
                }

                let (x, y) = point.position.to_terminal(inner_width, inner_height);
                let draw_x = area.x + 1 + x;
                let draw_y = area.y + 1 + y;

                if draw_x <= area.x || draw_x >= area.x + area.width - 1 {
                    continue;
                }
                if draw_y <= area.y || draw_y >= area.y + area.height - 1 {
                    continue;
                }

                // Dim color based on age
                let color = dim_color(base_color, age_factor * 0.5);
                let style = Style::default().fg(color);

                let cell = &mut buf[(draw_x, draw_y)];
                // Only draw if cell is empty (don't overwrite agents)
                if cell.symbol() == " " || cell.symbol().starts_with(['·', '˙', '∙', '•']) {
                    cell.set_symbol(symbol).set_style(style);
                }
            }
        }
    }
}

/// Render all agent trails
pub fn render_trails(agents: Vec<&Agent>, area: Rect, buf: &mut Buffer) {
    TrailsWidget::new(agents).render(area, buf);
}
