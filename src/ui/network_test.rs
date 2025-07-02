use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::{
    app::App,
    handlers::network_test::{get_current_test_info, NetworkTestInfo, TestStatus},
    ui::helpers::{create_block, get_progress_color},
};

pub fn draw_network_test_content(f: &mut Frame, _app: &mut App, area: Rect, is_focused: bool) {
    let test_info = get_current_test_info();
    
    if test_info.is_testing || !test_info.results.is_empty() {
        draw_test_interface(f, &test_info, area, is_focused);
    } else {
        draw_test_welcome(f, area, is_focused);
    }
}

fn draw_test_welcome(f: &mut Frame, area: Rect, is_focused: bool) {
    let block = create_block("网速测试", is_focused);
    
    let welcome_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🌐 ", Style::default().fg(Color::Cyan)),
            Span::styled("三网测速工具", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("支持运营商："),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::Green)),
            Span::styled("中国移动", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::Red)),
            Span::styled("中国联通", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::Blue)),
            Span::styled("中国电信", Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from("测试项目：延迟、下载速度、上传速度"),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 ", Style::default().fg(Color::Yellow)),
            Span::styled("进入此页面将自动开始测试", Style::default().fg(Color::White)),
        ]),
    ];
    
    let paragraph = Paragraph::new(welcome_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn draw_test_interface(f: &mut Frame, test_info: &NetworkTestInfo, area: Rect, is_focused: bool) {
    // 简单的上下布局：上面进度，下面结果
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // 进度区域
            Constraint::Min(8),     // 结果区域
        ])
        .split(area);
    
    // 绘制进度区域
    draw_progress_section(f, test_info, chunks[0], is_focused);
    
    // 绘制结果区域
    draw_results_section(f, test_info, chunks[1], is_focused);
}

fn draw_progress_section(f: &mut Frame, test_info: &NetworkTestInfo, area: Rect, is_focused: bool) {
    let block = create_block("测试进度", is_focused);
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 当前测试状态
            Constraint::Length(2),  // 进度条
            Constraint::Length(1),  // 当前速度信息
        ])
        .split(inner);
    
    // 显示当前测试状态
    let status_text = if test_info.is_testing {
        vec![Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::Cyan)),
            Span::styled(&test_info.current_stage, Style::default().fg(Color::White)),
        ])]
    } else if !test_info.results.is_empty() {
        vec![Line::from(vec![
            Span::styled("✅ ", Style::default().fg(Color::Green)),
            Span::styled("所有测试已完成", Style::default().fg(Color::Green)),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("⏸️  ", Style::default().fg(Color::Yellow)),
            Span::styled("等待开始测试...", Style::default().fg(Color::Yellow)),
        ])]
    };
    
    let status_paragraph = Paragraph::new(status_text);
    f.render_widget(status_paragraph, progress_chunks[0]);
    
    // 显示进度条
    let progress_ratio = test_info.progress / 100.0;
    let progress_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(get_progress_color(test_info.progress))
        .ratio(progress_ratio)
        .label(format!("{:.1}%", test_info.progress));
    f.render_widget(progress_gauge, progress_chunks[1]);
    
    // 显示当前速度信息
    let speed_text = if test_info.is_testing {
        if let Some(ref provider) = test_info.current_provider {
            if let Some(result) = test_info.results.get(provider) {
                let mut info_parts = Vec::new();
                
                if result.ping > 0.0 {
                    info_parts.push(format!("延迟: {:.1}ms", result.ping));
                }
                if result.download_speed > 0.0 {
                    info_parts.push(format!("下载: {:.1} Mbps", result.download_speed));
                }
                if result.upload_speed > 0.0 {
                    info_parts.push(format!("上传: {:.1} Mbps", result.upload_speed));
                }
                
                if info_parts.is_empty() {
                    "正在测试...".to_string()
                } else {
                    info_parts.join(" | ")
                }
            } else {
                "正在测试...".to_string()
            }
        } else {
            "初始化中...".to_string()
        }
    } else {
        "测试完成".to_string()
    };
    
    let speed_paragraph = Paragraph::new(speed_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(speed_paragraph, progress_chunks[2]);
}

fn draw_results_section(f: &mut Frame, test_info: &NetworkTestInfo, area: Rect, is_focused: bool) {
    let block = create_block("测试结果", is_focused);
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    if test_info.results.is_empty() {
        let waiting_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled("等待测试结果...", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("将依次测试：中国移动 → 中国联通 → 中国电信"),
        ];
        
        let paragraph = Paragraph::new(waiting_text)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, inner);
        return;
    }
    
    // 创建表格显示结果
    let header = Row::new(vec![
        "运营商", "状态", "延迟(ms)", "抖动(ms)", "下载(Mbps)", "上传(Mbps)", "评级"
    ]).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    
    let rows: Vec<Row> = vec!["中国移动", "中国联通", "中国电信"]
        .iter()
        .map(|&provider_name| {
            if let Some(result) = test_info.results.get(provider_name) {
                let status_style = match result.status {
                    TestStatus::NotStarted => Style::default().fg(Color::Gray),
                    TestStatus::Testing => Style::default().fg(Color::Yellow),
                    TestStatus::Completed => Style::default().fg(Color::Green),
                    TestStatus::Failed => Style::default().fg(Color::Red),
                };
                
                let status_text = match result.status {
                    TestStatus::NotStarted => "未开始",
                    TestStatus::Testing => "测试中",
                    TestStatus::Completed => "已完成",
                    TestStatus::Failed => "失败",
                };
                
                let ping_cell = if result.ping > 0.0 {
                    format!("{:.1}", result.ping)
                } else {
                    "-".to_string()
                };
                
                let jitter_cell = if result.jitter > 0.0 {
                    format!("{:.1}", result.jitter)
                } else {
                    "-".to_string()
                };
                
                let download_cell = if result.download_speed > 0.0 {
                    format!("{:.1}", result.download_speed)
                } else {
                    "-".to_string()
                };
                
                let upload_cell = if result.upload_speed > 0.0 {
                    format!("{:.1}", result.upload_speed)
                } else {
                    "-".to_string()
                };
                
                let rating = get_speed_rating(result.download_speed);
                
                Row::new(vec![
                    provider_name.to_string(),
                    status_text.to_string(),
                    ping_cell,
                    jitter_cell,
                    download_cell,
                    upload_cell,
                    rating.0,
                ]).style(status_style)
            } else {
                Row::new(vec![
                    provider_name.to_string(),
                    "未开始".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                ]).style(Style::default().fg(Color::Gray))
            }
        }).collect();
    
    let table = Table::new(rows, [
        Constraint::Length(8),   // 运营商
        Constraint::Length(8),   // 状态
        Constraint::Length(10),  // 延迟
        Constraint::Length(10),  // 抖动
        Constraint::Length(12),  // 下载
        Constraint::Length(12),  // 上传
        Constraint::Length(8),   // 评级
    ])
    .header(header)
    .column_spacing(1);
    
    f.render_widget(table, inner);
}

fn get_speed_rating(speed_mbps: f64) -> (String, Color) {
    if speed_mbps >= 100.0 {
        ("优秀".to_string(), Color::Green)
    } else if speed_mbps >= 50.0 {
        ("良好".to_string(), Color::Cyan)
    } else if speed_mbps >= 20.0 {
        ("一般".to_string(), Color::Yellow)
    } else if speed_mbps >= 5.0 {
        ("较差".to_string(), Color::Red)
    } else if speed_mbps > 0.0 {
        ("很差".to_string(), Color::Magenta)
    } else {
        ("-".to_string(), Color::Gray)
    }
}

fn draw_combined_test_interface(f: &mut Frame, area: Rect, test_info: &NetworkTestInfo, is_focused: bool) {
    // 创建上下布局，类似硬盘测试
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),  // 上部：实时测试状态
            Constraint::Min(8),      // 下部：测试结果
        ])
        .split(area);

    // 上部分：实时测试状态
    draw_realtime_test_status(f, main_chunks[0], test_info, is_focused);
    
    // 下部分：测试结果
    if !test_info.results.is_empty() {
        draw_test_results_table(f, main_chunks[1], test_info, is_focused);
    } else {
        draw_waiting_info(f, main_chunks[1], is_focused);
    }
}

fn draw_realtime_test_status(f: &mut Frame, area: Rect, test_info: &NetworkTestInfo, is_focused: bool) {
    let block = create_block("实时测试状态", is_focused);
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    // 创建内部布局
    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 当前阶段
            Constraint::Length(3),  // 总体进度条
            Constraint::Length(3),  // 当前测试进度条
            Constraint::Length(4),  // 实时速度显示
        ])
        .split(inner);
    
    // 当前阶段信息
    let stage_text = if let Some(ref provider) = test_info.current_provider {
        vec![Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("正在测试: {} - {}", provider, test_info.current_stage.split(" - ").last().unwrap_or("")),
                Style::default().fg(Color::White)
            ),
        ])]
    } else if test_info.is_testing {
        vec![Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::Cyan)),
            Span::styled(&test_info.current_stage, Style::default().fg(Color::White)),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("✅ ", Style::default().fg(Color::Green)),
            Span::styled("所有测试已完成", Style::default().fg(Color::Green)),
        ])]
    };
    
    let stage_paragraph = Paragraph::new(stage_text);
    f.render_widget(stage_paragraph, status_chunks[0]);
    
    // 总体进度条
    let overall_progress = test_info.overall_progress / 100.0;
    let overall_gauge = Gauge::default()
        .block(Block::default().title("总体进度").borders(Borders::ALL))
        .gauge_style(get_progress_color(test_info.overall_progress))
        .ratio(overall_progress)
        .label(format!("{:.1}%", test_info.overall_progress));
    f.render_widget(overall_gauge, status_chunks[1]);
    
    // 当前测试进度条
    let current_progress = test_info.progress / 100.0;
    let current_title = if let Some(ref provider) = test_info.current_provider {
        format!("{} 测试进度", provider)
    } else {
        "当前测试".to_string()
    };
    
    let current_gauge = Gauge::default()
        .block(Block::default().title(current_title).borders(Borders::ALL))
        .gauge_style(get_progress_color(test_info.progress))
        .ratio(current_progress)
        .label(format!("{:.1}%", test_info.progress));
    f.render_widget(current_gauge, status_chunks[2]);
    
    // 实时速度显示
    draw_realtime_speeds(f, status_chunks[3], test_info);
}

fn draw_realtime_speeds(f: &mut Frame, area: Rect, test_info: &NetworkTestInfo) {
    let block = Block::default()
        .title("实时测试数据")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    // 获取当前正在测试的运营商结果
    let current_result = if let Some(ref provider) = test_info.current_provider {
        test_info.results.get(provider)
    } else {
        None
    };
    
    let speed_text = if let Some(result) = current_result {
        vec![
            Line::from(vec![
                Span::styled("📡 延迟: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if result.ping > 0.0 { format!("{:.1}ms", result.ping) } else { "测试中...".to_string() },
                    Style::default().fg(Color::White)
                ),
                Span::styled("  抖动: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if result.jitter > 0.0 { format!("{:.1}ms", result.jitter) } else { "-".to_string() },
                    Style::default().fg(Color::White)
                ),
            ]),
            Line::from(vec![
                Span::styled("⬇️ 下载: ", Style::default().fg(Color::Green)),
                Span::styled(
                    if result.download_speed > 0.0 { format!("{:.1} Mbps", result.download_speed) } else { "测试中...".to_string() },
                    get_speed_style(result.download_speed)
                ),
                Span::styled("  ⬆️ 上传: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    if result.upload_speed > 0.0 { format!("{:.1} Mbps", result.upload_speed) } else { "测试中...".to_string() },
                    get_speed_style(result.upload_speed)
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("📡 延迟: ", Style::default().fg(Color::Yellow)),
                Span::styled("等待测试...", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("⬇️ 下载: ", Style::default().fg(Color::Green)),
                Span::styled("等待测试...", Style::default().fg(Color::Gray)),
                Span::styled("  ⬆️ 上传: ", Style::default().fg(Color::Cyan)),
                Span::styled("等待测试...", Style::default().fg(Color::Gray)),
            ]),
        ]
    };
    
    let paragraph = Paragraph::new(speed_text);
    f.render_widget(paragraph, inner);
}

fn get_speed_style(speed: f64) -> Style {
    if speed > 50.0 {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else if speed > 20.0 {
        Style::default().fg(Color::Yellow)
    } else if speed > 0.0 {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn draw_waiting_info(f: &mut Frame, area: Rect, is_focused: bool) {
    let block = create_block("等待测试", is_focused);
    
    let waiting_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled("准备开始三网测速...", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from("将依次测试："),
        Line::from(vec![
            Span::styled("1. ", Style::default().fg(Color::White)),
            Span::styled("中国移动", Style::default().fg(Color::Green)),
            Span::styled(" - 延迟、下载、上传", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("2. ", Style::default().fg(Color::White)),
            Span::styled("中国联通", Style::default().fg(Color::Red)),
            Span::styled(" - 延迟、下载、上传", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("3. ", Style::default().fg(Color::White)),
            Span::styled("中国电信", Style::default().fg(Color::Blue)),
            Span::styled(" - 延迟、下载、上传", Style::default().fg(Color::Gray)),
        ]),
    ];
    
    let paragraph = Paragraph::new(waiting_text)
        .block(block)
        .alignment(Alignment::Left);
    
    f.render_widget(paragraph, area);
}



fn draw_test_results_table(f: &mut Frame, area: Rect, test_info: &NetworkTestInfo, is_focused: bool) {
    let block = create_block("测试结果", is_focused);
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    // 创建表格显示结果
    let header = Row::new(vec![
        "运营商", "状态", "延迟", "抖动", "下载", "上传", "评级"
    ]).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    
    let rows: Vec<Row> = test_info.results.values().map(|result| {
        let status_cell = match result.status {
            TestStatus::NotStarted => "未开始",
            TestStatus::Testing => "测试中",
            TestStatus::Completed => "已完成",
            TestStatus::Failed => "失败",
        };
        
        let ping_cell = if result.ping > 0.0 {
            format!("{:.1}ms", result.ping)
        } else {
            "-".to_string()
        };
        
        let jitter_cell = if result.jitter > 0.0 {
            format!("{:.1}ms", result.jitter)
        } else {
            "-".to_string()
        };
        
        let download_cell = if result.download_speed > 0.0 {
            format!("{:.1} Mbps", result.download_speed)
        } else {
            "-".to_string()
        };
        
        let upload_cell = if result.upload_speed > 0.0 {
            format!("{:.1} Mbps", result.upload_speed)
        } else {
            "-".to_string()
        };
        
        let rating = get_speed_rating(result.download_speed);
        
        Row::new(vec![
            result.provider.clone(),
            status_cell.to_string(),
            ping_cell,
            jitter_cell,
            download_cell,
            upload_cell,
            rating.0,
        ])
    }).collect();
    
    let table = Table::new(rows, [
        Constraint::Length(8),   // 运营商
        Constraint::Length(8),   // 状态
        Constraint::Length(8),   // 延迟
        Constraint::Length(8),   // 抖动
        Constraint::Length(12),  // 下载
        Constraint::Length(12),  // 上传
        Constraint::Length(8),   // 评级
    ])
    .header(header)
    .column_spacing(1);
    
    f.render_widget(table, inner);
    
    // 如果有错误信息，显示在底部
    if let Some(ref error) = test_info.error_message {
        let error_area = Rect {
            x: inner.x,
            y: inner.bottom().saturating_sub(3),
            width: inner.width,
            height: 3,
        };
        
        let error_text = vec![
            Line::from(vec![
                Span::styled("❌ 错误: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ])
        ];
        
        let error_paragraph = Paragraph::new(error_text)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
        
        f.render_widget(Clear, error_area);
        f.render_widget(error_paragraph, error_area);
    }
}


