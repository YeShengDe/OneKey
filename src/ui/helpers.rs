use ratatui::{
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
};
use std::time::Duration;

use crate::theme::Theme;

/// 创建标准的块组件
pub fn create_block(title: &str, is_focused: bool) -> Block {
    let border_style = if is_focused {
        Style::default().fg(Theme::PRIMARY)
    } else {
        Style::default().fg(Theme::MUTED)
    };
    
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}

/// 格式化时间持续
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    
    if minutes > 0 {
        format!("{}:{:02}", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// 根据进度获取颜色
pub fn get_progress_color(progress: f64) -> Style {
    let color = if progress < 30.0 {
        Color::Red
    } else if progress < 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };
    
    Style::default().fg(color)
}

/// 创建增强的进度条
pub fn create_enhanced_progress_bar(progress: u8, width: usize) -> Span<'static> {
    let filled = (progress as usize * width) / 100;
    let empty = width - filled;
    
    let color = if progress < 30 {
        Theme::ERROR
    } else if progress < 70 {
        Theme::WARNING
    } else {
        Theme::SUCCESS
    };
    
    let bar = format!("[{}{}]", 
        "█".repeat(filled),
        "░".repeat(empty)
    );
    
    Span::styled(bar, Style::default().fg(color))
}

/// 创建速度条形图
pub fn create_speed_bar(speed: f64, max_speed: f64, is_read: bool) -> Span<'static> {
    let width = 20;
    let filled = ((speed / max_speed) * width as f64).min(width as f64) as usize;
    let empty = width - filled;
    
    let color = if is_read {
        if speed >= 100.0 {
            Theme::SUCCESS
        } else if speed >= 10.0 {
            Theme::WARNING
        } else {
            Theme::ERROR
        }
    } else {
        if speed >= 50.0 {
            Theme::SUCCESS
        } else if speed >= 5.0 {
            Theme::WARNING
        } else {
            Theme::ERROR
        }
    };
    
    let bar = format!("  [{}{}]", 
        "▇".repeat(filled),
        "▁".repeat(empty)
    );
    
    Span::styled(bar, Style::default().fg(color))
}

/// 格式化速度显示
pub fn format_speed(speed: f64) -> String {
    if speed >= 1000.0 {
        format!("{:.2} GB/s", speed / 1024.0)
    } else {
        format!("{:.2} MB/s", speed)
    }
}

/// 格式化IOPS显示
pub fn format_iops(iops: f64) -> String {
    if iops >= 1000.0 {
        format!("{:.1}K", iops / 1000.0)
    } else {
        format!("{:.0}", iops)
    }
}
