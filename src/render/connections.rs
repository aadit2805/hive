use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::positioning::Position;
use crate::state::field::ActiveConnection;

use super::colors::dim_color;

/// Widget for rendering connections between agents
pub struct ConnectionsWidget<'a> {
    connections: &'a [ActiveConnection],
    /// Function to get agent positions
    get_position: Box<dyn Fn(&str) -> Option<Position> + 'a>,
}

impl<'a> ConnectionsWidget<'a> {
    pub fn new<F>(connections: &'a [ActiveConnection], get_position: F) -> Self
    where
        F: Fn(&str) -> Option<Position> + 'a,
    {
        Self {
            connections,
            get_position: Box::new(get_position),
        }
    }
}

impl Widget for ConnectionsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);

        for conn in self.connections {
            let Some(from_pos) = (self.get_position)(&conn.from) else {
                continue;
            };
            let Some(to_pos) = (self.get_position)(&conn.to) else {
                continue;
            };

            let (x1, y1) = from_pos.to_terminal(inner_width, inner_height);
            let (x2, y2) = to_pos.to_terminal(inner_width, inner_height);

            // Draw line between positions
            draw_line(
                buf,
                area.x + 1 + x1,
                area.y + 1 + y1,
                area.x + 1 + x2,
                area.y + 1 + y2,
                area,
                conn.opacity,
            );

            // Draw label at midpoint if opacity is high enough
            if conn.opacity > 0.5 && !conn.label.is_empty() {
                let mid_x = (x1 + x2) / 2 + area.x + 1;
                let mid_y = (y1 + y2) / 2 + area.y + 1;

                let label_style = Style::default().fg(dim_color(
                    Color::Rgb(200, 200, 200),
                    conn.opacity * 0.7,
                ));

                let label = truncate_label(&conn.label, 15);
                let label_start = mid_x.saturating_sub(label.len() as u16 / 2);

                for (i, ch) in label.chars().enumerate() {
                    let x = label_start + i as u16;
                    if x > area.x && x < area.x + area.width - 1 && mid_y > area.y && mid_y < area.y + area.height - 1
                    {
                        let cell = &mut buf[(x, mid_y)];
                        if is_line_char(cell.symbol()) || cell.symbol() == " " {
                            cell.set_char(ch).set_style(label_style);
                        }
                    }
                }
            }
        }
    }
}

/// Draw a line between two points using Bresenham's algorithm
fn draw_line(
    buf: &mut Buffer,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
    bounds: Rect,
    opacity: f32,
) {
    let color = dim_color(Color::Rgb(100, 150, 200), opacity);
    let style = Style::default().fg(color);

    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = (y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1i32 } else { -1 };
    let sy = if y1 < y2 { 1i32 } else { -1 };
    let mut err = dx - dy;

    let mut x = x1 as i32;
    let mut y = y1 as i32;

    let min_x = bounds.x as i32 + 1;
    let max_x = bounds.x as i32 + bounds.width as i32 - 2;
    let min_y = bounds.y as i32 + 1;
    let max_y = bounds.y as i32 + bounds.height as i32 - 2;

    loop {
        if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
            let cell = &mut buf[(x as u16, y as u16)];

            // Choose line character based on direction
            let ch = if dx > dy * 2 {
                '─'
            } else if dy > dx * 2 {
                '│'
            } else if (sx > 0) == (sy > 0) {
                '╲'
            } else {
                '╱'
            };

            // Only draw on empty cells or existing line chars
            if cell.symbol() == " " || is_line_char(cell.symbol()) {
                cell.set_char(ch).set_style(style);
            }
        }

        if x == x2 as i32 && y == y2 as i32 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn is_line_char(s: &str) -> bool {
    matches!(s, "─" | "│" | "╱" | "╲" | "·" | "•" | "∙")
}

fn truncate_label(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

/// Render all connections
pub fn render_connections<F>(
    connections: &[ActiveConnection],
    get_position: F,
    area: Rect,
    buf: &mut Buffer,
)
where
    F: Fn(&str) -> Option<Position>,
{
    ConnectionsWidget::new(connections, get_position).render(area, buf);
}
