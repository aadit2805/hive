use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::state::Agent;

use super::colors::{dim_color, get_agent_color};

/// Widget for rendering all agents
pub struct AgentsWidget<'a> {
    agents: Vec<&'a Agent>,
    selected_agent: Option<&'a str>,
    hovered_agent: Option<&'a str>,
}

impl<'a> AgentsWidget<'a> {
    pub fn new(agents: Vec<&'a Agent>) -> Self {
        Self {
            agents,
            selected_agent: None,
            hovered_agent: None,
        }
    }

    pub fn selected(mut self, agent_id: Option<&'a str>) -> Self {
        self.selected_agent = agent_id;
        self
    }

    pub fn hovered(mut self, agent_id: Option<&'a str>) -> Self {
        self.hovered_agent = agent_id;
        self
    }
}

impl Widget for AgentsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);

        for agent in &self.agents {
            render_single_agent(agent, area, inner_width, inner_height, buf, self.selected_agent, self.hovered_agent);
        }
    }
}

fn render_single_agent(
    agent: &Agent,
    area: Rect,
    inner_width: u16,
    inner_height: u16,
    buf: &mut Buffer,
    selected: Option<&str>,
    hovered: Option<&str>,
) {
    let (x, y) = agent.position.to_terminal(inner_width, inner_height);
    let draw_x = area.x + 1 + x;
    let draw_y = area.y + 1 + y;

    // Skip if outside bounds
    if draw_x <= area.x || draw_x >= area.x + area.width - 1 {
        return;
    }
    if draw_y <= area.y || draw_y >= area.y + area.height - 1 {
        return;
    }

    let base_color = get_agent_color(agent.color_index);
    let brightness = agent.pulse_brightness();
    let color = if brightness > 0.8 {
        base_color
    } else {
        dim_color(base_color, brightness)
    };

    let is_selected = selected.is_some_and(|id| id == agent.id);
    let is_hovered = hovered.is_some_and(|id| id == agent.id);

    let mut style = Style::default().fg(color);
    if is_selected {
        style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
    } else if is_hovered {
        // Highlight hovered agent with underline and bold
        style = style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    } else if agent.intensity > 0.7 {
        style = style.add_modifier(Modifier::BOLD);
    }

    // Draw the agent symbol
    let symbol = agent.symbol();
    buf[(draw_x, draw_y)].set_symbol(symbol).set_style(style);

    // Draw glow effect for high intensity agents
    if agent.intensity > 0.6 && !is_selected {
        let glow_color = dim_color(base_color, 0.3);
        let glow_style = Style::default().fg(glow_color);

        // Horizontal glow
        if draw_x > area.x + 1 {
            let cell = &mut buf[(draw_x - 1, draw_y)];
            if cell.symbol() == " " {
                cell.set_symbol("·").set_style(glow_style);
            }
        }
        if draw_x < area.x + area.width - 2 {
            let cell = &mut buf[(draw_x + 1, draw_y)];
            if cell.symbol() == " " {
                cell.set_symbol("·").set_style(glow_style);
            }
        }
    }

    // Draw agent label below (if space allows)
    let label = agent.short_name();
    let label_y = draw_y + 1;

    if label_y < area.y + area.height - 1 {
        let label_style = Style::default().fg(dim_color(base_color, 0.6));
        let label_start = draw_x.saturating_sub(label.len() as u16 / 2);

        for (i, ch) in label.chars().enumerate() {
            let cx = label_start + i as u16;
            if cx > area.x && cx < area.x + area.width - 1 {
                let cell = &mut buf[(cx, label_y)];
                // Only draw if cell is empty
                if cell.symbol() == " " {
                    cell.set_char(ch).set_style(label_style);
                }
            }
        }
    }
}

/// Render all agents
pub fn render_agents(agents: Vec<&Agent>, area: Rect, buf: &mut Buffer, selected: Option<&str>) {
    AgentsWidget::new(agents).selected(selected).render(area, buf);
}

/// Widget for the agent detail popup
pub struct AgentDetailWidget<'a> {
    agent: &'a Agent,
}

impl<'a> AgentDetailWidget<'a> {
    pub fn new(agent: &'a Agent) -> Self {
        Self { agent }
    }
}

impl Widget for AgentDetailWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Background
        let bg_style = Style::default().bg(Color::Rgb(30, 30, 40));
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(bg_style);
            }
        }

        // Border
        let border_style = Style::default().fg(get_agent_color(self.agent.color_index));

        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_char('─').set_style(border_style);
            buf[(x, area.y + area.height - 1)]
                .set_char('─')
                .set_style(border_style);
        }

        for y in area.y..area.y + area.height {
            buf[(area.x, y)].set_char('│').set_style(border_style);
            buf[(area.x + area.width - 1, y)]
                .set_char('│')
                .set_style(border_style);
        }

        buf[(area.x, area.y)].set_char('╭').set_style(border_style);
        buf[(area.x + area.width - 1, area.y)]
            .set_char('╮')
            .set_style(border_style);
        buf[(area.x, area.y + area.height - 1)]
            .set_char('╰')
            .set_style(border_style);
        buf[(area.x + area.width - 1, area.y + area.height - 1)]
            .set_char('╯')
            .set_style(border_style);

        // Content
        let content_width = area.width.saturating_sub(4) as usize;
        let title_style = Style::default()
            .fg(get_agent_color(self.agent.color_index))
            .add_modifier(Modifier::BOLD);
        let label_style = Style::default().fg(Color::Rgb(150, 150, 160));
        let value_style = Style::default().fg(Color::Rgb(200, 200, 210));

        let mut y = area.y + 1;

        // Agent name
        let name = &self.agent.id;
        render_text(buf, area.x + 2, y, name, title_style, content_width);
        y += 1;

        // Status
        let status = format!("{:?}", self.agent.status);
        render_text(buf, area.x + 2, y, "Status: ", label_style, content_width);
        render_text(
            buf,
            area.x + 2 + 8,
            y,
            &status,
            value_style,
            content_width.saturating_sub(8),
        );
        y += 1;

        // Intensity
        let intensity_bar = create_intensity_bar(self.agent.intensity, 10);
        render_text(buf, area.x + 2, y, "Power: ", label_style, content_width);
        render_text(
            buf,
            area.x + 2 + 7,
            y,
            &intensity_bar,
            value_style,
            content_width.saturating_sub(7),
        );
        y += 1;

        // Focus
        if !self.agent.focus.is_empty() {
            let focus_str = self.agent.focus.join(", ");
            render_text(buf, area.x + 2, y, "Focus: ", label_style, content_width);
            y += 1;
            render_text(buf, area.x + 2, y, &focus_str, value_style, content_width);
            y += 1;
        }

        // Message
        if !self.agent.message.is_empty() && y < area.y + area.height - 1 {
            render_text(buf, area.x + 2, y, "Msg: ", label_style, content_width);
            y += 1;
            let msg = truncate_str(&self.agent.message, content_width);
            render_text(buf, area.x + 2, y, &msg, value_style, content_width);
        }
    }
}

fn render_text(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style, max_width: usize) {
    for (i, ch) in text.chars().take(max_width).enumerate() {
        buf[(x + i as u16, y)].set_char(ch).set_style(style);
    }
}

fn create_intensity_bar(intensity: f32, width: usize) -> String {
    let filled = (intensity * width as f32) as usize;
    let empty = width - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
