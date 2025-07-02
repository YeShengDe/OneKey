use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::{app::{App, FocusArea}, types::Result};

/// 处理键盘事件，返回 false 表示退出程序
pub fn handle_events(app: &mut App) -> Result<bool> {
    // 减少轮询间隔，实现更流畅的实时更新（特别是磁盘测试和CPU测试时）
    let poll_duration = match app.menu.selected_item() {
        crate::menu::MenuItem::DiskTest | 
        crate::menu::MenuItem::CpuTest | 
        crate::menu::MenuItem::NetworkSpeedTest => {
            Duration::from_millis(50)  // 测试时更频繁的更新
        },
        _ => Duration::from_millis(100) // 其他情况保持原有频率
    };
    
    if event::poll(poll_duration)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // 检查退出程序的快捷键
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('d') | KeyCode::Char('D') => return Ok(false), // Ctrl+D 退出程序
                        _ => {}
                    }
                }
                return handle_key_press(app, key);
            }
        }
    }
    Ok(true)
}

fn handle_key_press(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Ok(false),
        
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.toggle_menu();
            Ok(true)
        }
        
        // 数字键快速选择菜单项
        KeyCode::Char(n @ '0'..='9') => {
            if app.show_menu && app.focus_area == FocusArea::Menu {
                if app.menu.select_by_number(n) {
                    // 自动切换到内容区域并触发选择
                    app.set_focus(FocusArea::Content);
                    app.handle_menu_selection();
                }
            }
            Ok(true)
        }
        
        // 焦点切换
        KeyCode::Tab => {
            app.toggle_focus();
            Ok(true)
        }
        KeyCode::Left => {
            if app.show_menu {
                app.set_focus(FocusArea::Menu);
            }
            Ok(true)
        }
        KeyCode::Right => {
            if app.show_menu {
                app.set_focus(FocusArea::Content);
            }
            Ok(true)
        }
        
        // 上下键移动
        KeyCode::Up => {
            if app.focus_area == FocusArea::Menu && app.show_menu {
                app.menu.previous();
            } else {
                app.move_up();
            }
            Ok(true)
        }
        KeyCode::Down => {
            if app.focus_area == FocusArea::Menu && app.show_menu {
                app.menu.next();
            } else {
                app.move_down();
            }
            Ok(true)
        }
        
        // 翻页
        KeyCode::PageUp => {
            app.page_up();
            Ok(true)
        }
        KeyCode::PageDown => {
            app.page_down();
            Ok(true)
        }
        
        // 回车键 - 在菜单焦点时选择项目
        KeyCode::Enter => {
            app.handle_menu_selection();
            Ok(true)
        }
        
        // 快速导航
        KeyCode::Home => {
            app.reset_scroll();
            Ok(true)
        }
        KeyCode::End => {
            // 滚动到底部
            let max_scroll = app.scroll_position.max.saturating_sub(app.scroll_position.viewport_height);
            app.scroll_position.current = max_scroll;
            app.update_scrollbar();
            Ok(true)
        }
        
        _ => Ok(true),
    }
}