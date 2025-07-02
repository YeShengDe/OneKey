mod app;
mod events;
mod menu;
mod types;
mod ui;
mod handlers;
mod utils;
mod theme;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::{app::App, events::handle_events, types::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化终端
    let mut terminal = init_terminal()?;
    
    // 创建应用实例
    let mut app = App::new();
    
    // 运行应用
    let res = run_app(&mut terminal, &mut app).await;
    
    // 恢复终端
    restore_terminal(&mut terminal)?;
    
    // 处理错误
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }
    
    Ok(())
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_refresh = std::time::Instant::now();
    let refresh_interval = std::time::Duration::from_millis(100); // 100ms刷新间隔
    
    loop {
        let now = std::time::Instant::now();
        let should_refresh = now.duration_since(last_refresh) >= refresh_interval;
        
        if should_refresh {
            // 检查系统信息是否需要刷新
            if handlers::system_info::check_needs_refresh() {
                app.clear_cache(); // 清除缓存以强制重新获取内容
            }
            
            // 检查磁盘测试是否需要刷新
            if handlers::disk_test::check_needs_refresh() {
                app.clear_cache(); // 清除缓存以强制重新获取内容
                app.needs_refresh = true; // 标记需要UI刷新
            }
            
            // 检查CPU测试是否需要刷新
            if handlers::cpu_test::check_needs_refresh() {
                app.clear_cache(); // 清除缓存以强制重新获取内容
                app.needs_refresh = true; // 标记需要UI刷新
            }
            
            // 检查网速测试是否需要刷新
            if handlers::network_test::check_needs_refresh() {
                app.clear_cache(); // 清除缓存以强制重新获取内容
                app.needs_refresh = true; // 标记需要UI刷新
            }
            
            // 如果是磁盘测试界面且在测试中，更新动画帧
            if let crate::menu::MenuItem::DiskTest = app.menu.selected_item() {
                handlers::disk_test::update_animation_frame();
            }
            
            // 如果是CPU测试界面且在测试中，更新动画帧
            if let crate::menu::MenuItem::CpuTest = app.menu.selected_item() {
                handlers::cpu_test::update_animation_frame();
            }
            
            last_refresh = now;
        }
        
        // 绘制UI (每次循环都绘制以确保响应性)
        terminal.draw(|f| ui::draw(f, app))?;
        
        // 处理事件 (非阻塞或超时很短)
        if !handle_events(app)? {
            break;
        }
        
        // 短暂休眠以避免CPU占用过高
        tokio::time::sleep(std::time::Duration::from_millis(16)).await; // ~60fps
    }
    Ok(())
}