use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::{app::{App, FocusArea}, types::Result};

/// 处理键盘事件，返回 false 表示退出程序
pub fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // 检查 Ctrl+D 退出程序
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('d') => return Ok(false),
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
        
        // Ctrl+C 复制内容到剪贴板
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let _ = app.copy_content();
                Ok(true)
            } else {
                Ok(true)
            }
        }
        
        // Ctrl+S 切换选择模式 (禁用/启用鼠标捕获)
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.toggle_selection_mode();
                if app.selection_mode {
                    let _ = App::enable_text_selection();
                } else {
                    let _ = App::disable_text_selection();
                }
                Ok(true)
            } else {
                Ok(true)
            }
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
            app.move_up();
            Ok(true)
        }
        KeyCode::Down => {
            app.move_down();
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