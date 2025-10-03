pub fn format_runtime(elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    format!("{:02}:{:02}:{:02}", hours, mins % 60, secs % 60)
}

pub fn blend_colors(base: ratatui::style::Color, shine: ratatui::style::Color, intensity: f32) -> ratatui::style::Color {
    let intensity = intensity.clamp(0.0, 1.0);

    let (base_r, base_g, base_b) = match base {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255), // Default to white if not RGB
    };

    let (shine_r, shine_g, shine_b) = match shine {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255), // Default to white if not RGB
    };

    let blended_r = ((base_r as f32 * (1.0 - intensity) + shine_r as f32 * intensity) as u8).clamp(0, 255);
    let blended_g = ((base_g as f32 * (1.0 - intensity) + shine_g as f32 * intensity) as u8).clamp(0, 255);
    let blended_b = ((base_b as f32 * (1.0 - intensity) + shine_b as f32 * intensity) as u8).clamp(0, 255);

    ratatui::style::Color::Rgb(blended_r, blended_g, blended_b)
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.1}{}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_runtime() {
        assert_eq!(format_runtime(Duration::from_secs(0)), "00:00:00");
        assert_eq!(format_runtime(Duration::from_secs(59)), "00:00:59");
        assert_eq!(format_runtime(Duration::from_secs(60)), "00:01:00");
        assert_eq!(format_runtime(Duration::from_secs(3661)), "01:01:01");
    }
}