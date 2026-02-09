pub mod pulse;
pub mod connection;

pub use pulse::PulseAnimation;
pub use connection::ConnectionAnimation;

use std::time::{Duration, Instant};

/// Target frame rate
pub const TARGET_FPS: u32 = 30;

/// Frame duration for target FPS
pub const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS as u64);

/// Animation loop state
pub struct AnimationLoop {
    last_frame: Instant,
    frame_count: u64,
    fps_sample_start: Instant,
    fps_sample_count: u32,
    current_fps: u32,
}

impl AnimationLoop {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            frame_count: 0,
            fps_sample_start: now,
            fps_sample_count: 0,
            current_fps: TARGET_FPS,
        }
    }

    /// Check if it's time for a new frame
    pub fn should_render(&self) -> bool {
        self.last_frame.elapsed() >= FRAME_DURATION
    }

    /// Get delta time since last frame
    pub fn delta_time(&self) -> f32 {
        self.last_frame.elapsed().as_secs_f32()
    }

    /// Mark frame as rendered
    pub fn frame_rendered(&mut self) {
        self.last_frame = Instant::now();
        self.frame_count += 1;
        self.fps_sample_count += 1;

        // Update FPS calculation every second
        if self.fps_sample_start.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.fps_sample_count;
            self.fps_sample_count = 0;
            self.fps_sample_start = Instant::now();
        }
    }

    /// Get current FPS
    pub fn fps(&self) -> u32 {
        self.current_fps
    }

    /// Get total frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Time until next frame
    pub fn time_until_next_frame(&self) -> Duration {
        let elapsed = self.last_frame.elapsed();
        if elapsed >= FRAME_DURATION {
            Duration::ZERO
        } else {
            FRAME_DURATION - elapsed
        }
    }
}

impl Default for AnimationLoop {
    fn default() -> Self {
        Self::new()
    }
}
