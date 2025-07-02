use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Gauge, Padding},
    Frame,
};

use crate::{app::App, theme::Theme};
use super::components::draw_scrollbar;

/// 绘制磁盘测试内容
pub fn draw_disk_test_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取磁盘测试信息
    let test_info = crate::handlers::disk_test::get_current_test_info();
    
    if test_info.is_testing || !test_info.results.is_empty() {
        // 显示合并的测试和结果界面
        draw_combined_test_results_ui(f, app, area, &test_info, is_focused);
        return;
    }
    
    // 显示准备状态或错误信息的静态界面
    draw_disk_test_static_content(f, app, area, &test_info, is_focused);
}

// 绘制合并的测试和结果界面
fn draw_combined_test_results_ui(f: &mut Frame, app: &mut App, area: Rect, test_info: &crate::handlers::disk_test::DiskTestInfo, is_focused: bool) {
    // 创建上下布局
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // 上部：实时测试状态
            Constraint::Min(6),      // 下部：测试结果
        ].as_ref())
        .split(area);

    // 上部分：实时测试状态
    draw_realtime_test_status(f, main_chunks[0], test_info, is_focused);
    
    // 下部分：测试结果
    if !test_info.results.is_empty() {
        draw_compact_test_results(f, app, main_chunks[1], &test_info.results, is_focused);
    } else if test_info.is_testing {
        draw_testing_progress_info(f, main_chunks[1], test_info, is_focused);
    }
}

// 绘制实时测试状态（上半部分）
fn draw_realtime_test_status(f: &mut Frame, area: Rect, test_info: &crate::handlers::disk_test::DiskTestInfo, is_focused: bool) {
    // 确定当前测试模式
    let is_read_test = test_info.current_test_phase.contains("读") || test_info.current_test_phase.contains("read");
    let is_write_test = test_info.current_test_phase.contains("写") || test_info.current_test_phase.contains("write");
    
    let current_speed = if is_read_test {
        test_info.current_read_speed
    } else if is_write_test {
        test_info.current_write_speed
    } else {
        0.0
    };
    
    let current_iops = if is_read_test {
        test_info.current_read_iops
    } else if is_write_test {
        test_info.current_write_iops
    } else {
        0.0
    };

    // 创建三列布局：测试类型(25%) | 进度条(50%) | IOPS(25%)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // 左列：测试类型
            Constraint::Percentage(50), // 中列：进度条和速度
            Constraint::Percentage(25), // 右列：IOPS
        ].as_ref())
        .split(area);

    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };

    // 左上角：测试类型（紧凑显示）
    let test_type = if is_read_test {
        "📖 读取"
    } else if is_write_test {
        "📤 写入"
    } else {
        "🔄 准备"
    };

    let test_type_block = Block::default()
        .borders(Borders::ALL)
        .title(" 测试 ")
        .title_style(title_style)
        .border_style(border_style);
    
    let test_inner = test_type_block.inner(columns[0]);
    f.render_widget(test_type_block, columns[0]);
    
    // 在内部区域居中显示内容
    if test_inner.height > 0 {
        let test_content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 测试类型
                Constraint::Length(1), // 速度
            ].as_ref())
            .split(test_inner);
        
        // 测试类型行
        f.render_widget(
            ratatui::widgets::Paragraph::new(test_type)
                .style(Theme::accent().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            test_content_area[0]
        );
        
        // 速度行
        if test_content_area.len() > 1 {
            let speed_text = format!("{:.1} MB/s", current_speed);
            f.render_widget(
                ratatui::widgets::Paragraph::new(speed_text)
                    .style(ratatui::style::Style::default()
                        .fg(Theme::chart_read_color())
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center),
                test_content_area[1]
            );
        }
    }

    // 中间：进度条（铺满）
    let clamped_progress = test_info.progress.min(100); // 确保进度不超过100%
    let progress_ratio = (clamped_progress as f64 / 100.0).max(0.0).min(1.0);
    
    // 创建上下布局用于进度条
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 总体进度
            Constraint::Length(3), // 速度仪表盘
        ].as_ref())
        .split(columns[1]);

    // 总体进度条
    let progress_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 总体进度 ")
                .title_style(title_style)
                .border_style(border_style)
        )
        .gauge_style(Theme::disk_test_progress_style(clamped_progress as f64))
        .ratio(progress_ratio)
        .label(format!("{}%", clamped_progress));

    f.render_widget(progress_gauge, progress_chunks[0]);

    // 速度仪表盘
    let base_max_speed = 1000.0; // 1GB/s
    let max_speed = if current_speed > base_max_speed {
        current_speed * 1.4
    } else {
        base_max_speed
    };
    
    // 安全的比例计算，确保在 0.0-1.0 范围内
    let speed_ratio = if max_speed <= 0.0 || !current_speed.is_finite() || !max_speed.is_finite() {
        0.0
    } else {
        (current_speed / max_speed).max(0.0).min(1.0)
    };
    
    let speed_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 速度 ")
                .title_style(title_style)
                .border_style(border_style)
        )
        .gauge_style(ratatui::style::Style::default().fg(Theme::chart_read_color()))
        .ratio(speed_ratio)
        .label(format!("{:.1} MB/s", current_speed));

    f.render_widget(speed_gauge, progress_chunks[1]);

    // 右上角：IOPS（紧凑显示）
    let iops_block = Block::default()
        .borders(Borders::ALL)
        .title(" IOPS ")
        .title_style(title_style)
        .border_style(border_style);
    
    let iops_inner = iops_block.inner(columns[2]);
    f.render_widget(iops_block, columns[2]);
    
    // 在内部区域居中显示内容
    if iops_inner.height > 0 {
        let iops_content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 标题
                Constraint::Length(1), // 数值
            ].as_ref())
            .split(iops_inner);
        
        // IOPS标题行
        f.render_widget(
            ratatui::widgets::Paragraph::new("📊 IOPS")
                .style(Theme::accent().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            iops_content_area[0]
        );
        
        // IOPS数值行
        if iops_content_area.len() > 1 {
            let iops_text = format!("{:.0} ops/s", current_iops);
            f.render_widget(
                ratatui::widgets::Paragraph::new(iops_text)
                    .style(ratatui::style::Style::default()
                        .fg(Theme::chart_read_color())
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center),
                iops_content_area[1]
            );
        }
    }
}

// 绘制紧凑的测试结果（下半部分）- FIO风格表格
fn draw_compact_test_results(f: &mut Frame, app: &mut App, area: Rect, results: &[crate::handlers::disk_test::DiskTestResult], is_focused: bool) {
    let mut items = Vec::new();
    
    // 创建清晰易读的卡片式表格显示
    let block_sizes = ["4K", "64K", "512K", "1M"];
    
    // 检查是否有多块大小测试结果
    let has_multi_block_results = block_sizes.iter().any(|&block_size| {
        results.iter().any(|r| r.test_name.contains(block_size))
    });
    
    if has_multi_block_results {
        // 卡片式表格显示
        draw_fio_style_table(&mut items, results, &block_sizes);
    } else {
        // 传统简单显示
        draw_simple_results(&mut items, results);
    }
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2);
    app.update_content_height(content_height, viewport_height);
    
    // 创建可见内容
    let visible_items = if content_height > viewport_height {
        items
            .iter()
            .skip(app.scroll_position.current as usize)
            .take(viewport_height as usize)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        items
    };
    
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📊 测试结果 ")
                .title_style(title_style)
                .border_style(border_style)
                .padding(Padding::uniform(1)),
        );
    
    f.render_widget(list, area);
    
    // 绘制滚动条（如果需要）
    if content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}

// 绘制清晰易读的卡片式表格
fn draw_fio_style_table(items: &mut Vec<ListItem>, results: &[crate::handlers::disk_test::DiskTestResult], block_sizes: &[&str]) {
    // 创建卡片式的结果展示，每个块大小一个卡片
    for (idx, &block_size) in block_sizes.iter().enumerate() {
        if idx > 0 {
            items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        }
        
        // 查找当前块大小的测试结果
        let read_result = results.iter().find(|r| r.test_name == format!("{} 读取", block_size));
        let write_result = results.iter().find(|r| r.test_name == format!("{} 写入", block_size));
        let total_result = results.iter().find(|r| r.test_name == format!("{} 总计", block_size));
        
        // 如果没有找到结果，跳过这个块大小
        if read_result.is_none() && write_result.is_none() {
            continue;
        }
        
        // 卡片标题 - 块大小
        items.push(ListItem::new(Line::from(vec![
            Span::styled("┌─ ", Theme::accent()),
            Span::styled(format!("📊 {} 块大小测试", block_size), Theme::primary().add_modifier(Modifier::BOLD)),
            Span::styled(" ─".repeat(40), Theme::accent()),
        ])));
        
        // 读取性能
        if let Some(read) = read_result {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("│ ", Theme::accent()),
                Span::styled("📖 读取性能: ", Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>12}", read.read_speed), Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled("  │  IOPS: ", Theme::muted()),
                Span::styled(format!("{:>8}", read.read_iops), Theme::success()),
            ])));
        }
        
        // 写入性能
        if let Some(write) = write_result {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("│ ", Theme::accent()),
                Span::styled("📤 写入性能: ", Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>12}", write.write_speed), Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled("  │  IOPS: ", Theme::muted()),
                Span::styled(format!("{:>8}", write.write_iops), Theme::success()),
            ])));
        }
        
        // 总计性能（如果有）
        if let Some(total) = total_result {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("│ ", Theme::accent()),
                Span::styled("─".repeat(56), Theme::muted()),
            ])));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("│ ", Theme::accent()),
                Span::styled("🔄 总计性能: ", Theme::accent().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>12}", total.total_speed), Theme::accent().add_modifier(Modifier::BOLD)),
                Span::styled("  │  IOPS: ", Theme::muted()),
                Span::styled(format!("{:>8}", total.total_iops), Theme::accent()),
            ])));
        }
        
        // 卡片底部
        items.push(ListItem::new(Line::from(vec![
            Span::styled("└─", Theme::accent()),
            Span::styled("─".repeat(54), Theme::accent()),
        ])));
    }
        
    
    // 找出最佳性能
    let mut best_read_speed = 0.0;
    let mut best_write_speed = 0.0;
    let mut best_read_block = "";
    let mut best_write_block = "";
    
    for &block_size in block_sizes {
        if let Some(read) = results.iter().find(|r| r.test_name == format!("{} 读取", block_size)) {
            let speed_val = parse_speed_value(&read.read_speed);
            if speed_val > best_read_speed {
                best_read_speed = speed_val;
                best_read_block = block_size;
            }
        }
        if let Some(write) = results.iter().find(|r| r.test_name == format!("{} 写入", block_size)) {
            let speed_val = parse_speed_value(&write.write_speed);
            if speed_val > best_write_speed {
                best_write_speed = speed_val;
                best_write_block = block_size;
            }
        }
    }
    
    if best_read_speed > 0.0 {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🏆 最佳读取: ", Theme::success()),
            Span::styled(format!("{} 块大小", best_read_block), Theme::success().add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({:.1} MB/s)", best_read_speed), Theme::muted()),
        ])));
    }
    
    if best_write_speed > 0.0 {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🏆 最佳写入: ", Theme::success()),
            Span::styled(format!("{} 块大小", best_write_block), Theme::success().add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({:.1} MB/s)", best_write_speed), Theme::muted()),
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 说明
    items.push(ListItem::new(Line::from(vec![
        Span::styled("💡 说明: ", Theme::accent()),
        Span::styled("较大的块大小通常有更高的顺序读写性能，较小的块大小有更好的随机访问性能", Theme::muted())
    ])));
}

// 解析速度值为数字（MB/s）
fn parse_speed_value(speed_str: &str) -> f64 {
    if speed_str == "N/A" {
        return 0.0;
    }
    
    let clean_str = speed_str.replace(" MB/s", "").replace(" GB/s", "").replace(",", "");
    if let Ok(mut value) = clean_str.parse::<f64>() {
        // 如果是GB/s，转换为MB/s
        if speed_str.contains("GB/s") {
            value *= 1024.0;
        }
        value
    } else {
        0.0
    }
}

// 绘制简单结果显示
fn draw_simple_results(items: &mut Vec<ListItem>, results: &[crate::handlers::disk_test::DiskTestResult]) {
    // 为每个测试结果创建紧凑显示
    for result in results.iter() {
        // 测试名称行
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("📊 {}", result.test_name), Theme::accent())
        ])));
        
        // 性能数据行（更紧凑）
        let mut performance_spans = vec![];
        
        if result.read_speed != "N/A" {
            performance_spans.extend(vec![
                Span::styled("📖 ", Theme::success()),
                Span::styled(format!("{}", result.read_speed), Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", result.read_iops), Theme::muted()),
            ]);
        }
        
        if result.write_speed != "N/A" {
            if !performance_spans.is_empty() {
                performance_spans.push(Span::styled(" | ", Theme::muted()));
            }
            performance_spans.extend(vec![
                Span::styled("📤 ", Theme::success()),
                Span::styled(format!("{}", result.write_speed), Theme::success().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", result.write_iops), Theme::muted()),
            ]);
        }
        
        if !performance_spans.is_empty() {
            items.push(ListItem::new(Line::from(performance_spans)));
        }
    }
}

// 绘制测试进行中的信息
fn draw_testing_progress_info(f: &mut Frame, area: Rect, test_info: &crate::handlers::disk_test::DiskTestInfo, is_focused: bool) {
    let elapsed_time = if let Some(start_time) = test_info.test_start_time {
        start_time.elapsed().as_secs()
    } else {
        0
    };

    // 动画效果
    let animation_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
    let animation_char = animation_chars[test_info.animation_frame % animation_chars.len()];

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🔄 正在进行磁盘性能测试", Theme::primary().add_modifier(Modifier::BOLD))
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(animation_char, Theme::spinning_animation()),
            Span::styled(format!(" {}", test_info.current_test_phase), Theme::secondary())
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⏱️  运行时间: ", Theme::accent()),
            Span::styled(format!("{}秒", elapsed_time), Theme::secondary()),
            Span::styled(" | 数据点: ", Theme::accent()),
            Span::styled(format!("{}", test_info.realtime_data.len()), Theme::secondary())
        ])),
    ];

    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 测试状态 ")
                .title_style(title_style)
                .border_style(border_style)
                .padding(Padding::uniform(1)),
        );

    f.render_widget(list, area);
}

// 静态内容显示函数（准备状态或错误）
fn draw_disk_test_static_content(f: &mut Frame, app: &mut App, area: Rect, test_info: &crate::handlers::disk_test::DiskTestInfo, is_focused: bool) {
    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("💾 磁盘性能测试", Theme::primary().add_modifier(Modifier::BOLD))
        ])),
        ListItem::new(Line::from(vec![Span::raw("")])),
    ];

    if let Some(ref error) = test_info.error_message {
        // 显示错误信息
        items.push(ListItem::new(Line::from(vec![
            Span::styled("❌ 错误: ", Theme::error()),
            Span::styled(error.clone(), Theme::error().add_modifier(Modifier::BOLD))
        ])));
        items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    } else {
        // 准备测试状态
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🔄 准备执行磁盘测试...", Theme::accent().add_modifier(Modifier::ITALIC))
        ])));
        items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    }
    
    // 工具检查状态 - 使用卡片式布局
    items.push(ListItem::new(Line::from(vec![
        Span::styled("🔧 测试工具检查", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(30), Theme::muted())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("📦 FIO: ", Theme::accent()),
        Span::styled(
            if test_info.has_fio { "✅ 已安装" } else { "❌ 未安装" },
            if test_info.has_fio { Theme::success() } else { Theme::success() }
        ),
        Span::styled(
            if test_info.has_fio { " (专业测试)" } else { " (建议安装)" },
            Theme::muted()
        )
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("📦 DD: ", Theme::accent()),
        Span::styled(
            if test_info.has_dd { "✅ 已安装" } else { "❌ 未安装" },
            if test_info.has_dd { Theme::success() } else { Theme::success() }
        ),
        Span::styled(
            if test_info.has_dd { " (基础测试)" } else { " (系统工具)" },
            Theme::muted()
        )
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("🦀 Rust内置: ", Theme::accent()),
        Span::styled("✅ 可用", Theme::success()),
        Span::styled(" (无需外部依赖)", Theme::muted())
    ])));
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 磁盘使用情况 - 格式化显示
    items.push(ListItem::new(Line::from(vec![
        Span::styled("📊 磁盘使用情况", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(30), Theme::muted())
    ])));
    
    if !test_info.disk_usage.is_empty() {
        let lines: Vec<&str> = test_info.disk_usage.lines().collect();
        if !lines.is_empty() && lines[0].contains("Filesystem") {
            // 跳过标题行，显示实际数据
            for line in lines.iter().skip(1).take(6) {
                if !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled(format!("💿 {}", parts[0]), Theme::accent()),
                        ])));
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled("   大小: ", Theme::muted()),
                            Span::styled(parts[1], Theme::secondary()),
                            Span::styled(" | 已用: ", Theme::muted()),
                            Span::styled(parts[2], Theme::success()),
                            Span::styled(" | 可用: ", Theme::muted()),
                            Span::styled(parts[3], Theme::success()),
                            Span::styled(" | 使用率: ", Theme::muted()),
                            Span::styled(parts[4], Theme::accent()),
                        ])));
                        items.push(ListItem::new(Line::from(vec![Span::raw("")])));
                    }
                }
            }
        } else {
            // 直接显示原始输出
            for line in lines.iter().take(8) {
                if !line.trim().is_empty() {
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(line.to_string(), Theme::secondary())
                    ])));
                }
            }
        }
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("⚠️  无法获取磁盘使用信息", Theme::success())
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 添加提示信息
    items.push(ListItem::new(Line::from(vec![
        Span::styled("💡 提示", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(30), Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 即使没有安装FIO或DD，程序也会使用内置Rust实现进行测试", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 测试将自动开始，请耐心等待结果", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 使用 ↑↓ 键可滚动查看详细信息", Theme::muted())
    ])));
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2);
    app.update_content_height(content_height, viewport_height);
    
    // 创建可见内容
    let visible_items = if content_height > viewport_height {
        items
            .iter()
            .skip(app.scroll_position.current as usize)
            .take(viewport_height as usize)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        items
    };

    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };

    let title = if is_focused {
        " 💾 磁盘测试准备 🎯 "
    } else {
        " 💾 磁盘测试准备 "
    };

    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style)
                .border_style(border_style)
                .padding(Padding::uniform(1)),
        );

    f.render_widget(list, area);
    
    // 绘制滚动条（如果需要）
    if content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}
