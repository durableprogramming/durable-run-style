use crate::theme::Theme;
use crate::utils::{blend_colors, format_bytes, format_runtime};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Padding},
    Frame,
};
use std::time::Duration;

const SPARKLINE_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn wrap_command_text(command: &[String], max_width: usize, max_lines: usize) -> Vec<String> {
    let full_command = command.join(" ");
    let wrap_indicator = "↩";
    let indicator_len = wrap_indicator.chars().count();
    let mut lines = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = full_command.chars().collect();

    while start < chars.len() {
        let remaining = chars.len() - start;
        let end = start + max_width.min(remaining);
        if end < chars.len() {
            // Will add indicator, so take less to make room
            let actual_end = start + (max_width - indicator_len).min(remaining);
            let line: String = chars[start..actual_end].iter().cloned().collect();
            lines.push(format!("{}{}", line, wrap_indicator));
            start = actual_end;
        } else {
            let line: String = chars[start..end].iter().cloned().collect();
            lines.push(line);
            start = end;
        }
    }

    // Limit to max_lines
    let was_truncated = lines.len() > max_lines;
    if was_truncated {
        lines.truncate(max_lines);
        // Since there are more lines, indicate truncation with ellipsis on the last line
        if let Some(last_line) = lines.last_mut() {
            // Remove any trailing wrap_indicator
            let line_without_indicator = last_line.trim_end_matches(wrap_indicator);
            // Truncate to make room for ellipsis
            let ellipsis = "…";
            let ellipsis_len = ellipsis.chars().count();
            let max_len = max_width.saturating_sub(ellipsis_len);
            let truncated_line: String = line_without_indicator.chars().take(max_len).collect();
            *last_line = format!("{}{}", truncated_line, ellipsis);
        }
    }

    lines
}

fn generate_sparkline(data: &[f32], width: usize, theme: &Theme) -> Vec<Span<'static>> {
    if data.is_empty() {
        return vec![Span::styled(" ".repeat(width), Style::default().fg(theme.secondary))];
    }
    let max_val = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_val = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let range = if (max_val - min_val).abs() < f32::EPSILON { 1.0 } else { max_val - min_val };
    data.iter().take(width).map(|&val| {
        let normalized = if range == 0.0 { 0.0 } else { (val - min_val) / range };
        let index = (normalized * (SPARKLINE_CHARS.len() - 1) as f32).round() as usize;
        let color = theme.shades[index];
        Span::styled(SPARKLINE_CHARS[index].to_string(), Style::default().fg(color))
    }).collect()
}

fn generate_sparkline_u64(data: &[u64], width: usize, theme: &Theme) -> Vec<Span<'static>> {
    if data.is_empty() {
        return vec![Span::styled(" ".repeat(width), Style::default().fg(theme.secondary))];
    }
    let max_val = *data.iter().max().unwrap_or(&0);
    let min_val = *data.iter().min().unwrap_or(&0);
    let range = if max_val == min_val { 1 } else { max_val - min_val };
    data.iter().take(width).map(|&val| {
        let normalized = if range == 0 { 0.0 } else { (val - min_val) as f32 / range as f32 };
        let index = (normalized * (SPARKLINE_CHARS.len() - 1) as f32).round() as usize;
        let color = theme.shades[index];
        Span::styled(SPARKLINE_CHARS[index].to_string(), Style::default().fg(color))
    }).collect()
}

pub struct DrawContext<'a> {
    pub command: &'a [String],
    pub pwd: &'a str,
    pub elapsed: Duration,
    pub pid: u32,
    pub ppid: u32,
    pub animation_frame: u32,
    pub theme: &'a Theme,
    pub output_lines: &'a [String],
    pub sidebar_width: u16,
    pub max_command_lines: usize,
    pub cpu_percent: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub child_network_rx: u64,
    pub child_network_tx: u64,
    pub cpu_history: &'a [f32],
    pub memory_history: &'a [u64],
    pub disk_read_history: &'a [u64],
    pub disk_write_history: &'a [u64],
    pub follow_mode: bool,
    pub scroll_offset: usize,
}

pub fn draw_ui(f: &mut Frame, context: DrawContext) {
    let size = f.size();
    let sidebar_width = context.sidebar_width;
    let main_rect = Rect::new(0, 0, size.width.saturating_sub(sidebar_width), size.height);
    let sidebar_rect = Rect::new(size.width.saturating_sub(sidebar_width), 0, sidebar_width, size.height);

    // Draw main area with output
    let output_text = context.output_lines.join("\n");
    let output_block = Block::default()
        .borders(Borders::ALL)
        .title(if context.follow_mode { "Output (Follow)" } else { "Output (Scroll)" })
        .border_style(Style::default().fg(context.theme.primary))
        .title_style(Style::default().fg(context.theme.primary).add_modifier(Modifier::BOLD));

    // Calculate scroll position
    let total_lines = context.output_lines.len();
    let visible_height = main_rect.height.saturating_sub(2) as usize; // Subtract border height
    let scroll_pos = if context.follow_mode {
        // In follow mode, scroll to show the latest lines
        total_lines.saturating_sub(visible_height)
    } else {
        // In scroll mode, scroll_offset is lines scrolled up from bottom
        (total_lines.saturating_sub(visible_height)).saturating_sub(context.scroll_offset)
    };

    let output_paragraph = Paragraph::new(output_text)
        .block(output_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_pos as u16, 0));
    f.render_widget(output_paragraph, main_rect);

    // Draw sidebar with border
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .border_style(Style::default().fg(context.theme.border).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(context.theme.background));

    let inner_area = block.inner(sidebar_rect);
    f.render_widget(block, sidebar_rect);

    // Create content
    let mut lines = Vec::new();

    // Header with shine effect
    let header_lines: Vec<&str> = include_str!("../ascii_art.txt").lines().filter(|line| !line.trim().is_empty()).collect();
    let content_width = inner_area.width as usize;
    let remaining_width = content_width - 11;
    let shine_width = 3.0;
    let total_cycle_frames = 110;
    let cycle_frame = context.animation_frame % total_cycle_frames;

    let mut line_len = 0;
    for line in header_lines {

        if line_len == 0 {
            line_len = line.chars().count();
        }
        let total_pad = content_width.saturating_sub(line_len);
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;
        let padded_line = format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(right_pad));
        let line_len_f32 = padded_line.chars().count() as f32;
        let shine_position = if cycle_frame < 30 {
            // moving left to right
            let progress = cycle_frame as f32 / 29.0;
            progress * (line_len_f32 + shine_width * 2.0) - shine_width
        } else if cycle_frame < 55 {
            // pause at right
            line_len_f32 + shine_width
        } else if cycle_frame < 85 {
            // moving right to left
            let progress = (cycle_frame - 55) as f32 / 29.0;
            (1.0 - progress) * (line_len_f32 + shine_width * 2.0) - shine_width
        } else {
            // pause at left
            -shine_width
        };

        let mut spans = Vec::new();
        for (char_idx, ch) in padded_line.chars().enumerate() {
            let distance_from_shine = (char_idx as f32 - shine_position).abs();
            let shine_intensity = if distance_from_shine <= shine_width {
                (1.0 - (distance_from_shine / shine_width)) * 0.5
            } else {
                0.0
            };

            let color = if shine_intensity > 0.0 {
                blend_colors(context.theme.primary, ratatui::style::Color::White, shine_intensity)
            } else {
                context.theme.primary
            };

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }
        lines.push(Line::from(spans));
    }

    // Empty line for spacing
    lines.push(Line::from(""));


    // Command line (wrapped)
    let wrapped_command_lines = wrap_command_text(context.command, content_width, context.max_command_lines);
    for line in wrapped_command_lines {
        lines.push(Line::from(Span::styled(
            line,
            Style::default().fg(context.theme.text),
        )));
    }

    // PWD line (wrapped)
    let pwd_with_label = format!("PWD: {}", context.pwd);
    let wrapped_pwd_lines = wrap_command_text(&[pwd_with_label], content_width, context.max_command_lines);
    for line in wrapped_pwd_lines {
        lines.push(Line::from(Span::styled(
            line,
            Style::default().fg(context.theme.secondary),
        )));
    }

    // Empty line for spacing between runtime and command
    lines.push(Line::from(""));

    // Runtime
    let runtime_value = format_runtime(context.elapsed);
    let runtime_text = format!("{:<10}{:>width$}", "Runtime:", runtime_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        runtime_text,
        Style::default().fg(context.theme.secondary).add_modifier(Modifier::BOLD),
    )));

    // PID
    let pid_value = context.pid.to_string();
    let pid_text = format!("{:<10}{:>width$}", "PID:", pid_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        pid_text,
        Style::default().fg(context.theme.accent),
    )));

    // PPID
    let ppid_value = context.ppid.to_string();
    let ppid_text = format!("{:<10}{:>width$}", "PPID:", ppid_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        ppid_text,
        Style::default().fg(context.theme.accent),
    )));

    // CPU sparkline
    let cpu_sparkline_spans = generate_sparkline(context.cpu_history, content_width, &context.theme);
    lines.push(Line::from(cpu_sparkline_spans));
    // Padding below flamegraph
    lines.push(Line::from(""));

    // CPU usage
    let cpu_value = format!("{:.1}%", context.cpu_percent);
    let cpu_text = format!("{:<10}{:>width$}", "CPU:", cpu_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        cpu_text,
        Style::default().fg(context.theme.accent),
    )));

    // Memory sparkline
    let memory_sparkline_spans = generate_sparkline_u64(context.memory_history, content_width, &context.theme);
    lines.push(Line::from(memory_sparkline_spans));
    // Padding below flamegraph
    lines.push(Line::from(""));

    // Memory usage
    let memory_percent = if context.memory_total > 0 {
        (context.memory_used as f64 / context.memory_total as f64 * 100.0).round() as u8
    } else {
        0
    };
    let memory_value = format!("{} ({:.1}%)", format_bytes(context.memory_used),  memory_percent);
    let memory_text = format!("{:<10}{:>width$}", "Memory:", memory_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        memory_text,
        Style::default().fg(context.theme.accent),
    )));

    // Disk IO sparkline (combined read and write, using read for simplicity)
    let disk_sparkline_spans = generate_sparkline_u64(context.disk_read_history, content_width, &context.theme);
    lines.push(Line::from(disk_sparkline_spans));
    // Padding below flamegraph
    lines.push(Line::from(""));

    // Disk IO
    let disk_value = format!("R {} W {}", format_bytes(context.disk_read), format_bytes(context.disk_write));
    let disk_text = format!("{:<10}{:>width$}", "Disk IO:", disk_value, width = remaining_width);
    lines.push(Line::from(Span::styled(
        disk_text,
        Style::default().fg(context.theme.accent),
    )));

    // Network usage
    #[cfg(feature = "experimental-pcap")]
    {
        // Network sparkline (using RX for simplicity, but since network is not in history, perhaps skip or use a placeholder)
        // Since network is not historied, use a static sparkline or skip
        let network_sparkline_spans = vec![Span::styled(" ".repeat(content_width), Style::default().fg(context.theme.secondary))]; // Placeholder, since no history
        lines.push(Line::from(network_sparkline_spans));
        // Padding below flamegraph
        lines.push(Line::from(""));

        let network_value = format!("RX {} TX {}", format_bytes(context.child_network_rx), format_bytes(context.child_network_tx));
        let network_text = format!("{:<10}{:>width$}", "Network:", network_value, width = remaining_width);
        lines.push(Line::from(Span::styled(
            network_text,
            Style::default().fg(context.theme.accent),
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);
}

pub struct GfxDemoDrawContext<'a> {
    pub animation_frame: u32,
    pub theme: &'a Theme,
    pub crunch_result: f64,
    pub memory_allocated: usize,
    pub memory_fluctuation: &'a [f64],
    pub cpu_load: &'a [f64],
    pub elapsed: Duration,
    pub display_states: &'a [(f64, f64)],
    pub status_updates: &'a [String],
}

pub fn draw_gfx_demo_ui(f: &mut Frame, context: GfxDemoDrawContext) {
    use ratatui::layout::{Layout, Direction, Constraint};

    let size = f.size();

    // Split screen horizontally: left for current display, right for status updates
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left side: current technobabble display
            Constraint::Percentage(40), // Right side: scrolling status updates
        ])
        .split(size);

    // Left side: Current technobabble display
    let mut left_lines = Vec::new();

    // Title
    left_lines.push(Line::from(Span::styled(
        "🖥️ STARSHIP SYSTEMS MONITOR 🖥️",
        Style::default().fg(context.theme.primary).add_modifier(Modifier::BOLD),
    )));
    left_lines.push(Line::from(""));

    // Runtime
    let runtime_value = format_runtime(context.elapsed);
    left_lines.push(Line::from(vec![
        Span::styled("Mission Elapsed Time: ", Style::default().fg(context.theme.secondary)),
        Span::styled(runtime_value, Style::default().fg(context.theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    left_lines.push(Line::from(""));

    // Technobabble lines with random values
    let technobabble = vec![
        ("Warp Core Containment Field Integrity", "%"),
        ("Photon Torpedo Yield", "GW"),
        ("Dilithium Crystal Resonance Frequency", "THz"),
        ("Plasma Conduit Pressure", "MPa"),
        ("Quantum Flux Capacitor Charge", "%"),
        ("Subspace Communication Bandwidth", "kbps"),
        ("Antimatter Containment Efficiency", "%"),
        ("Transporter Buffer Capacity", "kg"),
        ("Deflector Shield Harmonics", "Hz"),
        ("Inertial Dampener Field Strength", "Gs"),
        ("Navigational Deflector Power Output", "TW"),
        ("Life Support Oxygen Regeneration Rate", "%"),
        ("Tachyon Pulse Emission Frequency", "MHz"),
        ("Gravimetric Field Density", "g/cm³"),
        ("Chroniton Particle Flux", "particles/s"),
        ("Neutrino Radiation Shielding", "%"),
        ("Hyperdrive Engine Synchronization", "97.8%"),
        ("Cloaking Device Phase Variance", ""),
        ("Tractor Beam Coherence", "%"),
        ("Sensor Array Resolution", "arcseconds"),
    ];

    for (i, (label, unit)) in technobabble.iter().enumerate() {
        let (value, _) = context.display_states[i];
        let is_warning = value > 90.0 || value < 10.0;
        let value_color = if is_warning {
            ratatui::style::Color::Red
        } else {
            context.theme.shades[(context.animation_frame.wrapping_add(label.len() as u32) / 10 % context.theme.shades.len() as u32) as usize]
        };
        let formatted_value = format!("{:.8}", value);
        left_lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(context.theme.secondary)),
            Span::styled(format!("{} {}", formatted_value, unit), Style::default().fg(value_color).add_modifier(Modifier::BOLD)),
        ]));
    }

    left_lines.push(Line::from(""));

    // Instructions
    left_lines.push(Line::from(Span::styled(
        "Press 'q' or Ctrl+C to exit",
        Style::default().fg(context.theme.secondary),
    )));

    // Add some animated elements
    let animation_phase = (context.animation_frame / 5) % 4;
    let spinner = match animation_phase {
        0 => "🌀",
        1 => "⚡",
        2 => "🔥",
        3 => "💫",
        _ => "🌀",
    };
    left_lines.push(Line::from(vec![
        Span::styled("Systems Status: ", Style::default().fg(context.theme.secondary)),
        Span::styled(spinner, Style::default().fg(context.theme.accent).add_modifier(Modifier::BOLD)),
    ]));

    // Left side block and paragraph
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title("Systems Monitor")
        .border_style(Style::default().fg(context.theme.primary))
        .title_style(Style::default().fg(context.theme.primary).add_modifier(Modifier::BOLD))
        .padding(Padding::horizontal(1));

    let left_inner = left_block.inner(chunks[0]);
    f.render_widget(left_block, chunks[0]);

    let left_paragraph = Paragraph::new(left_lines);
    f.render_widget(left_paragraph, left_inner);

    // Right side: Scrolling status updates
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title("Status Updates")
        .border_style(Style::default().fg(context.theme.primary))
        .title_style(Style::default().fg(context.theme.primary).add_modifier(Modifier::BOLD))
        .padding(Padding::horizontal(1));

    let right_inner = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    // Create status update lines (newest at bottom)
    let mut right_lines = Vec::new();
    for update in context.status_updates.iter() {
        right_lines.push(Line::from(Span::styled(
            update.clone(),
            Style::default().fg(context.theme.secondary),
        )));
    }

    let right_paragraph = Paragraph::new(right_lines)
        .scroll((context.status_updates.len().saturating_sub(right_inner.height as usize) as u16, 0)); // Scroll to show latest
    f.render_widget(right_paragraph, right_inner);
}



fn generate_sparkline_f64(data: &[f64], width: usize, theme: &Theme) -> Vec<Span<'static>> {
    if data.is_empty() {
        return vec![Span::styled(" ".repeat(width), Style::default().fg(theme.secondary))];
    }
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let range = if (max_val - min_val).abs() < f64::EPSILON { 1.0 } else { max_val - min_val };
    data.iter().take(width).map(|&val| {
        let normalized = if range == 0.0 { 0.0 } else { (val - min_val) / range };
        let index = (normalized * (SPARKLINE_CHARS.len() - 1) as f64).round() as usize;
        let color = theme.shades[index];
        Span::styled(SPARKLINE_CHARS[index].to_string(), Style::default().fg(color))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_command_text_short() {
        let command = vec!["short".to_string(), "command".to_string()];
        let result = wrap_command_text(&command, 20, 3);
        assert_eq!(result, vec!["short command"]);
    }

    #[test]
    fn test_wrap_command_text_long_single_line() {
        let command = vec!["this".to_string(), "is".to_string(), "a".to_string(), "very".to_string(), "long".to_string(), "command".to_string(), "that".to_string(), "should".to_string(), "wrap".to_string()];
        let result = wrap_command_text(&command, 20, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "this is a very long↩");
        assert_eq!(result[1], " command that shoul↩");
        assert_eq!(result[2], "d wrap");
    }

    #[test]
    fn test_wrap_command_text_max_lines() {
        let command = vec!["word1".to_string(), "word2".to_string(), "word3".to_string(), "word4".to_string(), "word5".to_string(), "word6".to_string(), "word7".to_string()];
        let result = wrap_command_text(&command, 10, 2);
        assert_eq!(result.len(), 2);
        assert!(result[1].ends_with("…"));
    }

    #[test]
    fn test_wrap_command_text_no_wrap_needed() {
        let command = vec!["hello".to_string(), "world".to_string()];
        let result = wrap_command_text(&command, 20, 3);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_wrap_pwd_text() {
        let pwd = "/very/long/path/to/some/directory/that/might/wrap".to_string();
        let result = wrap_command_text(&[pwd], 20, 3);
        assert!(result.len() >= 1);
        assert!(result[0].contains("↩") || result.len() == 1);
    }
}