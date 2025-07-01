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

fn main() -> Result<()> {
    // 初始化终端
    let mut terminal = init_terminal()?;
    
    // 创建应用实例
    let mut app = App::new();
    
    // 运行应用
    let res = run_app(&mut terminal, &mut app);
    
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // 检查系统信息是否需要刷新
        if handlers::system_info::check_needs_refresh() {
            app.clear_cache(); // 清除缓存以强制重新获取内容
        }
        
        // 绘制UI
        terminal.draw(|f| ui::draw(f, app))?;
        
        // 处理事件
        if !handle_events(app)? {
            break;
        }
    }
    Ok(())
}