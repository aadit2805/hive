//! Agent detail panel widget for hover display.
//!
//! Renders a small panel showing agent details when hovering over an agent.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::state::Agent;
use super::colors::get_agent_color;

/// Panel dimensions
const PANEL_WIDTH: u16 = 24;
const PANEL_HEIGHT: u16 = 8;

/// Widget for displaying agent details on hover.
///
/// Renders a compact panel showing:
/// - Agent name and status
/// - Current focus keywords
/// - Intensity bar
/// - Recent message (truncated)
pub struct AgentPanel<'a> {
    agent: &'a Agent,
}

impl<'a> AgentPanel<'a> {
    /// Create a new agent panel widget.
    pub fn new(agent: &'a Agent) -> Self {
        Self { agent }
    }

    /// Get the preferred panel dimensions.
    pub fn dimensions() -> (u16, u16) {
        (PANEL_WIDTH, PANEL_HEIGHT)
    }

    /// Calculate the best position for the panel given agent position and screen bounds.
    ///
    /// Tries to place the panel near the agent without going off-screen.
    pub fn calculate_position(
        agent_x: u16,
        agent_y: u16,
        area: Rect,
    ) -> (u16, u16) {
        // Try to place panel to the right of the agent
        let mut panel_x = agent_x.saturating_add(2);
        let mut panel_y = agent_y.saturating_sub(PANEL_HEIGHT / 2);

        // If panel would go off right edge, place it to the left
        if panel_x + PANEL_WIDTH > area.x + area.width {
            panel_x = agent_x.saturating_sub(PANEL_WIDTH + 2);
        }

        // If panel would go off left edge, clamp to left edge
        if panel_x < area.x {
            panel_x = area.x + 1;
        }

        // If panel would go off top, clamp to top
        if panel_y < area.y {
            panel_y = area.y + 1;
        }

        // If panel would go off bottom, clamp to bottom
        if panel_y + PANEL_HEIGHT > area.y + area.height {
            panel_y = (area.y + area.height).saturating_sub(PANEL_HEIGHT + 1);
        }

        (panel_x, panel_y)
    }
}

impl Widget for AgentPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Use the minimum of requested area and panel size
        let width = area.width.min(PANEL_WIDTH);
        let height = area.height.min(PANEL_HEIGHT);

        if width < 10 || height < 4 {
            return; // Too small to render
        }

        let agent_color = get_agent_color(self.agent.color_index);

        // Background
        let bg_style = Style::default().bg(Color::Rgb(25, 25, 35));
        for y in area.y..area.y + height {
            for x in area.x..area.x + width {
                if x < buf.area.width && y < buf.area.height {
                    buf[(x, y)].set_char(' ').set_style(bg_style);
                }
            }
        }

        // Border
        let border_style = Style::default().fg(agent_color);

        // Top and bottom borders
        for x in area.x..area.x + width {
            if x < buf.area.width {
                if area.y < buf.area.height {
                    buf[(x, area.y)].set_char('─').set_style(border_style);
                }
                if area.y + height - 1 < buf.area.height {
                    buf[(x, area.y + height - 1)].set_char('─').set_style(border_style);
                }
            }
        }

        // Left and right borders
        for y in area.y..area.y + height {
            if y < buf.area.height {
                if area.x < buf.area.width {
                    buf[(area.x, y)].set_char('│').set_style(border_style);
                }
                if area.x + width - 1 < buf.area.width {
                    buf[(area.x + width - 1, y)].set_char('│').set_style(border_style);
                }
            }
        }

        // Corners
        if area.x < buf.area.width && area.y < buf.area.height {
            buf[(area.x, area.y)].set_char('╭').set_style(border_style);
        }
        if area.x + width - 1 < buf.area.width && area.y < buf.area.height {
            buf[(area.x + width - 1, area.y)].set_char('╮').set_style(border_style);
        }
        if area.x < buf.area.width && area.y + height - 1 < buf.area.height {
            buf[(area.x, area.y + height - 1)].set_char('╰').set_style(border_style);
        }
        if area.x + width - 1 < buf.area.width && area.y + height - 1 < buf.area.height {
            buf[(area.x + width - 1, area.y + height - 1)].set_char('╯').set_style(border_style);
        }

        // Content area
        let content_width = (width.saturating_sub(4)) as usize;
        let content_x = area.x + 2;
        let mut y = area.y + 1;

        // Agent name (bold, colored)
        let name_style = Style::default()
            .fg(agent_color)
            .add_modifier(Modifier::BOLD);
        let name = truncate(&self.agent.id, content_width);
        render_text(buf, content_x, y, &name, name_style);
        y += 1;

        // Status
        let status_str = format!("{:?}", self.agent.status);
        let status_color = match self.agent.status {
            crate::event::AgentStatus::Active => Color::Rgb(100, 200, 150),
            crate::event::AgentStatus::Thinking => Color::Rgb(150, 150, 255),
            crate::event::AgentStatus::Waiting => Color::Rgb(200, 200, 100),
            crate::event::AgentStatus::Idle => Color::Rgb(100, 100, 120),
            crate::event::AgentStatus::Error => Color::Rgb(255, 100, 100),
        };
        let status_style = Style::default().fg(status_color);
        render_text(buf, content_x, y, &status_str, status_style);
        y += 1;

        // Intensity bar
        if y < area.y + height - 1 {
            let bar_width = content_width.min(12);
            let intensity_bar = create_intensity_bar(self.agent.intensity, bar_width);
            let bar_style = Style::default().fg(Color::Rgb(180, 180, 200));
            render_text(buf, content_x, y, &intensity_bar, bar_style);
            y += 1;
        }

        // Focus keywords (if any)
        if y < area.y + height - 1 && !self.agent.focus.is_empty() {
            let focus_str = self.agent.focus.join(", ");
            let focus_truncated = truncate(&focus_str, content_width);
            let focus_style = Style::default().fg(Color::Rgb(150, 200, 255));
            render_text(buf, content_x, y, &focus_truncated, focus_style);
            y += 1;
        }

        // Recent message (if any and space allows)
        if y < area.y + height - 1 && !self.agent.message.is_empty() {
            let msg_truncated = truncate(&self.agent.message, content_width);
            let msg_style = Style::default().fg(Color::Rgb(120, 120, 140));
            render_text(buf, content_x, y, &msg_truncated, msg_style);
        }
    }
}

/// Render text at a specific position
fn render_text(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx < buf.area.width && y < buf.area.height {
            buf[(cx, y)].set_char(ch).set_style(style);
        }
    }
}

/// Create an intensity bar visualization
fn create_intensity_bar(intensity: f32, width: usize) -> String {
    let bar_width = width.saturating_sub(2); // Account for brackets
    let filled = (intensity * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

/// Truncate a string to fit within a maximum width
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else if max_len > 1 {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    } else {
        "…".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hi", 2), "hi");
    }

    #[test]
    fn test_intensity_bar() {
        assert_eq!(create_intensity_bar(0.0, 12), "[░░░░░░░░░░]");
        assert_eq!(create_intensity_bar(1.0, 12), "[██████████]");
        assert_eq!(create_intensity_bar(0.5, 12), "[█████░░░░░]");
    }

    #[test]
    fn test_panel_dimensions() {
        let (w, h) = AgentPanel::dimensions();
        assert_eq!(w, PANEL_WIDTH);
        assert_eq!(h, PANEL_HEIGHT);
    }
}
