pub mod components;
pub mod system_info;
pub mod disk_test;
pub mod cpu_test;
pub mod network_test;
pub mod helpers;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{app::{App, FocusArea}, theme::Theme};
use components::{draw_menu, draw_scrollbar};
use system_info::draw_system_info_content;
use disk_test::draw_disk_test_content;
use cpu_test::draw_cpu_test_content;
use network_test::draw_network_test_content;

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
        crate::menu::MenuItem::CpuTest => {
            // 检查是否需要自动启动CPU测试
            let test_info = crate::handlers::cpu_test::get_current_test_info();
            if !test_info.is_testing && test_info.results.is_empty() && test_info.error_message.is_none() {
                // 自动启动CPU测试
                crate::handlers::cpu_test::start_cpu_test();
            }
            draw_cpu_test_content(f, app, content_area, is_focused);
        },
        crate::menu::MenuItem::NetworkSpeedTest => {
            // 检查是否需要自动启动网速测试
            let test_info = crate::handlers::network_test::get_current_test_info();
            if !test_info.is_testing && test_info.results.is_empty() && test_info.error_message.is_none() {
                // 自动启动网速测试
                crate::handlers::network_test::start_network_test();
            }
            draw_network_test_content(f, app, content_area, is_focused);
        },
        _ => {
            draw_regular_content(f, app, content_area, is_focused);
        }
    }
}

fn draw_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.show_menu {
        match app.focus_area {
            FocusArea::Menu => " Ctrl+D/Q 退出 │ ↑↓ 选择菜单 │ →/Tab 切换到内容 │ M 隐藏菜单 │ Enter 选择 ",
            FocusArea::Content => " Q 退出 │ ↑↓/PgUp/PgDn 滚动 │ ←/Tab 切换到菜单 │ M 隐藏菜单 ",
        }
    } else {
        " Q 退出 │ ↑↓/PgUp/PgDn 滚动内容 │ M 显示菜单 "
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

// 通用内容显示函数
fn draw_regular_content(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
    let content = app.get_content();
    
    // 将内容按行分割
    let lines: Vec<&str> = content.lines().collect();
    let items: Vec<ListItem> = lines
        .iter()
        .map(|line| {
            if line.starts_with("━━━") {
                // 这是一个标题行
                ListItem::new(Line::from(vec![
                    Span::styled(*line, Theme::primary())
                ]))
            } else if line.contains(": ") {
                // 这是一个键值对行
                let parts: Vec<&str> = line.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{}: ", parts[0]), Theme::accent()),
                        Span::styled(parts[1], Theme::secondary())
                    ]))
                } else {
                    ListItem::new(Line::from(vec![
                        Span::styled(*line, Theme::secondary())
                    ]))
                }
            } else {
                // 普通文本行
                ListItem::new(Line::from(vec![
                    Span::styled(*line, Theme::secondary())
                ]))
            }
        })
        .collect();
    
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
        " 内容 ● "
    } else {
        " 内容 "
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
