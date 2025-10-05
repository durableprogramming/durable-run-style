/// Easing functions for smooth animations
/// All functions take a normalized time value t in range [0.0, 1.0]
/// and return a normalized value in range [0.0, 1.0] (or slightly outside for overshoot effects)

use std::f32::consts::PI;

/// Linear interpolation (no easing)
pub fn linear(t: f32) -> f32 {
    t
}

/// Quadratic ease in (accelerating from zero)
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Quadratic ease out (decelerating to zero)
pub fn ease_out_quad(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Quadratic ease in-out
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// Cubic ease in
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Cubic ease out
pub fn ease_out_cubic(t: f32) -> f32 {
    let t1 = t - 1.0;
    t1 * t1 * t1 + 1.0
}

/// Cubic ease in-out
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t1 = 2.0 * t - 2.0;
        1.0 + t1 * t1 * t1 / 2.0
    }
}

/// Quartic ease in
pub fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

/// Quartic ease out
pub fn ease_out_quart(t: f32) -> f32 {
    let t1 = t - 1.0;
    1.0 - t1 * t1 * t1 * t1
}

/// Quartic ease in-out
pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t1 = t - 1.0;
        1.0 - 8.0 * t1 * t1 * t1 * t1
    }
}

/// Quintic ease in
pub fn ease_in_quint(t: f32) -> f32 {
    t * t * t * t * t
}

/// Quintic ease out
pub fn ease_out_quint(t: f32) -> f32 {
    let t1 = t - 1.0;
    1.0 + t1 * t1 * t1 * t1 * t1
}

/// Quintic ease in-out
pub fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let t1 = 2.0 * t - 2.0;
        1.0 + t1 * t1 * t1 * t1 * t1 / 2.0
    }
}

/// Sine ease in
pub fn ease_in_sine(t: f32) -> f32 {
    1.0 - ((t * PI) / 2.0).cos()
}

/// Sine ease out
pub fn ease_out_sine(t: f32) -> f32 {
    ((t * PI) / 2.0).sin()
}

/// Sine ease in-out
pub fn ease_in_out_sine(t: f32) -> f32 {
    -(((PI * t).cos()) - 1.0) / 2.0
}

/// Exponential ease in
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * t - 10.0)
    }
}

/// Exponential ease out
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

/// Exponential ease in-out
pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0_f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

/// Circular ease in
pub fn ease_in_circ(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

/// Circular ease out
pub fn ease_out_circ(t: f32) -> f32 {
    ((2.0 - t) * t).sqrt()
}

/// Circular ease in-out
pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - 4.0 * t * t).sqrt()) / 2.0
    } else {
        (((2.0 * t - 3.0) * (2.0 * t - 1.0)).sqrt() + 1.0) / 2.0
    }
}

/// Back ease in (overshoots backwards before moving forward)
pub fn ease_in_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    C3 * t * t * t - C1 * t * t
}

/// Back ease out (overshoots forward before settling)
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let t1 = t - 1.0;
    1.0 + C3 * t1 * t1 * t1 + C1 * t1 * t1
}

/// Back ease in-out
pub fn ease_in_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;
    if t < 0.5 {
        (2.0 * t * 2.0 * t * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
    } else {
        let t1 = 2.0 * t - 2.0;
        (t1 * t1 * ((C2 + 1.0) * t1 + C2) + 2.0) / 2.0
    }
}

/// Elastic ease in (elastic snap effect at start)
pub fn ease_in_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * PI) / 3.0;
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * C4).sin()
    }
}

/// Elastic ease out (elastic snap effect at end)
pub fn ease_out_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * PI) / 3.0;
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
    }
}

/// Elastic ease in-out
pub fn ease_in_out_elastic(t: f32) -> f32 {
    const C5: f32 = (2.0 * PI) / 4.5;
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
    } else {
        (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0 + 1.0
    }
}

/// Bounce ease in (bouncing at start)
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Bounce ease out (bouncing at end)
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t1 = t - 1.5 / D1;
        N1 * t1 * t1 + 0.75
    } else if t < 2.5 / D1 {
        let t1 = t - 2.25 / D1;
        N1 * t1 * t1 + 0.9375
    } else {
        let t1 = t - 2.625 / D1;
        N1 * t1 * t1 + 0.984375
    }
}

/// Bounce ease in-out
pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

/// Apply easing to a value range
pub fn ease_range(t: f32, start: f32, end: f32, easing_fn: fn(f32) -> f32) -> f32 {
    start + (end - start) * easing_fn(t)
}

/// Clamp a value to [0.0, 1.0] range
pub fn clamp_01(value: f32) -> f32 {
    value.max(0.0).min(1.0)
}

/// Interpolate between keyframes
/// keyframes: Vec of (time, value) pairs, assumed to be sorted by time
/// t: normalized time [0.0, 1.0]
pub fn keyframe_interpolate(keyframes: &[(f32, f32)], t: f32) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }
    if keyframes.len() == 1 {
        return keyframes[0].1;
    }

    // Find the segment where t falls
    for i in 0..keyframes.len() - 1 {
        let (t1, v1) = keyframes[i];
        let (t2, v2) = keyframes[i + 1];
        if t >= t1 && t <= t2 {
            // Linear interpolation between t1,v1 and t2,v2
            let factor = (t - t1) / (t2 - t1);
            return v1 + (v2 - v1) * factor;
        }
    }

    // If t is beyond the last keyframe, return the last value
    keyframes.last().unwrap().1
}

/// Smooth step interpolation (hermite interpolation)
pub fn smooth_step(t: f32) -> f32 {
    let t = clamp_01(t);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (improved hermite)
pub fn smoother_step(t: f32) -> f32 {
    let t = clamp_01(t);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_quad() {
        assert_eq!(ease_in_quad(0.0), 0.0);
        assert_eq!(ease_in_quad(1.0), 1.0);
        assert!(ease_in_quad(0.5) < 0.5); // Should be slower at start
    }

    #[test]
    fn test_ease_out_quad() {
        assert_eq!(ease_out_quad(0.0), 0.0);
        assert_eq!(ease_out_quad(1.0), 1.0);
        assert!(ease_out_quad(0.5) > 0.5); // Should be faster at start
    }

    #[test]
    fn test_ease_in_out_quad() {
        assert_eq!(ease_in_out_quad(0.0), 0.0);
        assert_eq!(ease_in_out_quad(1.0), 1.0);
        assert_eq!(ease_in_out_quad(0.5), 0.5);
    }

    #[test]
    fn test_clamp_01() {
        assert_eq!(clamp_01(-0.5), 0.0);
        assert_eq!(clamp_01(0.5), 0.5);
        assert_eq!(clamp_01(1.5), 1.0);
    }

    #[test]
    fn test_smooth_step() {
        assert_eq!(smooth_step(0.0), 0.0);
        assert_eq!(smooth_step(1.0), 1.0);
        assert_eq!(smooth_step(0.5), 0.5);
    }

    #[test]
    fn test_ease_range() {
        assert_eq!(ease_range(0.0, 10.0, 20.0, linear), 10.0);
        assert_eq!(ease_range(1.0, 10.0, 20.0, linear), 20.0);
        assert_eq!(ease_range(0.5, 10.0, 20.0, linear), 15.0);
    }

    #[test]
    fn test_keyframe_interpolate() {
        let keyframes = vec![(0.0, 10.0), (0.5, 20.0), (1.0, 30.0)];
        assert_eq!(keyframe_interpolate(&keyframes, 0.0), 10.0);
        assert_eq!(keyframe_interpolate(&keyframes, 0.25), 15.0);
        assert_eq!(keyframe_interpolate(&keyframes, 0.5), 20.0);
        assert_eq!(keyframe_interpolate(&keyframes, 0.75), 25.0);
        assert_eq!(keyframe_interpolate(&keyframes, 1.0), 30.0);
        assert_eq!(keyframe_interpolate(&keyframes, 1.5), 30.0); // beyond last
    }

    #[test]
    fn test_bounce_bounds() {
        assert_eq!(ease_out_bounce(0.0), 0.0);
        assert_eq!(ease_out_bounce(1.0), 1.0);
        assert!(ease_out_bounce(0.5) >= 0.0 && ease_out_bounce(0.5) <= 1.0);
    }

    #[test]
    fn test_elastic_bounds() {
        assert_eq!(ease_out_elastic(0.0), 0.0);
        assert_eq!(ease_out_elastic(1.0), 1.0);
    }

    #[test]
    fn test_back_overshoot() {
        // Back easing should go slightly negative/beyond 1.0
        assert!(ease_in_back(0.3) < 0.0);
        assert!(ease_out_back(0.7) > 1.0);
    }
}
