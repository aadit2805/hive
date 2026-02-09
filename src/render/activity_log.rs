//! Activity log widget for displaying recent agent activity.
//!
//! The activity log shows a chronological list of recent agent events,
//! with the newest entries at the bottom. Entries fade based on age
//! to provide visual indication of recency.

use std::collections::VecDeque;
use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

/// A single entry in the activity log.
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    /// When this entry was created
    pub timestamp: Instant,
    /// The agent that generated this activity
    pub agent_id: String,
    /// The activity message
    pub message: String,
    /// Color associated with this agent
    pub color: Color,
}

impl ActivityEntry {
    /// Create a new activity entry.
    pub fn new(agent_id: String, message: String, color: Color) -> Self {
        Self {
            timestamp: Instant::now(),
            agent_id,
            message,
            color,
        }
    }

    /// Get the age of this entry in seconds.
    pub fn age_seconds(&self) -> f32 {
        self.timestamp.elapsed().as_secs_f32()
    }
}

/// Activity log that tracks recent agent events.
#[derive(Debug)]
pub struct ActivityLog {
    entries: VecDeque<ActivityEntry>,
    max_entries: usize,
}

impl ActivityLog {
    /// Create a new activity log with a maximum number of entries.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Add a new entry to the activity log.
    ///
    /// If the log is at capacity, the oldest entry will be removed.
    pub fn add(&mut self, agent_id: String, message: String, color: Color) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(ActivityEntry::new(agent_id, message, color));
    }

    /// Get an iterator over the entries (oldest first).
    pub fn entries(&self) -> impl Iterator<Item = &ActivityEntry> {
        self.entries.iter()
    }

    /// Get the number of entries in the log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries from the log.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ActivityLog {
    fn default() -> Self {
        Self::new(50) // Default to 50 entries
    }
}

/// Widget for rendering the activity log.
///
/// Displays recent activity entries with the newest at the bottom.
/// Entries fade based on age to indicate recency.
pub struct ActivityLogWidget<'a> {
    log: &'a ActivityLog,
    /// Maximum age in seconds before an entry is fully faded
    max_age: f32,
    /// Title to display above the log
    title: Option<&'a str>,
}

impl<'a> ActivityLogWidget<'a> {
    /// Create a new activity log widget.
    pub fn new(log: &'a ActivityLog) -> Self {
        Self {
            log,
            max_age: 30.0, // Entries fade over 30 seconds
            title: Some("Activity"),
        }
    }

    /// Set the maximum age for fading (in seconds).
    pub fn max_age(mut self, max_age: f32) -> Self {
        self.max_age = max_age;
        self
    }

    /// Set the title for the log widget.
    pub fn title(mut self, title: Option<&'a str>) -> Self {
        self.title = title;
        self
    }

    /// Calculate the opacity for an entry based on its age.
    fn opacity_for_age(&self, age_seconds: f32) -> f32 {
        // Start fading after 5 seconds, fully faded at max_age
        let fade_start = 5.0;
        if age_seconds < fade_start {
            1.0
        } else {
            let fade_progress = (age_seconds - fade_start) / (self.max_age - fade_start);
            (1.0 - fade_progress).clamp(0.3, 1.0) // Minimum 30% opacity
        }
    }

    /// Apply opacity to a color.
    fn apply_opacity(color: Color, opacity: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                (r as f32 * opacity) as u8,
                (g as f32 * opacity) as u8,
                (b as f32 * opacity) as u8,
            ),
            other => other,
        }
    }
}

impl Widget for ActivityLogWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut y = area.y;

        // Render title if present
        if let Some(title) = self.title {
            if y < area.y + area.height {
                let title_style = Style::default().fg(Color::Rgb(100, 200, 150));
                let title_text = format!(" {} ", title);
                for (i, ch) in title_text.chars().enumerate() {
                    if area.x + i as u16 >= area.x + area.width {
                        break;
                    }
                    buf[(area.x + i as u16, y)]
                        .set_char(ch)
                        .set_style(title_style);
                }
                y += 1;
            }
        }

        // Calculate how many entries we can show
        let available_height = (area.y + area.height).saturating_sub(y) as usize;
        if available_height == 0 {
            return;
        }

        // Get the last N entries that fit
        let entries: Vec<_> = self.log.entries().collect();
        let start_idx = entries.len().saturating_sub(available_height);
        let visible_entries = &entries[start_idx..];

        // Render entries (newest at bottom)
        for entry in visible_entries {
            if y >= area.y + area.height {
                break;
            }

            let age = entry.age_seconds();
            let opacity = self.opacity_for_age(age);

            // Format: "[agent_id] message"
            let agent_style = Style::default().fg(Self::apply_opacity(entry.color, opacity));
            let msg_style =
                Style::default().fg(Self::apply_opacity(Color::Rgb(180, 180, 190), opacity));

            let mut x = area.x;

            // Render agent ID in brackets
            buf[(x, y)].set_char('[').set_style(msg_style);
            x += 1;

            // Truncate agent ID if needed
            let max_id_len = 12;
            let agent_display: String = if entry.agent_id.len() > max_id_len {
                format!("{}...", &entry.agent_id[..max_id_len - 3])
            } else {
                entry.agent_id.clone()
            };

            for ch in agent_display.chars() {
                if x >= area.x + area.width - 1 {
                    break;
                }
                buf[(x, y)].set_char(ch).set_style(agent_style);
                x += 1;
            }

            buf[(x, y)].set_char(']').set_style(msg_style);
            x += 1;

            // Space before message
            if x < area.x + area.width {
                buf[(x, y)].set_char(' ').set_style(msg_style);
                x += 1;
            }

            // Render message (truncate if needed)
            let remaining_width = (area.x + area.width).saturating_sub(x) as usize;
            let message_display: String = if entry.message.len() > remaining_width {
                if remaining_width > 3 {
                    format!("{}...", &entry.message[..remaining_width - 3])
                } else {
                    entry.message.chars().take(remaining_width).collect()
                }
            } else {
                entry.message.clone()
            };

            for ch in message_display.chars() {
                if x >= area.x + area.width {
                    break;
                }
                buf[(x, y)].set_char(ch).set_style(msg_style);
                x += 1;
            }

            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_log_creation() {
        let log = ActivityLog::new(10);
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_activity_log_add() {
        let mut log = ActivityLog::new(10);
        log.add(
            "agent-1".to_string(),
            "Test message".to_string(),
            Color::Blue,
        );
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
    }

    #[test]
    fn test_activity_log_max_entries() {
        let mut log = ActivityLog::new(3);
        log.add("agent-1".to_string(), "Message 1".to_string(), Color::Blue);
        log.add("agent-2".to_string(), "Message 2".to_string(), Color::Green);
        log.add("agent-3".to_string(), "Message 3".to_string(), Color::Red);
        log.add(
            "agent-4".to_string(),
            "Message 4".to_string(),
            Color::Yellow,
        );

        assert_eq!(log.len(), 3);

        // First entry should be "agent-2" (oldest remaining)
        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].agent_id, "agent-2");
        assert_eq!(entries[2].agent_id, "agent-4");
    }

    #[test]
    fn test_activity_entry_age() {
        let entry = ActivityEntry::new(
            "test".to_string(),
            "message".to_string(),
            Color::Blue,
        );
        // Age should be very small (just created)
        assert!(entry.age_seconds() < 1.0);
    }

    #[test]
    fn test_activity_log_clear() {
        let mut log = ActivityLog::new(10);
        log.add("agent-1".to_string(), "Message 1".to_string(), Color::Blue);
        log.add("agent-2".to_string(), "Message 2".to_string(), Color::Green);
        assert_eq!(log.len(), 2);

        log.clear();
        assert!(log.is_empty());
    }
}
