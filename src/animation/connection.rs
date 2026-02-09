use std::time::{Duration, Instant};

/// Animation state for a connection between agents
#[derive(Debug, Clone)]
pub struct ConnectionAnimation {
    created_at: Instant,
    state: ConnectionState,
    opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionState {
    FadingIn,
    Visible,
    FadingOut,
}

/// Duration for fade in animation
const FADE_IN_DURATION: Duration = Duration::from_millis(300);

/// Duration to hold visible
const VISIBLE_DURATION: Duration = Duration::from_secs(3);

/// Duration for fade out animation
const FADE_OUT_DURATION: Duration = Duration::from_millis(500);

impl ConnectionAnimation {
    pub fn new() -> Self {
        Self {
            created_at: Instant::now(),
            state: ConnectionState::FadingIn,
            opacity: 0.0,
        }
    }

    /// Update animation state, returns true if animation is complete
    pub fn update(&mut self, dt: f32) -> bool {
        let age = self.created_at.elapsed();

        match self.state {
            ConnectionState::FadingIn => {
                let progress = age.as_secs_f32() / FADE_IN_DURATION.as_secs_f32();
                self.opacity = ease_out_quad(progress.min(1.0));

                if age >= FADE_IN_DURATION {
                    self.state = ConnectionState::Visible;
                }
            }
            ConnectionState::Visible => {
                self.opacity = 1.0;

                if age >= FADE_IN_DURATION + VISIBLE_DURATION {
                    self.state = ConnectionState::FadingOut;
                }
            }
            ConnectionState::FadingOut => {
                let fade_start = FADE_IN_DURATION + VISIBLE_DURATION;
                let fade_progress = (age - fade_start).as_secs_f32() / FADE_OUT_DURATION.as_secs_f32();
                self.opacity = 1.0 - ease_in_quad(fade_progress.min(1.0));

                if age >= fade_start + FADE_OUT_DURATION {
                    return true; // Animation complete
                }
            }
        }

        false
    }

    /// Get current opacity
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Force fade out
    pub fn start_fade_out(&mut self) {
        if self.state != ConnectionState::FadingOut {
            self.state = ConnectionState::FadingOut;
            // Adjust created_at so fade out starts from current opacity
            let elapsed_for_opacity = Duration::from_secs_f32(
                FADE_IN_DURATION.as_secs_f32()
                    + VISIBLE_DURATION.as_secs_f32()
                    + (1.0 - self.opacity) * FADE_OUT_DURATION.as_secs_f32(),
            );
            self.created_at = Instant::now() - elapsed_for_opacity;
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.state == ConnectionState::FadingOut && self.opacity <= 0.0
    }
}

impl Default for ConnectionAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Ease out quadratic
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Ease in quadratic
fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Data transfer animation (dots moving along connection)
pub struct DataTransferAnimation {
    progress: f32,
    speed: f32,
    active: bool,
}

impl DataTransferAnimation {
    pub fn new(speed: f32) -> Self {
        Self {
            progress: 0.0,
            speed,
            active: true,
        }
    }

    /// Update animation, returns true if complete
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.active {
            return true;
        }

        self.progress += dt * self.speed;

        if self.progress >= 1.0 {
            self.active = false;
            return true;
        }

        false
    }

    /// Get current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Get position along line for rendering dot
    pub fn dot_positions(&self, num_dots: usize) -> Vec<f32> {
        let spacing = 0.15;
        (0..num_dots)
            .map(|i| (self.progress - i as f32 * spacing).rem_euclid(1.0))
            .filter(|&p| p <= self.progress && p >= 0.0)
            .collect()
    }
}
