use std::f32::consts::PI;

/// Pulse animation for agent brightness
#[derive(Debug, Clone)]
pub struct PulseAnimation {
    phase: f32,
    frequency: f32,
    min_value: f32,
    max_value: f32,
}

impl PulseAnimation {
    pub fn new(frequency: f32) -> Self {
        Self {
            phase: 0.0,
            frequency,
            min_value: 0.6,
            max_value: 1.0,
        }
    }

    /// Update animation state
    pub fn update(&mut self, dt: f32) {
        self.phase = (self.phase + dt * self.frequency * 2.0 * PI) % (2.0 * PI);
    }

    /// Get current value
    pub fn value(&self) -> f32 {
        let normalized = (self.phase.sin() + 1.0) / 2.0;
        self.min_value + normalized * (self.max_value - self.min_value)
    }

    /// Set the intensity (affects pulse amplitude)
    pub fn set_intensity(&mut self, intensity: f32) {
        self.frequency = 1.0 + intensity * 2.0;
        self.min_value = 0.5 + intensity * 0.3;
        self.max_value = 0.8 + intensity * 0.2;
    }
}

impl Default for PulseAnimation {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Breathing animation (slower, more organic)
pub fn breathing(time: f32, speed: f32) -> f32 {
    let t = time * speed;
    // Combine multiple sine waves for more organic feel
    let base = (t * PI).sin();
    let harmonic = (t * PI * 2.0).sin() * 0.2;
    (base + harmonic + 1.0) / 2.4 * 0.4 + 0.6
}

/// Heartbeat animation (quick pulse followed by pause)
pub fn heartbeat(time: f32, bpm: f32) -> f32 {
    let period = 60.0 / bpm;
    let t = (time % period) / period;

    if t < 0.1 {
        // First beat
        let x = t / 0.1;
        (x * PI).sin()
    } else if t < 0.2 {
        // First beat down
        let x = (t - 0.1) / 0.1;
        (1.0 - x) * (x * PI).cos().abs()
    } else if t < 0.25 {
        // Second beat
        let x = (t - 0.2) / 0.05;
        (x * PI).sin() * 0.7
    } else {
        // Rest
        0.0
    }
}

/// Flicker animation (random-ish noise)
pub fn flicker(time: f32, intensity: f32) -> f32 {
    let noise = ((time * 17.0).sin() * (time * 31.0).cos() + 1.0) / 2.0;
    1.0 - noise * intensity * 0.3
}
