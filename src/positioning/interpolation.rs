use super::Position;

/// Easing functions for smooth animations

/// Ease out cubic - fast start, slow end
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Ease in out cubic - slow start and end
pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Ease out elastic - bouncy effect
pub fn ease_out_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 || t == 1.0 {
        return t;
    }

    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
}

/// Smooth interpolation between positions with easing
pub fn smooth_lerp(from: &Position, to: &Position, t: f32, easing: EasingFunction) -> Position {
    let eased_t = match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseOutCubic => ease_out_cubic(t),
        EasingFunction::EaseInOutCubic => ease_in_out_cubic(t),
        EasingFunction::EaseOutElastic => ease_out_elastic(t),
    };

    from.lerp(to, eased_t)
}

/// Available easing functions
#[derive(Debug, Clone, Copy, Default)]
pub enum EasingFunction {
    Linear,
    #[default]
    EaseOutCubic,
    EaseInOutCubic,
    EaseOutElastic,
}

/// Smooth step function for gradual transitions
pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Perlin-like noise for organic movement (simplified)
pub fn pseudo_noise(x: f32, y: f32, seed: u32) -> f32 {
    let n = (x * 12.9898 + y * 78.233 + seed as f32).sin() * 43758.5453;
    n.fract()
}

/// Add organic jitter to a position
pub fn add_jitter(pos: &Position, amount: f32, time: f32) -> Position {
    let jitter_x = (time * 2.0).sin() * amount * 0.5
        + (time * 3.7).sin() * amount * 0.3
        + (time * 5.3).sin() * amount * 0.2;

    let jitter_y = (time * 2.3).cos() * amount * 0.5
        + (time * 3.1).cos() * amount * 0.3
        + (time * 4.7).cos() * amount * 0.2;

    Position::new(pos.x + jitter_x, pos.y + jitter_y).clamp()
}
