use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Gauge, Paragraph},
    Frame,
};

use crate::{app::App, theme::Theme};
use super::components::draw_scrollbar;

/// 绘制系统信息内容
pub fn draw_system_info_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    // 获取系统信息
    let system_info = crate::handlers::system_info::SystemInfo::get_current();
    
    // 创建左右分栏布局：左侧基本信息，右侧系统状态
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // 左侧：基本信息
            Constraint::Percentage(40), // 右侧：系统状态
        ])
        .split(area);
    
    // 绘制左侧：基本系统信息
    draw_basic_system_info(f, app, main_chunks[0], &system_info, is_focused);
    
    // 绘制右侧：系统状态监控
    draw_system_status_panel(f, main_chunks[1], &system_info, is_focused);
}/// 绘制基本系统信息（左侧面板）
fn draw_basic_system_info(f: &mut Frame, app: &mut App, area: Rect, system_info: &crate::handlers::system_info::SystemInfo, is_focused: bool) {
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
        Span::styled("系统时间: ", Theme::accent()),
        Span::styled(system_info.basic.system_time.clone(), Theme::secondary())
    ])));
    
    // 系统版本可能很长，需要换行处理
    let distro = &system_info.basic.distro;
    if distro.len() > 25 {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("系统版本: ", Theme::accent())
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  ", Theme::muted()),
            Span::styled(distro.clone(), Theme::secondary())
        ])));
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("系统版本: ", Theme::accent()),
            Span::styled(distro.clone(), Theme::secondary())
        ])));
    }
    
    // Linux版本可能很长，需要换行处理
    let kernel = &system_info.basic.kernel;
    if kernel.len() > 25 {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("Linux版本: ", Theme::accent())
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  ", Theme::muted()),
            Span::styled(kernel.clone(), Theme::secondary())
        ])));
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("Linux版本: ", Theme::accent()),
            Span::styled(kernel.clone(), Theme::secondary())
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("虚拟化: ", Theme::accent()),
        Span::styled(system_info.basic.vm_type.clone(), Theme::secondary())
    ])));
    
    // CPU 信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ CPU信息 ━━━", Theme::primary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU架构: ", Theme::accent()),
        Span::styled(system_info.basic.cpu_arch.clone(), Theme::secondary())
    ])));
    
    // CPU型号可能很长或为空，需要特殊处理
    let cpu_model = &system_info.basic.cpu_model;
    if cpu_model.is_empty() || cpu_model == "Unknown" {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("CPU型号: ", Theme::accent()),
            Span::styled("未知", Theme::muted())
        ])));
    } else if cpu_model.len() > 20 {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("CPU型号: ", Theme::accent())
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  ", Theme::muted()),
            Span::styled(cpu_model.clone(), Theme::secondary())
        ])));
    } else {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("CPU型号: ", Theme::accent()),
            Span::styled(cpu_model.clone(), Theme::secondary())
        ])));
    }
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU核心数: ", Theme::accent()),
        Span::styled(format!("{}", system_info.basic.cpu_cores), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("CPU频率: ", Theme::accent()),
        Span::styled(system_info.basic.cpu_frequency.clone(), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("系统负载: ", Theme::accent()),
        Span::styled(system_info.basic.load_avg.clone(), Theme::secondary())
    ])));
    
    // 网络信息分隔符
    items.push(ListItem::new(Line::from(vec![
        Span::styled("━━━ 网络信息 ━━━", Theme::primary())
    ])));
    
    let rx_mb = system_info.basic.network_stats.rx_bytes as f64 / (1024.0 * 1024.0);
    let tx_mb = system_info.basic.network_stats.tx_bytes as f64 / (1024.0 * 1024.0);
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("总接收: ", Theme::accent()),
        Span::styled(format!("{:.1}MB", rx_mb), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("总发送: ", Theme::accent()),
        Span::styled(format!("{:.1}MB", tx_mb), Theme::secondary())
    ])));
    
    items.push(ListItem::new(Line::from(vec![
        Span::styled("TCP算法: ", Theme::accent()),
        Span::styled(system_info.basic.network_algorithm.clone(), Theme::secondary())
    ])));
    
    if system_info.network_loading {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("正在获取网络信息...", Theme::warning().add_modifier(Modifier::ITALIC))
        ])));
    } else {
        if let Some(ref ipv4) = system_info.network.ipv4 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("IPv4: ", Theme::accent()),
                Span::styled(ipv4.clone(), Theme::secondary())
            ])));
        }
        
        if let Some(ref ipv6) = system_info.network.ipv6 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("IPv6: ", Theme::accent()),
                Span::styled(ipv6.clone(), Theme::secondary())
            ])));
        }
        
        // 运营商信息可能很长，需要换行处理
        if let Some(ref isp) = system_info.network.isp {
            if isp.len() > 20 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("运营商: ", Theme::accent())
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Theme::muted()),
                    Span::styled(isp.clone(), Theme::secondary())
                ])));
            } else {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("运营商: ", Theme::accent()),
                    Span::styled(isp.clone(), Theme::secondary())
                ])));
            }
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
        
        // DNS服务器信息
        if !system_info.basic.dns_servers.is_empty() && system_info.basic.dns_servers[0] != "Unknown" {
            let dns_list = system_info.basic.dns_servers.join(", ");
            if dns_list.len() > 20 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("DNS: ", Theme::accent())
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Theme::muted()),
                    Span::styled(dns_list, Theme::secondary())
                ])));
            } else {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("DNS: ", Theme::accent()),
                    Span::styled(dns_list, Theme::secondary())
                ])));
            }
        }
    }
    
    // 更新滚动状态
    let content_height = items.len() as u16;
    let viewport_height = area.height.saturating_sub(2); // 减去边框
    app.update_content_height(content_height, viewport_height);
    
    // 创建可见内容
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
}/// 绘制系统状态监控面板（右侧面板）
fn draw_system_status_panel(f: &mut Frame, area: Rect, system_info: &crate::handlers::system_info::SystemInfo, is_focused: bool) {
    // 创建上下布局：性能监控 + 存储信息
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // 性能监控区域
            Constraint::Min(6),     // 存储信息区域
        ])
        .split(area);
    
    // 绘制性能监控区域
    draw_performance_gauges(f, chunks[0], system_info, is_focused);
    
    // 绘制存储信息区域
    draw_storage_info(f, chunks[1], system_info, is_focused);
}

/// 绘制性能监控区域
fn draw_performance_gauges(f: &mut Frame, area: Rect, system_info: &crate::handlers::system_info::SystemInfo, is_focused: bool) {
    // 创建约束：CPU占用 + 内存占用 + 虚拟内存（如果有）
    let mut constraints = vec![
        Constraint::Length(3), // CPU占用
        Constraint::Length(3), // 内存占用
    ];
    
    if system_info.basic.swap_total > 0 {
        constraints.push(Constraint::Length(3)); // 虚拟内存
    }
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    let mut chunk_index = 0;
    
    // CPU使用率Gauge
    let cpu_usage_percent = system_info.basic.cpu_usage as f64;
    let cpu_ratio = (cpu_usage_percent / 100.0).max(0.0).min(1.0);
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CPU占用 ")
                .title_style(title_style)
                .border_style(border_style)
        )
        .gauge_style(Theme::gauge_bar_style(cpu_usage_percent))
        .ratio(cpu_ratio)
        .label(format!("{:.1}%", cpu_usage_percent));
    
    f.render_widget(cpu_gauge, chunks[chunk_index]);
    chunk_index += 1;
    
    // 内存使用率Gauge
    let memory_percent = (system_info.basic.memory_used as f64 / system_info.basic.memory_total as f64) * 100.0;
    let memory_used_gb = system_info.basic.memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_total_gb = system_info.basic.memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_ratio = (memory_percent / 100.0).max(0.0).min(1.0);
    let memory_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 物理内存 ")
                .title_style(title_style)
                .border_style(border_style)
        )
        .gauge_style(Theme::gauge_bar_style(memory_percent))
        .ratio(memory_ratio)
        .label(format!("{:.1}/{:.1}G ({:.1}%)", memory_used_gb, memory_total_gb, memory_percent));
    
    f.render_widget(memory_gauge, chunks[chunk_index]);
    chunk_index += 1;
    
    // 虚拟内存Gauge（如果有）
    if system_info.basic.swap_total > 0 && chunk_index < chunks.len() {
        let swap_percent = (system_info.basic.swap_used as f64 / system_info.basic.swap_total as f64) * 100.0;
        let swap_used_gb = system_info.basic.swap_used as f64 / (1024.0 * 1024.0 * 1024.0);
        let swap_total_gb = system_info.basic.swap_total as f64 / (1024.0 * 1024.0 * 1024.0);
        let swap_ratio = (swap_percent / 100.0).max(0.0).min(1.0);
        let swap_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 虚拟内存 ")
                    .title_style(title_style)
                    .border_style(border_style)
            )
            .gauge_style(Theme::gauge_bar_style(swap_percent))
            .ratio(swap_ratio)
            .label(format!("{:.1}/{:.1}G ({:.1}%)", swap_used_gb, swap_total_gb, swap_percent));
        
        f.render_widget(swap_gauge, chunks[chunk_index]);
    }
}

/// 绘制存储信息区域
fn draw_storage_info(f: &mut Frame, area: Rect, system_info: &crate::handlers::system_info::SystemInfo, is_focused: bool) {
    if system_info.basic.disk_info.is_empty() {
        // 如果没有磁盘信息，显示提示
        let (border_style, title_style) = if is_focused {
            (Theme::border_focused(), Theme::title_focused())
        } else {
            (Theme::border_unfocused(), Theme::title_unfocused())
        };
        
        let placeholder = Paragraph::new("正在获取存储信息...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 存储状态 ")
                    .title_style(title_style)
                    .border_style(border_style)
            )
            .alignment(Alignment::Center);
        
        f.render_widget(placeholder, area);
        return;
    }
    
    // 为每个磁盘创建约束
    let mut constraints = Vec::new();
    for _ in &system_info.basic.disk_info {
        constraints.push(Constraint::Length(3));
    }
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    let (border_style, title_style) = if is_focused {
        (Theme::border_focused(), Theme::title_focused())
    } else {
        (Theme::border_unfocused(), Theme::title_unfocused())
    };
    
    // 磁盘使用率Gauge
    for (index, disk) in system_info.basic.disk_info.iter().enumerate() {
        if index >= chunks.len() {
            break;
        }
        
        let used_space = disk.total_space - disk.available_space;
        let disk_percent = if disk.total_space > 0 {
            (used_space as f64 / disk.total_space as f64) * 100.0
        } else {
            0.0
        };
        
        let used_gb = used_space as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = disk.total_space as f64 / (1024.0 * 1024.0 * 1024.0);
        let disk_ratio = (disk_percent / 100.0).max(0.0).min(1.0);
        
        // 优化磁盘标题显示
        let disk_title = if disk.mount_point == "/" {
            " 系统盘 "
        } else if disk.mount_point.contains("vscode") {
            " VSCode "
        } else if disk.mount_point.contains("workspaces") {
            " 工作空间 "
        } else {
            " 磁盘 "
        };
        
        let disk_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(disk_title)
                    .title_style(title_style)
                    .border_style(border_style)
            )
            .gauge_style(Theme::gauge_bar_style(disk_percent))
            .ratio(disk_ratio)
            .label(format!("{:.0}/{:.0}G ({:.0}%)", used_gb, total_gb, disk_percent));
        
        f.render_widget(disk_gauge, chunks[index]);
    }
}