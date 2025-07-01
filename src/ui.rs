use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap, BarChart},
    Frame,
};

use crate::{app::{App, FocusArea}, theme::Theme};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();
    
    if app.show_menu {
        // 创建左右分栏布局
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)].as_ref())
            .split(size);
        
        // 绘制菜单
        let is_menu_focused = app.focus_area == FocusArea::Menu;
        draw_menu(f, app, chunks[0], is_menu_focused);
        
        // 绘制内容区域
        let is_content_focused = app.focus_area == FocusArea::Content;
        draw_content(f, app, chunks[1], is_content_focused);
    } else {
        // 菜单隐藏时，内容区域占据全屏
        draw_content(f, app, size, true); // 内容区域始终有焦点
    }
    
    // 绘制底部帮助栏
    draw_help_bar(f, app, size);
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let menu_items: Vec<ListItem> = app.menu.items()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.menu.selected_index() {
                Theme::list_selected()
            } else {
                Theme::list_unselected()
            };
            
            let content = Line::from(vec![
                Span::styled(format!(" {} ", item.as_str()), style),
            ]);
            
            ListItem::new(content)
        })
        .collect();
    
    // 根据焦点状态设置边框颜色和样式
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let title = if is_focused {
        " 功能菜单 [M] ● "
    } else {
        " 功能菜单 [M] "
    };
    
    let menu = List::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style)
                .title_alignment(Alignment::Center)
                .border_style(border_style),
        )
        .highlight_style(Theme::list_highlight());
    
    f.render_widget(menu, area);
}

fn draw_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 预留底部帮助栏的空间
    let content_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(1), // 减去底部帮助栏的高度
    };
    
    // 检查当前选中的菜单项，如果是系统信息或磁盘测试，使用特殊渲染
    let current_item = app.menu.selected_item();
    match current_item {
        crate::menu::MenuItem::SystemInfo => {
            draw_system_info_content(f, app, content_area, is_focused);
        },
        crate::menu::MenuItem::DiskTest => {
            draw_disk_test_content(f, app, content_area, is_focused);
        },
        _ => {
            draw_regular_content(f, app, content_area, is_focused);
        }
    }
}

fn draw_scrollbar(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    let thumb_style = if is_focused {
        Theme::scrollbar_focused()
    } else {
        Theme::scrollbar_unfocused()
    };
    
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .thumb_symbol("█")
        .thumb_style(thumb_style)
        .track_style(Theme::scrollbar_track());
    
    // 滚动条应该占据整个内容区域的右边，包括顶部和底部边框
    let scrollbar_area = Rect {
        x: area.right() - 1,
        y: area.top() + 1,  // 留出顶部边框
        width: 1,
        height: area.height.saturating_sub(2),  // 减去顶部和底部边框
    };
    
    f.render_stateful_widget(scrollbar, scrollbar_area, &mut app.scrollbar_state);
}

fn draw_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.show_menu {
        match app.focus_area {
            FocusArea::Menu => " Ctrl+D/Q 退出 │ ↑↓ 选择菜单 │ →/Tab 切换到内容 │ M 隐藏菜单 │ Enter 选择 ",
            FocusArea::Content => " Q 退出 │ ↑↓/PgUp/PgDn 滚动 │ ←/Tab 切换到菜单 │ M 隐藏菜单 │ Ctrl+C 复制 │ Ctrl+S 选择模式 ",
        }
    } else {
        " Q 退出 │ ↑↓/PgUp/PgDn 滚动内容 │ M 显示菜单 │ Ctrl+C 复制内容 │ Ctrl+S 文本选择模式 "
    };
    
    let help = Paragraph::new(help_text)
        .style(Theme::help_bar())
        .alignment(Alignment::Center);
    
    let help_area = Rect {
        x: area.x,
        y: area.bottom() - 1,
        width: area.width,
        height: 1,
    };
    
    f.render_widget(help, help_area);
}

fn draw_system_info_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取系统信息
    let system_info = crate::handlers::system_info::SystemInfo::get_current();
    
    // 创建系统信息项目列表
    let mut items = Vec::new();
    
    // 系统基本信息
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 系统信息 ━━━", Theme::primary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("主机名: ", Theme::accent()),
        Span::styled(system_info.basic.hostname.clone(), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("运行时间: ", Theme::accent()),
        Span::styled(system_info.basic.uptime.clone(), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("系统版本: ", Theme::accent()),
        Span::styled(system_info.basic.distro.clone(), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Linux版本: ", Theme::accent()),
        Span::styled(system_info.basic.kernel.clone(), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("系统时间: ", Theme::accent()),
        Span::styled(system_info.basic.system_time.clone(), Theme::secondary())
    ])));
    
    // CPU 信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ CPU信息 ━━━", Theme::primary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU架构: ", Theme::accent()),
        Span::styled(system_info.basic.cpu_arch.clone(), Theme::secondary())
    ])));
    
    let cpu_info = format!("{}", system_info.basic.cpu_model);
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU型号: ", Theme::accent()),
        Span::styled(cpu_info, Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU核心数: ", Theme::accent()),
        Span::styled(format!("{}", system_info.basic.cpu_cores), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU频率: ", Theme::accent()),
        Span::styled(system_info.basic.cpu_frequency.clone(), Theme::secondary())
    ])));
    
    // CPU使用率 - 分两行显示
    let cpu_usage_percent = system_info.basic.cpu_usage as f64;
    let cpu_usage_text = format!("{:.1}%", cpu_usage_percent);
    let cpu_bar = format!("[{}{}]", 
        "█".repeat((cpu_usage_percent / 5.0) as usize),
        "░".repeat(20 - (cpu_usage_percent / 5.0) as usize)
    );
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU占用: ", Theme::accent()),
        Span::styled(cpu_usage_text, Theme::gauge_style(cpu_usage_percent))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("         ", Theme::secondary()), // 对齐缩进
        Span::styled(cpu_bar, Theme::gauge_bar_style(cpu_usage_percent))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("系统负载: ", Theme::accent()),
        Span::styled(system_info.basic.load_avg.clone(), Theme::secondary())
    ])));
    
    // 内存信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 内存信息 ━━━", Theme::primary())
    ])));
    
    // 物理内存使用率 - 分两行显示
    let memory_percent = (system_info.basic.memory_used as f64 / system_info.basic.memory_total as f64) * 100.0;
    let memory_used_gb = system_info.basic.memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_total_gb = system_info.basic.memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_detail = format!("{:.1}/{:.1}G ({:.1}%)", memory_used_gb, memory_total_gb, memory_percent);
    let memory_bar = format!("[{}{}]", 
        "█".repeat((memory_percent / 5.0) as usize),
        "░".repeat(20 - (memory_percent / 5.0) as usize)
    );
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("物理内存: ", Theme::accent()),
        Span::styled(memory_detail, Theme::gauge_style(memory_percent))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("          ", Theme::secondary()), // 对齐缩进
        Span::styled(memory_bar, Theme::gauge_bar_style(memory_percent))
    ])));
    
    // 虚拟内存使用率 - 分两行显示
    if system_info.basic.swap_total > 0 {
        let swap_percent = (system_info.basic.swap_used as f64 / system_info.basic.swap_total as f64) * 100.0;
        let swap_used_gb = system_info.basic.swap_used as f64 / (1024.0 * 1024.0 * 1024.0);
        let swap_total_gb = system_info.basic.swap_total as f64 / (1024.0 * 1024.0 * 1024.0);
        let swap_detail = format!("{:.1}/{:.1}G ({:.1}%)", swap_used_gb, swap_total_gb, swap_percent);
        let swap_bar = format!("[{}{}]", 
            "█".repeat((swap_percent / 5.0) as usize),
            "░".repeat(20 - (swap_percent / 5.0) as usize)
        );
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("虚拟内存: ", Theme::accent()),
            Span::styled(swap_detail, Theme::gauge_style(swap_percent))
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("          ", Theme::secondary()), // 对齐缩进
            Span::styled(swap_bar, Theme::gauge_bar_style(swap_percent))
        ])));
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("虚拟内存: ", Theme::accent()),
            Span::styled("未配置", Theme::muted())
        ])));
    }
    
    // 磁盘信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 磁盘信息 ━━━", Theme::primary())
    ])));
    
    // 磁盘使用情况 - 分两行显示
    if !system_info.basic.disk_info.is_empty() {
        for disk in &system_info.basic.disk_info {
            let used_space = disk.total_space - disk.available_space;
            let disk_percent = if disk.total_space > 0 {
                (used_space as f64 / disk.total_space as f64) * 100.0
            } else {
                0.0
            };
            
            let used_gb = used_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let total_gb = disk.total_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let disk_detail = format!("{:.0}/{:.0}G ({:.0}%)", used_gb, total_gb, disk_percent);
            let disk_bar = format!("[{}{}]", 
                "█".repeat((disk_percent / 5.0) as usize),
                "░".repeat(20 - (disk_percent / 5.0) as usize)
            );
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("硬盘占用({}): ", disk.mount_point), Theme::accent()),
                Span::styled(disk_detail, Theme::gauge_style(disk_percent))
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled("          ", Theme::secondary()), // 对齐缩进
                Span::styled(disk_bar, Theme::gauge_bar_style(disk_percent))
            ])));
        }
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("硬盘占用: ", Theme::accent()),
            Span::styled("获取中...", Theme::muted())
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("虚拟化: ", Theme::accent()),
        Span::styled(system_info.basic.vm_type.clone(), Theme::secondary())
    ])));
    
    // 网络流量统计分隔符  
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 网络流量 ━━━", Theme::primary())
    ])));
    
    let rx_mb = system_info.basic.network_stats.rx_bytes as f64 / (1024.0 * 1024.0);
    let tx_mb = system_info.basic.network_stats.tx_bytes as f64 / (1024.0 * 1024.0);
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("总接收: ", Theme::accent()),
        Span::styled(format!("{:.2}M", rx_mb), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("总发送: ", Theme::accent()),
        Span::styled(format!("{:.2}M", tx_mb), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("网络算法: ", Theme::accent()),
        Span::styled(system_info.basic.network_algorithm.clone(), Theme::secondary())
    ])));
    
    // 网络信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 网络信息 ━━━", Theme::primary())
    ])));
    
    if system_info.network_loading {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("正在加载网络信息...", Theme::warning().add_modifier(Modifier::ITALIC))
        ])));
    } else {
        if let Some(ref isp) = system_info.network.isp {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("运营商: ", Theme::accent()),
                Span::styled(isp.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref ipv4) = system_info.network.ipv4 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("IPv4地址: ", Theme::accent()),
                Span::styled(ipv4.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref ipv6) = system_info.network.ipv6 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("IPv6: ", Theme::accent()),
                Span::styled(ipv6.clone(), Theme::secondary())
            ])));
        }
        
        if !system_info.basic.dns_servers.is_empty() && system_info.basic.dns_servers[0] != "Unknown" {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("DNS地址: ", Theme::accent()),
                Span::styled(system_info.basic.dns_servers.join(", "), Theme::secondary())
            ])));
        }
        
        if let Some(ref location) = system_info.network.location {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("地理位置: ", Theme::accent()),
                Span::styled(location.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref country) = system_info.network.country {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("国家: ", Theme::accent()),
                Span::styled(country.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref asn) = system_info.network.asn {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("ASN: ", Theme::accent()),
                Span::styled(asn.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref hostname) = system_info.network.hostname {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("主机: ", Theme::accent()),
                Span::styled(hostname.clone(), Theme::secondary())
            ])));
        }
    }
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2); // 减去边框
    app.update_content_height(content_height, viewport_height);
    
    // 创建可见内容 - 修复滚动逻辑，确保滚动条能正确显示位置
    let start_index = app.scroll_position.current as usize;
    let end_index = (start_index + viewport_height as usize).min(items.len());
    let visible_items = if start_index < items.len() {
        items[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };
    
    // 根据焦点状态设置边框颜色和样式
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let title = if is_focused {
        " 系统信息 ● "
    } else {
        " 系统信息 "
    };
    
    // 创建列表
    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style)
                .title_alignment(Alignment::Center)
                .border_style(border_style),
        );
    
    f.render_widget(list, area);
    
    // 如果需要，绘制滚动条
    if is_focused && content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}

fn draw_disk_test_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取磁盘测试信息
    let test_info = crate::handlers::disk_test::get_current_test_info();
    
    if !test_info.results.is_empty() {
        // 显示FIO风格的测试结果表格
        draw_disk_fio_style_results(f, app, area, &test_info.results, is_focused);
        return;
    }
    
    // 创建磁盘测试项目列表
    let mut items = Vec::new();
    
    // 标题
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 磁盘性能测试 ━━━", Theme::primary())
    ])));
    
    if test_info.is_testing {
        // 显示测试进度
        items.push(ListItem::new(Line::from(vec![
            Span::styled("状态: ", Theme::accent()),
            Span::styled(test_info.current_test.clone(), Theme::warning().add_modifier(Modifier::ITALIC))
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("进度: ", Theme::accent()),
            Span::styled(format!("{}%", test_info.progress), Theme::secondary())
        ])));
        
        // 进度条
        let progress_bar_length = 30;
        let filled = (test_info.progress as usize * progress_bar_length) / 100;
        let empty = progress_bar_length - filled;
        let progress_bar = format!("[{}{}]", 
            "█".repeat(filled),
            "░".repeat(empty)
        );
        items.push(ListItem::new(Line::from(vec![
            Span::styled(progress_bar, Theme::disk_test_progress_style(test_info.progress as f64))
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("正在执行磁盘性能测试，请稍候...", Theme::muted().add_modifier(Modifier::ITALIC))
        ])));
        
    } else if let Some(ref error) = test_info.error_message {
        // 显示错误信息
        items.push(ListItem::new(Line::from(vec![
            Span::styled("错误: ", Theme::accent()),
            Span::styled(error.clone(), Theme::warning())
        ])));
        
        // 工具检查状态
        items.push(ListItem::new(Line::from(vec![
            Span::styled("━━━ 工具状态 ━━━", Theme::primary())
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("FIO: ", Theme::accent()),
            Span::styled(
                if test_info.has_fio { "已安装" } else { "未安装" },
                if test_info.has_fio { Theme::success() } else { Theme::warning() }
            )
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("DD: ", Theme::accent()),
            Span::styled(
                if test_info.has_dd { "已安装" } else { "未安装" },
                if test_info.has_dd { Theme::success() } else { Theme::warning() }
            )
        ])));
        
        if !test_info.has_fio && !test_info.has_dd {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("建议: ", Theme::accent()),
                Span::styled("请安装 fio: apt-get install fio", Theme::muted())
            ])));
        }
        
    } else {
        // 准备测试状态
        items.push(ListItem::new(Line::from(vec![
            Span::styled("准备执行磁盘测试...", Theme::muted().add_modifier(Modifier::ITALIC))
        ])));
        
        // 工具检查状态
        items.push(ListItem::new(Line::from(vec![
            Span::styled("━━━ 检查测试工具 ━━━", Theme::primary())
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("FIO: ", Theme::accent()),
            Span::styled(
                if test_info.has_fio { "已安装 ✓" } else { "未安装 ✗" },
                if test_info.has_fio { Theme::success() } else { Theme::warning() }
            )
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("DD: ", Theme::accent()),
            Span::styled(
                if test_info.has_dd { "已安装 ✓" } else { "未安装 ✗" },
                if test_info.has_dd { Theme::success() } else { Theme::warning() }
            )
        ])));
    }
    
    // 添加磁盘使用情况信息
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 磁盘使用情况 ━━━", Theme::primary())
    ])));
    
    // 显示磁盘使用情况
    if !test_info.disk_usage.is_empty() {
        for line in test_info.disk_usage.lines() {
            if !line.trim().is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(line, Theme::secondary())
                ])));
            }
        }
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("正在获取磁盘使用情况...", Theme::muted().add_modifier(Modifier::ITALIC))
        ])));
    }
    
    // 添加磁盘设备信息
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 磁盘设备信息 ━━━", Theme::primary())
    ])));
    
    // 显示磁盘设备信息
    if !test_info.disk_info.is_empty() {
        for line in test_info.disk_info.lines() {
            if !line.trim().is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(line, Theme::secondary())
                ])));
            }
        }
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("正在获取磁盘设备信息...", Theme::muted().add_modifier(Modifier::ITALIC))
        ])));
    }
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2); // 减去边框
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
    
    // 根据焦点状态设置边框颜色和样式
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let title = if is_focused {
        " 磁盘测试 ● "
    } else {
        " 磁盘测试 "
    };
    
    // 创建列表
    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style)
                .title_alignment(Alignment::Center)
                .border_style(border_style),
        );
    
    f.render_widget(list, area);
    
    // 绘制滚动条（如果需要）
    if content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}

// 绘制FIO风格的磁盘测试结果
fn draw_disk_fio_style_results(f: &mut Frame, app: &mut App, area: Rect, results: &[crate::handlers::disk_test::DiskTestResult], is_focused: bool) {
    let mut items = Vec::new();
    
    // 标题
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 磁盘性能测试结果 ━━━", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // FIO风格表头
    items.push(ListItem::new(Line::from(vec![
        Span::styled("测试类型          ", Theme::accent().add_modifier(Modifier::BOLD)),
        Span::styled("读速度        ", Theme::accent().add_modifier(Modifier::BOLD)),
        Span::styled("写速度        ", Theme::accent().add_modifier(Modifier::BOLD)),
        Span::styled("读IOPS       ", Theme::accent().add_modifier(Modifier::BOLD)),
        Span::styled("写IOPS", Theme::accent().add_modifier(Modifier::BOLD))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("─────────────────┬─────────────┬─────────────┬─────────────┬─────────────", Theme::muted())
    ])));
    
    // 数据行
    for result in results.iter() {
        let test_name = if result.test_name.len() > 16 {
            format!("{}…", &result.test_name[..15])
        } else {
            format!("{:17}", result.test_name)
        };
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled(test_name, Theme::secondary()),
            Span::styled(format!("{:>12} ", result.read_speed), Theme::success()),
            Span::styled(format!("{:>12} ", result.write_speed), Theme::warning()),
            Span::styled(format!("{:>12} ", result.read_iops), Theme::success()),
            Span::styled(format!("{:>12}", result.write_iops), Theme::warning())
        ])));
        
        // 如果有总计数据，也显示
        if result.total_speed != "N/A" && !result.total_speed.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  └─ 总计        ", Theme::muted()),
                Span::styled(format!("{:>12} ", result.total_speed), Theme::accent()),
                Span::styled("           ", Theme::muted()),
                Span::styled(format!("{:>12} ", result.total_iops), Theme::accent()),
                Span::styled("           ", Theme::muted())
            ])));
        }
    }
    
    items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    // 添加说明
    items.push(ListItem::new(Line::from(vec![
        Span::styled("说明: ", Theme::accent()),
        Span::styled("速度单位 MB/s, IOPS为每秒输入/输出操作数", Theme::muted().add_modifier(Modifier::ITALIC))
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("提示: ", Theme::accent()),
        Span::styled("使用 ↑↓ 键可滚动查看更多结果", Theme::muted().add_modifier(Modifier::ITALIC))
    ])));
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2); // 减去边框
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
    
    // 根据焦点状态设置边框颜色和样式
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let title = if is_focused {
        " 磁盘测试结果 ● "
    } else {
        " 磁盘测试结果 "
    };
    
    // 创建列表（无额外边框，简洁显示）
    let list = List::new(visible_items)
        .block(
            Block::default()
                .borders(if is_focused { Borders::ALL } else { Borders::NONE })
                .title(title)
                .title_style(title_style)
                .title_alignment(Alignment::Center)
                .border_style(border_style),
        );
    
    f.render_widget(list, area);
    
    // 绘制滚动条（如果需要）
    if content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}

fn draw_regular_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取内容
    let content = app.get_content();
    let lines: Vec<&str> = content.lines().collect();
    
    // 更新滚动状态
    let content_height = lines.len() as u16;
    let viewport_height = area.height.saturating_sub(2); // 减去边框
    app.update_content_height(content_height, viewport_height);
    
    // 创建可见内容
    let visible_content = lines
        .iter()
        .skip(app.scroll_position.current as usize)
        .take(viewport_height as usize)
        .map(|line| Line::from(*line))
        .collect::<Vec<_>>();
    
    // 根据焦点状态设置边框颜色和样式
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let title = if is_focused {
        format!(" {} ● ", app.menu.selected_item())
    } else {
        format!(" {} ", app.menu.selected_item())
    };
    
    // 创建段落
    let paragraph = Paragraph::new(visible_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style)
                .title_alignment(Alignment::Center)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });
    
    f.render_widget(paragraph, area);
    
    // 绘制滚动条（如果需要）
    if content_height > viewport_height {
        draw_scrollbar(f, app, area, is_focused);
    }
}

// 创建带有文字的进度条
fn create_progress_bar_with_text(percentage: f64, bar_width: usize, _label: &str) -> String {
    let filled_chars = ((percentage / 100.0) * bar_width as f64) as usize;
    
    // 创建进度条文字
    let text = format!("{:.1}%", percentage);
    let text_len = text.len();
    
    // 计算文字在进度条中的位置（居中）
    let text_start = (bar_width.saturating_sub(text_len)) / 2;
    let text_end = text_start + text_len;
    
    let mut result = String::with_capacity(bar_width + 2);
    result.push('[');
    
    for i in 0..bar_width {
        if i >= text_start && i < text_end {
            // 在文字区域内
            let char_index = i - text_start;
            if let Some(ch) = text.chars().nth(char_index) {
                result.push(ch);
            } else {
                if i < filled_chars {
                    result.push('█');
                } else {
                    result.push('░');
                }
            }
        } else {
            // 在文字区域外
            if i < filled_chars {
                result.push('█');
            } else {
                result.push('░');
            }
        }
    }
    
    result.push(']');
    result
}

// 创建带有详细信息的进度条（如内存/磁盘）
fn create_detailed_progress_bar(used: f64, total: f64, unit: &str, bar_width: usize) -> String {
    let percentage = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };
    let filled_chars = ((percentage / 100.0) * bar_width as f64) as usize;
    
    // 创建进度条文字 - 显示用量/总量 (百分比)
    let text = format!("{:.1}/{:.1}{} {:.1}%", used, total, unit, percentage);
    let text_len = text.len();
    
    // 如果文字太长，只显示百分比
    let display_text = if text_len > bar_width.saturating_sub(2) {
        format!("{:.1}%", percentage)
    } else {
        text
    };
    
    let display_len = display_text.len();
    let text_start = (bar_width.saturating_sub(display_len)) / 2;
    let text_end = text_start + display_len;
    
    let mut result = String::with_capacity(bar_width + 2);
    result.push('[');
    
    for i in 0..bar_width {
        if i >= text_start && i < text_end {
            // 在文字区域内
            let char_index = i - text_start;
            if let Some(ch) = display_text.chars().nth(char_index) {
                result.push(ch);
            } else {
                if i < filled_chars {
                    result.push('█');
                } else {
                    result.push('░');
                }
            }
        } else {
            // 在文字区域外
            if i < filled_chars {
                result.push('█');
            } else {
                result.push('░');
            }
        }
    }
    
    result.push(']');
    result
}

// 绘制磁盘性能图表
fn draw_disk_performance_charts(f: &mut Frame, app: &mut App, area: Rect, results: &[crate::handlers::disk_test::DiskTestResult], is_focused: bool) {
    // 创建上下分栏布局：图表区域和详细数据区域
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // 标题
            Constraint::Length(12),     // 性能图表
            Constraint::Length(12),     // IOPS 图表
            Constraint::Min(0),         // 详细数据
        ].as_ref())
        .split(area);
    
    // 标题
    let title = Paragraph::new(Line::from(vec![
        Span::styled("━━━ 磁盘性能测试结果 ━━━", Theme::primary().add_modifier(Modifier::BOLD))
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(if is_focused { Borders::ALL } else { Borders::NONE }));
    f.render_widget(title, chunks[0]);
    
    // 准备性能数据 - 转换为BarChart需要的格式
    let mut speed_data: Vec<(&str, u64)> = Vec::new();
    let mut iops_data: Vec<(&str, u64)> = Vec::new();
    let mut speed_labels: Vec<String> = Vec::new();
    let mut iops_labels: Vec<String> = Vec::new();
    
    for (_i, result) in results.iter().enumerate() {
        // 解析速度数据 (MB/s)
        let _read_speed = parse_speed_value(&result.read_speed) as u64;
        let _write_speed = parse_speed_value(&result.write_speed) as u64;
        
        // 解析IOPS数据
        let _read_iops = parse_iops_value(&result.read_iops) as u64;
        let _write_iops = parse_iops_value(&result.write_iops) as u64;
        
        // 创建标签
        let read_label = format!("R-{}", truncate_test_name(&result.test_name, 6));
        let write_label = format!("W-{}", truncate_test_name(&result.test_name, 6));
        
        // 存储标签以保证生命周期
        speed_labels.push(read_label.clone());
        speed_labels.push(write_label.clone());
        iops_labels.push(read_label.clone());
        iops_labels.push(write_label.clone());
    }
    
    // 重新填充数据，使用引用
    for (i, result) in results.iter().enumerate() {
        let read_speed = parse_speed_value(&result.read_speed) as u64;
        let write_speed = parse_speed_value(&result.write_speed) as u64;
        let read_iops = parse_iops_value(&result.read_iops) as u64;
        let write_iops = parse_iops_value(&result.write_iops) as u64;
        
        let read_idx = i * 2;
        let write_idx = i * 2 + 1;
        
        if read_idx < speed_labels.len() && write_idx < speed_labels.len() {
            speed_data.push((&speed_labels[read_idx], read_speed));
            speed_data.push((&speed_labels[write_idx], write_speed));
            iops_data.push((&iops_labels[read_idx], read_iops));
            iops_data.push((&iops_labels[write_idx], write_iops));
        }
    }
    
    // 绘制速度图表
    if !speed_data.is_empty() {
        let max_speed = speed_data.iter().map(|(_, v)| *v as u64).max().unwrap_or(1);
        let speed_chart = BarChart::default()
            .block(Block::default()
                .title("磁盘读写速度 (MB/s)")
                .title_style(Theme::accent())
                .borders(Borders::ALL)
                .border_style(if is_focused { Theme::primary() } else { Theme::secondary() }))
            .data(&speed_data)
            .bar_width(4)
            .bar_gap(1)
            .bar_style(Theme::success())
            .value_style(Theme::secondary())
            .max(max_speed);
        f.render_widget(speed_chart, chunks[1]);
    }
    
    // 绘制IOPS图表
    if !iops_data.is_empty() {
        let max_iops = iops_data.iter().map(|(_, v)| *v as u64).max().unwrap_or(1);
        let iops_chart = BarChart::default()
            .block(Block::default()
                .title("磁盘读写IOPS")
                .title_style(Theme::accent())
                .borders(Borders::ALL)
                .border_style(if is_focused { Theme::primary() } else { Theme::secondary() }))
            .data(&iops_data)
            .bar_width(4)
            .bar_gap(1)
            .bar_style(Theme::warning())
            .value_style(Theme::secondary())
            .max(max_iops);
        f.render_widget(iops_chart, chunks[2]);
    }
    
    // 详细数据列表 - 优化格式和可读性
    let mut detail_items = Vec::new();
    
    detail_items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 详细测试数据 ━━━", Theme::primary().add_modifier(Modifier::BOLD))
    ])));
    detail_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    
    for (index, result) in results.iter().enumerate() {
        // 测试项标题 - 使用序号和清晰的分隔符
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}. ", index + 1), Theme::accent()),
            Span::styled(&result.test_name, Theme::accent().add_modifier(Modifier::BOLD))
        ])));
        
        // 创建美观的表格式数据显示
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   ┌─────────┬─────────────┬─────────────┐", Theme::muted())
        ])));
        
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   │", Theme::muted()),
            Span::styled(" 操作类型 ", Theme::accent().add_modifier(Modifier::BOLD)),
            Span::styled("│", Theme::muted()),
            Span::styled("    速度     ", Theme::accent().add_modifier(Modifier::BOLD)),
            Span::styled("│", Theme::muted()),
            Span::styled("    IOPS     ", Theme::accent().add_modifier(Modifier::BOLD)),
            Span::styled("│", Theme::muted()),
        ])));
        
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   ├─────────┼─────────────┼─────────────┤", Theme::muted())
        ])));
        
        // 读取性能数据行
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   │", Theme::muted()),
            Span::styled(" 读取     ", Theme::secondary()),
            Span::styled("│ ", Theme::muted()),
            Span::styled(format!("{:>11}", result.read_speed), Theme::success()),
            Span::styled(" │ ", Theme::muted()),
            Span::styled(format!("{:>11}", result.read_iops), Theme::success()),
            Span::styled(" │", Theme::muted()),
        ])));
        
        // 写入性能数据行
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   │", Theme::muted()),
            Span::styled(" 写入     ", Theme::secondary()),
            Span::styled("│ ", Theme::muted()),
            Span::styled(format!("{:>11}", result.write_speed), Theme::warning()),
            Span::styled(" │ ", Theme::muted()),
            Span::styled(format!("{:>11}", result.write_iops), Theme::warning()),
            Span::styled(" │", Theme::muted()),
        ])));
        
        // 总计性能数据行（如果有）
        if result.total_speed != "N/A" && !result.total_speed.is_empty() {
            detail_items.push(ListItem::new(Line::from(vec![
                Span::styled("   │", Theme::muted()),
                Span::styled(" 总计     ", Theme::secondary()),
                Span::styled("│ ", Theme::muted()),
                Span::styled(format!("{:>11}", result.total_speed), Theme::accent()),
                Span::styled(" │ ", Theme::muted()),
                Span::styled(format!("{:>11}", result.total_iops), Theme::accent()),
                Span::styled(" │", Theme::muted()),
            ])));
        }
        
        detail_items.push(ListItem::new(Line::from(vec![
            Span::styled("   └─────────┴─────────────┴─────────────┘", Theme::muted())
        ])));
        
        // 在每个测试项之间添加分隔空行
        if index < results.len() - 1 {
            detail_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        }
    }
    
    // 添加操作提示
    detail_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
    detail_items.push(ListItem::new(Line::from(vec![
        Span::styled("💡 ", Theme::accent()),
        Span::styled("使用 ↑↓ 键滚动查看详细数据", Theme::muted().add_modifier(Modifier::ITALIC))
    ])));
    
    // 更新详细数据区域的滚动状态
    let detail_content_height = detail_items.len() as u16;
    let detail_viewport_height = chunks[3].height.saturating_sub(2); // 减去边框
    
    // 更新滚动状态（仅针对详细数据区域）
    if detail_content_height > detail_viewport_height {
        // 有滚动需求时，更新滚动状态
        app.update_content_height(detail_content_height, detail_viewport_height);
        
        // 创建可见的详细数据项
        let visible_detail_items = detail_items
            .iter()
            .skip(app.scroll_position.current as usize)
            .take(detail_viewport_height as usize)
            .cloned()
            .collect::<Vec<_>>();
        
        // 绘制详细数据列表（带滚动）
        let detail_list = List::new(visible_detail_items)
            .block(Block::default()
                .title("详细数据 [↑↓滚动]")
                .title_style(Theme::accent())
                .borders(Borders::ALL)
                .border_style(if is_focused { Theme::primary() } else { Theme::secondary() }))
            .highlight_style(Theme::list_selected())
            .highlight_symbol("► ");
        
        f.render_widget(detail_list, chunks[3]);
        
        // 绘制滚动条
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        f.render_stateful_widget(
            scrollbar,
            chunks[3].inner(&ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut app.scrollbar_state,
        );
    } else {
        // 无滚动需求时，直接渲染所有内容
        let detail_list = List::new(detail_items)
            .block(Block::default()
                .title("详细数据")
                .title_style(Theme::accent())
                .borders(Borders::ALL)
                .border_style(if is_focused { Theme::primary() } else { Theme::secondary() }))
            .highlight_style(Theme::list_selected())
            .highlight_symbol("► ");
        
        f.render_widget(detail_list, chunks[3]);
    }
}

// 解析速度值（提取数字部分）
fn parse_speed_value(speed_str: &str) -> f64 {
    if speed_str == "N/A" || speed_str.is_empty() {
        return 0.0;
    }
    
    // 提取数字部分，支持格式如 "123.45 MB/s"
    let parts: Vec<&str> = speed_str.split_whitespace().collect();
    if let Some(first_part) = parts.first() {
        if let Ok(value) = first_part.parse::<f64>() {
            return value;
        }
    }
    
    // 尝试直接解析
    speed_str.chars()
        .take_while(|c| c.is_numeric() || *c == '.')
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}

// 解析IOPS值（提取数字部分）
fn parse_iops_value(iops_str: &str) -> f64 {
    if iops_str == "N/A" || iops_str.is_empty() {
        return 0.0;
    }
    
    // 提取数字部分，支持格式如 "1234 IOPS"
    let parts: Vec<&str> = iops_str.split_whitespace().collect();
    if let Some(first_part) = parts.first() {
        if let Ok(value) = first_part.parse::<f64>() {
            return value;
        }
    }
    
    // 尝试直接解析
    iops_str.chars()
        .take_while(|c| c.is_numeric() || *c == '.')
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}

// 截断测试名称以适应图表显示
fn truncate_test_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}