use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::state::field::StoredLandmark;
use std::collections::HashMap;

use crate::event::LandmarkId;

/// The main field widget that renders the background and landmarks
pub struct FieldWidget<'a> {
    landmarks: &'a HashMap<LandmarkId, StoredLandmark>,
    show_landmarks: bool,
}

impl<'a> FieldWidget<'a> {
    pub fn new(landmarks: &'a HashMap<LandmarkId, StoredLandmark>) -> Self {
        Self {
            landmarks,
            show_landmarks: true,
        }
    }

    pub fn show_landmarks(mut self, show: bool) -> Self {
        self.show_landmarks = show;
        self
    }
}

impl Widget for FieldWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw field border
        let border_style = Style::default().fg(Color::Rgb(40, 40, 50));

        // Top and bottom borders
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_char('─').set_style(border_style);
            buf[(x, area.y + area.height - 1)]
                .set_char('─')
                .set_style(border_style);
        }

        // Left and right borders
        for y in area.y..area.y + area.height {
            buf[(area.x, y)].set_char('│').set_style(border_style);
            buf[(area.x + area.width - 1, y)]
                .set_char('│')
                .set_style(border_style);
        }

        // Corners
        buf[(area.x, area.y)].set_char('┌').set_style(border_style);
        buf[(area.x + area.width - 1, area.y)]
            .set_char('┐')
            .set_style(border_style);
        buf[(area.x, area.y + area.height - 1)]
            .set_char('└')
            .set_style(border_style);
        buf[(area.x + area.width - 1, area.y + area.height - 1)]
            .set_char('┘')
            .set_style(border_style);

        // Render landmarks as faint labels
        if self.show_landmarks {
            let landmark_style = Style::default().fg(Color::Rgb(50, 50, 60));
            let inner_width = area.width.saturating_sub(2);
            let inner_height = area.height.saturating_sub(2);

            for landmark in self.landmarks.values() {
                let (x, y) = landmark
                    .position
                    .to_terminal(inner_width, inner_height);

                let draw_x = area.x + 1 + x;
                let draw_y = area.y + 1 + y;

                // Draw landmark label
                let label = &landmark.label;
                let label_start = draw_x.saturating_sub(label.len() as u16 / 2);

                for (i, ch) in label.chars().enumerate() {
                    let cx = label_start + i as u16;
                    if cx > area.x && cx < area.x + area.width - 1 && draw_y > area.y && draw_y < area.y + area.height - 1
                    {
                        buf[(cx, draw_y)].set_char(ch).set_style(landmark_style);
                    }
                }
            }
        }
    }
}

/// Render the field background
pub fn render_field(
    area: Rect,
    buf: &mut Buffer,
    landmarks: &HashMap<LandmarkId, StoredLandmark>,
) {
    FieldWidget::new(landmarks).render(area, buf);
}
