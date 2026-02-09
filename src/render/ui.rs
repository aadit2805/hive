use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::state::{Agent, History};
use super::DisplayMode;

/// Status bar at the bottom of the screen
pub struct StatusBar<'a> {
    agents: &'a [&'a Agent],
    paused: bool,
    playback_speed: f32,
    replay_mode: bool,
    replay_position: f32,
    fps: u32,
    display_mode: DisplayMode,
    /// Optional filter text to display when filtering is active
    filter_text: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(agents: &'a [&'a Agent]) -> Self {
        Self {
            agents,
            paused: false,
            playback_speed: 1.0,
            replay_mode: false,
            replay_position: 0.0,
            fps: 30,
            display_mode: DisplayMode::default(),
            filter_text: None,
        }
    }

    /// Set the filter text to display when filtering is active.
    pub fn filter_text(mut self, filter: Option<&'a str>) -> Self {
        self.filter_text = filter;
        self
    }

    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    pub fn playback_speed(mut self, speed: f32) -> Self {
        self.playback_speed = speed;
        self
    }

    pub fn replay_mode(mut self, mode: bool, position: f32) -> Self {
        self.replay_mode = mode;
        self.replay_position = position;
        self
    }

    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    pub fn display_mode(mut self, mode: DisplayMode) -> Self {
        self.display_mode = mode;
        self
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Background
        let bg_style = Style::default().bg(Color::Rgb(25, 25, 35));
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(bg_style);
        }

        let mut x = area.x + 1;
        let label_style = Style::default().fg(Color::Rgb(100, 100, 120));
        let value_style = Style::default().fg(Color::Rgb(180, 180, 200));
        let accent_style = Style::default()
            .fg(Color::Rgb(100, 200, 150))
            .add_modifier(Modifier::BOLD);

        // HIVE logo
        let logo = "◈ HIVE";
        for ch in logo.chars() {
            buf[(x, area.y)].set_char(ch).set_style(accent_style);
            x += 1;
        }
        x += 2;

        // Agent count
        let active_count = self.agents.iter().filter(|a| a.intensity > 0.1).count();
        let count_text = format!("Agents: {}/{}", active_count, self.agents.len());
        for ch in count_text.chars() {
            if x >= area.x + area.width - 1 {
                break;
            }
            buf[(x, area.y)].set_char(ch).set_style(value_style);
            x += 1;
        }
        x += 2;

        // Speed indicator
        let speed_text = format!("Speed: {:.1}x", self.playback_speed);
        for ch in speed_text.chars() {
            if x >= area.x + area.width - 1 {
                break;
            }
            buf[(x, area.y)].set_char(ch).set_style(label_style);
            x += 1;
        }
        x += 2;

        // Pause indicator
        if self.paused {
            let pause_style = Style::default()
                .fg(Color::Rgb(255, 200, 100))
                .add_modifier(Modifier::BOLD);
            let pause_text = "⏸ PAUSED";
            for ch in pause_text.chars() {
                if x >= area.x + area.width - 1 {
                    break;
                }
                buf[(x, area.y)].set_char(ch).set_style(pause_style);
                x += 1;
            }
            x += 2;
        }

        // Replay mode indicator
        if self.replay_mode {
            let replay_style = Style::default().fg(Color::Rgb(150, 150, 255));
            let pos_pct = (self.replay_position * 100.0) as u8;
            let replay_text = format!("⏪ REPLAY {}%", pos_pct);
            for ch in replay_text.chars() {
                if x >= area.x + area.width - 1 {
                    break;
                }
                buf[(x, area.y)].set_char(ch).set_style(replay_style);
                x += 1;
            }
            x += 2;
        }

        // Display mode indicator
        let mode_style = match self.display_mode {
            DisplayMode::Minimal => Style::default().fg(Color::Rgb(150, 200, 255)),
            DisplayMode::Standard => Style::default().fg(Color::Rgb(100, 200, 150)),
            DisplayMode::Debug => Style::default().fg(Color::Rgb(255, 200, 100)),
        };
        let mode_text = format!("[{}]", self.display_mode.name());
        for ch in mode_text.chars() {
            if x >= area.x + area.width - 1 {
                break;
            }
            buf[(x, area.y)].set_char(ch).set_style(mode_style);
            x += 1;
        }
        x += 2;

        // Filter indicator (amber when active)
        if let Some(filter) = self.filter_text {
            let filter_style = Style::default().fg(Color::Rgb(255, 200, 80)); // Amber
            let filter_text = format!("[FILTER: {}]", filter);
            for ch in filter_text.chars() {
                if x >= area.x + area.width - 1 {
                    break;
                }
                buf[(x, area.y)].set_char(ch).set_style(filter_style);
                x += 1;
            }
        }

        // Right-aligned help hint with mode key reminder
        let help_text = "m:mode ?:help";
        let help_x = area.x + area.width - help_text.len() as u16 - 1;
        let mut hx = help_x;
        for ch in help_text.chars() {
            if hx >= area.x + area.width - 1 {
                break;
            }
            buf[(hx, area.y)].set_char(ch).set_style(label_style);
            hx += 1;
        }
    }
}

/// Help overlay widget
pub struct HelpOverlay;

impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Semi-transparent background
        let bg_style = Style::default().bg(Color::Rgb(20, 20, 30));
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(bg_style);
            }
        }

        // Help box dimensions
        let box_width = 50u16;
        let box_height = 18u16;
        let box_x = area.x + (area.width.saturating_sub(box_width)) / 2;
        let box_y = area.y + (area.height.saturating_sub(box_height)) / 2;

        // Draw box background
        let box_bg = Style::default().bg(Color::Rgb(35, 35, 45));
        for y in box_y..box_y + box_height {
            for x in box_x..box_x + box_width {
                buf[(x, y)].set_char(' ').set_style(box_bg);
            }
        }

        // Draw border
        let border_style = Style::default().fg(Color::Rgb(100, 200, 150));
        for x in box_x..box_x + box_width {
            buf[(x, box_y)].set_char('─').set_style(border_style);
            buf[(x, box_y + box_height - 1)]
                .set_char('─')
                .set_style(border_style);
        }
        for y in box_y..box_y + box_height {
            buf[(box_x, y)].set_char('│').set_style(border_style);
            buf[(box_x + box_width - 1, y)]
                .set_char('│')
                .set_style(border_style);
        }
        buf[(box_x, box_y)].set_char('╭').set_style(border_style);
        buf[(box_x + box_width - 1, box_y)]
            .set_char('╮')
            .set_style(border_style);
        buf[(box_x, box_y + box_height - 1)]
            .set_char('╰')
            .set_style(border_style);
        buf[(box_x + box_width - 1, box_y + box_height - 1)]
            .set_char('╯')
            .set_style(border_style);

        // Title
        let title = " HIVE Controls ";
        let title_x = box_x + (box_width - title.len() as u16) / 2;
        let title_style = Style::default()
            .fg(Color::Rgb(100, 200, 150))
            .add_modifier(Modifier::BOLD);
        for (i, ch) in title.chars().enumerate() {
            buf[(title_x + i as u16, box_y)]
                .set_char(ch)
                .set_style(title_style);
        }

        // Help content
        let key_style = Style::default()
            .fg(Color::Rgb(200, 200, 100))
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::Rgb(180, 180, 190));

        let controls = [
            ("q, Esc", "Quit"),
            ("Space", "Pause/Resume"),
            ("+/-", "Speed up/down"),
            ("r", "Toggle replay mode"),
            ("←/→", "Seek backward/forward (replay)"),
            ("m", "Cycle display mode"),
            ("1/2/3", "Minimal/Standard/Debug mode"),
            ("h", "Toggle heat map"),
            ("t", "Toggle trails"),
            ("l", "Toggle landmarks"),
            ("c", "Clear heat map"),
            ("?", "Toggle this help"),
        ];

        let mut y = box_y + 2;
        for (key, desc) in controls {
            if y >= box_y + box_height - 1 {
                break;
            }

            let mut x = box_x + 3;

            // Key
            for ch in key.chars() {
                buf[(x, y)].set_char(ch).set_style(key_style);
                x += 1;
            }

            // Padding
            x = box_x + 15;

            // Description
            for ch in desc.chars() {
                if x >= box_x + box_width - 2 {
                    break;
                }
                buf[(x, y)].set_char(ch).set_style(desc_style);
                x += 1;
            }

            y += 1;
        }

        // Footer
        let footer = "Press any key to close";
        let footer_x = box_x + (box_width - footer.len() as u16) / 2;
        let footer_style = Style::default().fg(Color::Rgb(100, 100, 120));
        for (i, ch) in footer.chars().enumerate() {
            buf[(footer_x + i as u16, box_y + box_height - 2)]
                .set_char(ch)
                .set_style(footer_style);
        }
    }
}

/// Replay timeline slider
pub struct TimelineWidget<'a> {
    history: &'a History,
}

impl<'a> TimelineWidget<'a> {
    pub fn new(history: &'a History) -> Self {
        Self { history }
    }
}

impl Widget for TimelineWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 {
            return;
        }

        let bg_style = Style::default().bg(Color::Rgb(30, 30, 40));
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(bg_style);
        }

        // Track
        let track_style = Style::default().fg(Color::Rgb(60, 60, 70));
        let filled_style = Style::default().fg(Color::Rgb(100, 200, 150));

        let track_start = area.x + 2;
        let track_end = area.x + area.width - 3;
        let track_width = track_end - track_start;

        let position = self.history.position();
        let filled_width = (position * track_width as f32) as u16;

        for x in track_start..track_end {
            let ch = if x - track_start < filled_width {
                '━'
            } else {
                '─'
            };
            let style = if x - track_start < filled_width {
                filled_style
            } else {
                track_style
            };
            buf[(x, area.y)].set_char(ch).set_style(style);
        }

        // Playhead
        let playhead_x = track_start + filled_width;
        if playhead_x < track_end {
            buf[(playhead_x, area.y)]
                .set_char('●')
                .set_style(filled_style);
        }

        // Event count
        let count_text = format!(" {} events", self.history.len());
        let count_style = Style::default().fg(Color::Rgb(100, 100, 120));
        let mut x = track_end + 1;
        for ch in count_text.chars() {
            if x >= area.x + area.width {
                break;
            }
            buf[(x, area.y)].set_char(ch).set_style(count_style);
            x += 1;
        }
    }
}

/// Type of empty state to display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyStateType {
    /// No agents have connected yet
    NoAgents,
    /// All agents are filtered out
    AllFiltered,
}

impl EmptyStateType {
    /// Get the main message for this empty state type.
    pub fn message(&self) -> &'static str {
        match self {
            EmptyStateType::NoAgents => "No agents connected",
            EmptyStateType::AllFiltered => "No agents match filter",
        }
    }

    /// Get the hint text for this empty state type.
    pub fn hint(&self) -> &'static str {
        match self {
            EmptyStateType::NoAgents => "Waiting for agents to connect...",
            EmptyStateType::AllFiltered => "Press Esc to clear filter",
        }
    }
}

/// Widget for displaying an empty state message.
///
/// Used when there are no agents to display, either because none
/// have connected or because all are filtered out.
pub struct EmptyStateWidget {
    state_type: EmptyStateType,
}

impl EmptyStateWidget {
    /// Create a new empty state widget.
    pub fn new(state_type: EmptyStateType) -> Self {
        Self { state_type }
    }
}

impl Widget for EmptyStateWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        let message = self.state_type.message();
        let hint = self.state_type.hint();

        // Center the message vertically
        let center_y = area.y + area.height / 2;

        // Message style - slightly muted
        let message_style = Style::default().fg(Color::Rgb(150, 150, 160));
        let hint_style = Style::default().fg(Color::Rgb(100, 100, 110));

        // Render main message (centered)
        let msg_x = area.x + (area.width.saturating_sub(message.len() as u16)) / 2;
        for (i, ch) in message.chars().enumerate() {
            if msg_x + i as u16 >= area.x + area.width {
                break;
            }
            buf[(msg_x + i as u16, center_y)]
                .set_char(ch)
                .set_style(message_style);
        }

        // Render hint below message (centered)
        if center_y + 1 < area.y + area.height {
            let hint_x = area.x + (area.width.saturating_sub(hint.len() as u16)) / 2;
            for (i, ch) in hint.chars().enumerate() {
                if hint_x + i as u16 >= area.x + area.width {
                    break;
                }
                buf[(hint_x + i as u16, center_y + 1)]
                    .set_char(ch)
                    .set_style(hint_style);
            }
        }

        // Optional: Add a subtle icon above the message
        if center_y > area.y {
            let icon = match self.state_type {
                EmptyStateType::NoAgents => "...",
                EmptyStateType::AllFiltered => "( )",
            };
            let icon_x = area.x + (area.width.saturating_sub(icon.len() as u16)) / 2;
            for (i, ch) in icon.chars().enumerate() {
                if icon_x + i as u16 >= area.x + area.width {
                    break;
                }
                buf[(icon_x + i as u16, center_y - 1)]
                    .set_char(ch)
                    .set_style(hint_style);
            }
        }
    }
}

/// Render the UI elements
pub fn render_ui(
    area: Rect,
    buf: &mut Buffer,
    agents: &[&Agent],
    paused: bool,
    speed: f32,
    history: &History,
    show_help: bool,
    fps: u32,
) {
    // Status bar at bottom
    let status_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    StatusBar::new(agents)
        .paused(paused)
        .playback_speed(speed)
        .replay_mode(history.replay_mode, history.position())
        .fps(fps)
        .render(status_area, buf);

    // Timeline when in replay mode
    if history.replay_mode {
        let timeline_area = Rect::new(area.x, area.y + area.height - 2, area.width, 1);
        TimelineWidget::new(history).render(timeline_area, buf);
    }

    // Help overlay
    if show_help {
        HelpOverlay.render(area, buf);
    }
}
