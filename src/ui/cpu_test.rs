use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Gauge, Padding, Paragraph},
    Frame,
};

use crate::{app::App, theme::Theme};
use super::components::draw_scrollbar;

/// 绘制CPU测试内容
pub fn draw_cpu_test_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取CPU测试信息
    let test_info = crate::handlers::cpu_test::get_current_test_info();
    
    if test_info.is_testing || !test_info.results.is_empty() {
        // 显示合并的测试和结果界面
        draw_combined_test_results_ui(f, app, area, &test_info, is_focused);
        return;
    }
    
    // 显示准备状态或错误信息的静态界面
    draw_cpu_test_static_content(f, app, area, &test_info, is_focused);
}

// 绘制合并的测试和结果界面
fn draw_combined_test_results_ui(f: &mut Frame, app: &mut App, area: Rect, test_info: &crate::handlers::cpu_test::CpuTestInfo, is_focused: bool) {
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
fn draw_realtime_test_status(f: &mut Frame, area: Rect, test_info: &crate::handlers::cpu_test::CpuTestInfo, is_focused: bool) {
    // 创建三列布局：当前测试(30%) | 进度条(40%) | 实时分数(30%)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // 左列：当前测试
            Constraint::Percentage(40), // 中列：进度条
            Constraint::Percentage(30), // 右列：实时分数
        ].as_ref())
        .split(area);

    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };

    // 左侧：当前测试阶段
    let test_phase_block = Block::default()
        .borders(Borders::ALL)
        .title(" 测试阶段 ")
        .title_style(title_style)
        .border_style(border_style);
    
    let test_inner = test_phase_block.inner(columns[0]);
    f.render_widget(test_phase_block, columns[0]);
    
    if test_inner.height > 0 {
        let test_content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 测试阶段
                Constraint::Length(1), // 阶段进度
            ].as_ref())
            .split(test_inner);
        
        // 测试阶段显示
        let phase_text = if test_info.is_testing {
            format!("🔬 {}", test_info.current_test_phase)
        } else {
            "🏁 测试完成".to_string()
        };
        
        f.render_widget(
            Paragraph::new(phase_text)
                .style(Theme::accent().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            test_content_area[0]
        );
        
        // 阶段进度显示
        if test_content_area.len() > 1 {
            let phase_progress_text = format!("{}/{}", 
                test_info.current_phase_index + 1, 
                test_info.total_test_phases
            );
            f.render_widget(
                Paragraph::new(phase_progress_text)
                    .style(Theme::muted())
                    .alignment(Alignment::Center),
                test_content_area[1]
            );
        }
    }

    // 中间：进度条
    let clamped_progress = test_info.progress.min(100);
    let progress_ratio = (clamped_progress as f64 / 100.0).max(0.0).min(1.0);
    
    // 创建上下布局用于进度条
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 总体进度
            Constraint::Length(3), // 测试计时
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

    // 测试计时器
    let elapsed_time = if let Some(start_time) = test_info.test_start_time {
        start_time.elapsed().as_secs()
    } else {
        0
    };
    
    let time_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 耗时 ")
                .title_style(title_style)
                .border_style(border_style)
        )
        .gauge_style(ratatui::style::Style::default().fg(Theme::chart_read_color()))
        .ratio(0.5) // 固定比例，仅用于显示
        .label(format!("{}s", elapsed_time));

    f.render_widget(time_gauge, progress_chunks[1]);

    // 右侧：实时分数
    let scores_block = Block::default()
        .borders(Borders::ALL)
        .title(" 实时评分 ")
        .title_style(title_style)
        .border_style(border_style);
    
    let scores_inner = scores_block.inner(columns[2]);
    f.render_widget(scores_block, columns[2]);
    
    if scores_inner.height > 0 {
        let scores_content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 单核分数
                Constraint::Length(1), // 多核分数
            ].as_ref())
            .split(scores_inner);
        
        // 单核分数
        let single_score_text = if test_info.estimated_single_core > 0 {
            format!("单核: {}", test_info.estimated_single_core)
        } else {
            "单核: --".to_string()
        };
        
        f.render_widget(
            Paragraph::new(single_score_text)
                .style(Theme::success().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            scores_content_area[0]
        );
        
        // 多核分数
        if scores_content_area.len() > 1 {
            let multi_score_text = if test_info.estimated_multi_core > 0 {
                format!("多核: {}", test_info.estimated_multi_core)
            } else {
                "多核: --".to_string()
            };
            
            f.render_widget(
                Paragraph::new(multi_score_text)
                    .style(Theme::warning().add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center),
                scores_content_area[1]
            );
        }
    }
}

// 绘制紧凑的测试结果（下半部分）
fn draw_compact_test_results(f: &mut Frame, app: &mut App, area: Rect, results: &[crate::handlers::cpu_test::CpuTestResult], is_focused: bool) {
    let mut items = Vec::new();
    
    // 查找综合评分
    let final_result = results.iter().find(|r| r.test_name == "综合评分");
    
    if let Some(final_score) = final_result {
        // 显示最终综合评分
        items.push(ListItem::new(Line::from(vec![
            Span::styled("🏆 综合评分", Theme::success().add_modifier(Modifier::BOLD))
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("┌─────────────────────────────────────────────────┐", Theme::accent())
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("│ ", Theme::accent()),
            Span::styled("单核心: ", Theme::primary()),
            Span::styled(format!("{:>6} 分", final_score.single_core_score), Theme::success().add_modifier(Modifier::BOLD)),
            Span::styled("  │  多核心: ", Theme::primary()),
            Span::styled(format!("{:>6} 分", final_score.multi_core_score), Theme::warning().add_modifier(Modifier::BOLD)),
            Span::styled(" │", Theme::accent()),
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("└─────────────────────────────────────────────────┘", Theme::accent())
        ])));
        
        items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    }
    
    // 详细测试结果
    items.push(ListItem::new(Line::from(vec![
        Span::styled("📊 详细测试结果", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(50), Theme::muted())
    ])));
    
    for result in results.iter() {
        if result.test_name != "综合评分" {
            // 测试项目名称
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("🔧 {}", result.test_name), Theme::accent())
            ])));
            
            // 性能数据
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   单核: ", Theme::muted()),
                Span::styled(format!("{:>5} 分", result.single_core_score), Theme::success()),
                Span::styled(" │ 多核: ", Theme::muted()),
                Span::styled(format!("{:>5} 分", result.multi_core_score), Theme::warning()),
                Span::styled(" │ 耗时: ", Theme::muted()),
                Span::styled(format!("{}ms", result.duration_ms), Theme::secondary()),
            ])));
            
            items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        }
    }
    
    // 性能对比参考
    if let Some(final_score) = final_result {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("📈 性能参考", Theme::primary().add_modifier(Modifier::BOLD))
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("─".repeat(50), Theme::muted())
        ])));
        
        let single_score = final_score.single_core_score;
        let multi_score = final_score.multi_core_score;
        
        let single_rating = match single_score {
            0..=800 => ("入门级", Theme::error()),
            801..=1200 => ("中等", Theme::warning()),
            1201..=1600 => ("良好", Theme::success()),
            1601..=2000 => ("优秀", Theme::accent()),
            _ => ("顶级", Theme::primary()),
        };
        
        let multi_rating = match multi_score {
            0..=3000 => ("入门级", Theme::error()),
            3001..=6000 => ("中等", Theme::warning()),
            6001..=9000 => ("良好", Theme::success()),
            9001..=12000 => ("优秀", Theme::accent()),
            _ => ("顶级", Theme::primary()),
        };
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("单核性能等级: ", Theme::muted()),
            Span::styled(single_rating.0, single_rating.1.add_modifier(Modifier::BOLD)),
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("多核性能等级: ", Theme::muted()),
            Span::styled(multi_rating.0, multi_rating.1.add_modifier(Modifier::BOLD)),
        ])));
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

// 绘制测试进行中的信息
fn draw_testing_progress_info(f: &mut Frame, area: Rect, test_info: &crate::handlers::cpu_test::CpuTestInfo, is_focused: bool) {
    let elapsed_time = if let Some(start_time) = test_info.test_start_time {
        start_time.elapsed().as_secs()
    } else {
        0
    };

    // 动画效果
    let animation_chars = ["⚡", "🔥", "💪", "🚀", "⭐", "🎯", "🏃", "💨"];
    let animation_char = animation_chars[test_info.animation_frame % animation_chars.len()];

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🔬 正在进行CPU性能测试", Theme::primary().add_modifier(Modifier::BOLD))
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(animation_char, Theme::spinning_animation()),
            Span::styled(format!(" {}", test_info.current_test_phase), Theme::secondary())
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⏱️  运行时间: ", Theme::accent()),
            Span::styled(format!("{}秒", elapsed_time), Theme::secondary()),
            Span::styled(" | 阶段: ", Theme::accent()),
            Span::styled(format!("{}/{}", test_info.current_phase_index + 1, test_info.total_test_phases), Theme::secondary())
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("💡 正在测试CPU的各项性能指标...", Theme::muted())
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
fn draw_cpu_test_static_content(f: &mut Frame, app: &mut App, area: Rect, test_info: &crate::handlers::cpu_test::CpuTestInfo, is_focused: bool) {
    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🔬 Geekbench 风格 CPU 性能测试", Theme::primary().add_modifier(Modifier::BOLD))
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
            Span::styled("🚀 准备执行CPU性能测试...", Theme::accent().add_modifier(Modifier::ITALIC))
        ])));
        items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    }
    
    // CPU信息显示
    items.push(ListItem::new(Line::from(vec![
        Span::styled("💻 系统CPU信息", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(40), Theme::muted())
    ])));
    
    // 解析CPU信息
    let cpu_info_lines: Vec<&str> = test_info.cpu_info.lines().collect();
    for line in &cpu_info_lines {
        if !line.trim().is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(line.to_string(), Theme::secondary())
            ])));
        }
    }
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 测试项目说明
    items.push(ListItem::new(Line::from(vec![
        Span::styled("📋 测试项目", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(40), Theme::muted())
    ])));
    
    let test_items = [
        ("🔢", "整数运算", "斐波那契数列、质数计算"),
        ("🧮", "浮点运算", "三角函数、开方运算"),
        ("⚡", "矢量运算", "SIMD操作模拟"),
        ("🔐", "加密算法", "哈希计算性能"),
        ("📦", "压缩算法", "数据压缩性能"),
        ("💾", "内存带宽", "内存访问速度"),
        ("🎯", "综合测试", "混合负载性能"),
    ];
    
    for (icon, name, desc) in &test_items {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} {}: ", icon, name), Theme::accent()),
            Span::styled(desc.to_string(), Theme::muted())
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 添加提示信息
    items.push(ListItem::new(Line::from(vec![
        Span::styled("💡 测试说明", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(40), Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 测试将模拟Geekbench的评分算法", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 单核测试评估单线程性能", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 多核测试评估并行计算能力", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 测试期间CPU使用率会达到100%", Theme::muted())
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("• 预计耗时30-60秒，请耐心等待", Theme::muted())
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
        " 🔬 CPU 性能测试 🎯 "
    } else {
        " 🔬 CPU 性能测试 "
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
