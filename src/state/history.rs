use std::time::{Duration, Instant};

use crate::event::{HiveEvent, TimestampedEvent};

/// History buffer for replay functionality
pub struct History {
    events: Vec<TimestampedEvent>,
    /// Current playback position (index into events)
    playback_index: usize,
    /// Whether we're in replay mode
    pub replay_mode: bool,
    /// Start time of the replay
    replay_start: Option<Instant>,
    /// Time offset into the recording
    replay_offset: Duration,
}

impl History {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            playback_index: 0,
            replay_mode: false,
            replay_start: None,
            replay_offset: Duration::ZERO,
        }
    }

    /// Record a new event
    pub fn record(&mut self, event: HiveEvent) {
        self.events.push(TimestampedEvent {
            event,
            received_at: Instant::now(),
        });
    }

    /// Load events from a list (for replay from file)
    pub fn load_events(&mut self, events: Vec<HiveEvent>) {
        let now = Instant::now();
        self.events.clear();

        for (i, event) in events.into_iter().enumerate() {
            self.events.push(TimestampedEvent {
                event,
                // Space events out based on their timestamps
                received_at: now + Duration::from_millis(i as u64 * 100),
            });
        }
    }

    /// Get total duration of recorded history
    pub fn duration(&self) -> Duration {
        if self.events.is_empty() {
            return Duration::ZERO;
        }

        let first = self.events.first().unwrap().received_at;
        let last = self.events.last().unwrap().received_at;
        last.duration_since(first)
    }

    /// Get number of recorded events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Enter replay mode
    pub fn start_replay(&mut self) {
        self.replay_mode = true;
        self.playback_index = 0;
        self.replay_start = Some(Instant::now());
        self.replay_offset = Duration::ZERO;
    }

    /// Exit replay mode
    pub fn stop_replay(&mut self) {
        self.replay_mode = false;
        self.replay_start = None;
    }

    /// Seek to a specific position (0.0 to 1.0)
    pub fn seek(&mut self, position: f32) {
        if self.events.is_empty() {
            return;
        }

        let position = position.clamp(0.0, 1.0);
        let target_index = ((self.events.len() - 1) as f32 * position) as usize;

        self.playback_index = target_index;
        self.replay_start = Some(Instant::now());
        self.replay_offset = self.duration().mul_f32(position);
    }

    /// Get current playback position (0.0 to 1.0)
    pub fn position(&self) -> f32 {
        if self.events.is_empty() {
            return 0.0;
        }
        self.playback_index as f32 / self.events.len() as f32
    }

    /// Get events to process for the current frame during replay
    pub fn get_replay_events(&mut self, speed: f32) -> Vec<HiveEvent> {
        if !self.replay_mode || self.events.is_empty() {
            return Vec::new();
        }

        let Some(start) = self.replay_start else {
            return Vec::new();
        };

        let elapsed = start.elapsed().mul_f32(speed) + self.replay_offset;
        let first_time = self.events.first().unwrap().received_at;
        let target_time = first_time + elapsed;

        let mut events = Vec::new();

        while self.playback_index < self.events.len() {
            let event = &self.events[self.playback_index];
            if event.received_at <= target_time {
                events.push(event.event.clone());
                self.playback_index += 1;
            } else {
                break;
            }
        }

        // Loop back to beginning if we've reached the end
        if self.playback_index >= self.events.len() {
            self.playback_index = 0;
            self.replay_start = Some(Instant::now());
            self.replay_offset = Duration::ZERO;
        }

        events
    }

    /// Get all events up to the current playback position
    pub fn get_events_to_position(&self) -> Vec<HiveEvent> {
        self.events
            .iter()
            .take(self.playback_index)
            .map(|e| e.event.clone())
            .collect()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
