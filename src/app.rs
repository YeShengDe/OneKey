use ratatui::widgets::ScrollbarState;
use crate::{
    handlers,
    menu::{Menu, MenuItem},
    types::ScrollPosition,
};

// 定义焦点区域枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusArea {
    Menu,
    Content,
}

pub struct App {
    pub menu: Menu,
    pub show_menu: bool,
    pub focus_area: FocusArea,  // 新增：当前焦点区域
    pub scroll_position: ScrollPosition,
    pub scrollbar_state: ScrollbarState,
    pub needs_refresh: bool,  // 新增：是否需要刷新UI
    pub selection_mode: bool,  // 新增：是否在文本选择模式
    content_cache: Option<(MenuItem, String)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(),
            show_menu: true,
            focus_area: FocusArea::Menu,  // 默认焦点在菜单
            scroll_position: ScrollPosition::new(),
            scrollbar_state: ScrollbarState::default(),
            needs_refresh: false,  // 初始化为不需要刷新
            selection_mode: false,  // 初始化为不在选择模式
            content_cache: None,
        }
    }
    
    pub fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
        if self.show_menu {
            self.focus_area = FocusArea::Menu;  // 显示菜单时焦点回到菜单
        } else {
            self.focus_area = FocusArea::Content;  // 隐藏菜单时焦点到内容
        }
    }
    
    // 新增：切换焦点区域
    pub fn toggle_focus(&mut self) {
        if self.show_menu {
            self.focus_area = match self.focus_area {
                FocusArea::Menu => FocusArea::Content,
                FocusArea::Content => FocusArea::Menu,
            };
        }
        // 如果菜单隐藏，焦点始终在内容区域
    }
    
    // 新增：设置焦点到特定区域
    pub fn set_focus(&mut self, area: FocusArea) {
        if self.show_menu || area == FocusArea::Content {
            self.focus_area = area;
        }
    }
    
    // 修改：根据焦点区域执行上移操作
    pub fn move_up(&mut self) {
        match self.focus_area {
            FocusArea::Menu if self.show_menu => {
                self.menu.previous();
                self.reset_scroll();
                self.clear_cache();
            }
            FocusArea::Content => {
                self.scroll_up(1);
            }
            _ => {}
        }
    }
    
    // 修改：根据焦点区域执行下移操作
    pub fn move_down(&mut self) {
        match self.focus_area {
            FocusArea::Menu if self.show_menu => {
                self.menu.next();
                self.reset_scroll();
                self.clear_cache();
            }
            FocusArea::Content => {
                self.scroll_down(1);
            }
            _ => {}
        }
    }
    
    // 新增：处理菜单选择（回车键）
    pub fn handle_menu_selection(&mut self) {
        if self.focus_area == FocusArea::Menu && self.show_menu {
            // 根据当前选中的菜单项执行相应操作
            match self.menu.selected_item() {
                crate::menu::MenuItem::CpuTest => {
                    // 启动CPU测试
                    crate::handlers::cpu_test::start_cpu_test();
                }
                crate::menu::MenuItem::DiskTest => {
                    // 启动磁盘测试（如果磁盘测试有类似的start函数）
                    // crate::handlers::disk_test::start_disk_test();
                }
                _ => {
                    // 其他菜单项的处理
                }
            }
            
            // 选择后自动切换到内容区域
            self.set_focus(FocusArea::Content);
        }
    }
    
    pub fn scroll_up(&mut self, lines: u16) {
        self.scroll_position.scroll_up(lines);
        self.update_scrollbar();
    }
    
    pub fn scroll_down(&mut self, lines: u16) {
        self.scroll_position.scroll_down(lines);
        self.update_scrollbar();
    }
    
    // 修改：翻页只在内容区域有焦点时生效
    pub fn page_up(&mut self) {
        if self.focus_area == FocusArea::Content {
            let page_size = self.scroll_position.viewport_height.saturating_sub(2);
            self.scroll_up(page_size);
        }
    }
    
    // 修改：翻页只在内容区域有焦点时生效
    pub fn page_down(&mut self) {
        if self.focus_area == FocusArea::Content {
            let page_size = self.scroll_position.viewport_height.saturating_sub(2);
            self.scroll_down(page_size);
        }
    }
    
    pub fn reset_scroll(&mut self) {
        self.scroll_position.reset();
        self.update_scrollbar();
    }
    
    pub fn update_content_height(&mut self, height: u16, viewport_height: u16) {
        self.scroll_position.max = height;
        self.scroll_position.viewport_height = viewport_height;
        self.update_scrollbar();
    }
    
    pub fn update_scrollbar(&mut self) {
        // 修复滚动条状态更新逻辑
        let content_length = self.scroll_position.max as usize;
        let current_position = self.scroll_position.current as usize;
        
        // 滚动条状态应该反映总内容长度和当前位置
        self.scrollbar_state = self.scrollbar_state
            .content_length(content_length)
            .position(current_position);
    }
    
    pub fn get_content(&mut self) -> String {
        let current_item = self.menu.selected_item();
        
        // 检查缓存
        if let Some((cached_item, cached_content)) = &self.content_cache {
            if *cached_item == current_item {
                return cached_content.clone();
            }
        }
        
        // 获取新内容
        let content = handlers::get_content(current_item);
        self.content_cache = Some((current_item, content.clone()));
        content
    }
    
    pub fn clear_cache(&mut self) {
        self.content_cache = None;
    }
    
    // 新增：设置需要刷新标志
    pub fn set_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
    
    // 新增：检查并重置刷新标志
    pub fn check_needs_refresh(&mut self) -> bool {
        if self.needs_refresh {
            self.needs_refresh = false;
            true
        } else {
            false
        }
    }
    
    // 新增：切换选择模式
    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = !self.selection_mode;
    }
    
    // 新增：复制当前内容到剪贴板
    pub fn copy_content(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = if let Some((_, cached_content)) = &self.content_cache {
            cached_content.clone()
        } else {
            return Ok(()); // 没有内容可复制
        };
        
        // 尝试使用系统命令复制到剪贴板
        #[cfg(target_os = "linux")]
        {
            use std::process::{Command, Stdio};
            use std::io::Write;
            
            // 尝试使用 xclip
            if let Ok(mut child) = Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(content.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
            
            // 如果 xclip 不可用，尝试使用 xsel
            if let Ok(mut child) = Command::new("xsel")
                .arg("--clipboard")
                .arg("--input")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(content.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::{Command, Stdio};
            use std::io::Write;
            
            if let Ok(mut child) = Command::new("pbcopy")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(content.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            use std::process::{Command, Stdio};
            use std::io::Write;
            
            if let Ok(mut child) = Command::new("clip")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(content.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    // 新增：临时禁用鼠标捕获以允许文本选择
    pub fn enable_text_selection() -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{execute, event::DisableMouseCapture};
        use std::io::stdout;
        
        execute!(stdout(), DisableMouseCapture)?;
        Ok(())
    }
    
    // 新增：重新启用鼠标捕获
    pub fn disable_text_selection() -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{execute, event::EnableMouseCapture};
        use std::io::stdout;
        
        execute!(stdout(), EnableMouseCapture)?;
        Ok(())
    }
}