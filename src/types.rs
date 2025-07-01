use std::error::Error;

/// 统一的 Result 类型
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// 滚动位置信息
#[derive(Debug, Clone, Copy)]
pub struct ScrollPosition {
    pub current: u16,
    pub max: u16,
    pub viewport_height: u16,
}

impl ScrollPosition {
    pub fn new() -> Self {
        Self {
            current: 0,
            max: 0,
            viewport_height: 0,
        }
    }
    
    pub fn can_scroll_down(&self) -> bool {
        if self.max <= self.viewport_height {
            false
        } else {
            self.current < self.max.saturating_sub(self.viewport_height)
        }
    }
    
    pub fn can_scroll_up(&self) -> bool {
        self.current > 0
    }
    
    pub fn scroll_down(&mut self, lines: u16) {
        if self.max > self.viewport_height {
            let max_scroll = self.max.saturating_sub(self.viewport_height);
            self.current = (self.current + lines).min(max_scroll);
        }
    }
    
    pub fn scroll_up(&mut self, lines: u16) {
        self.current = self.current.saturating_sub(lines);
    }
    
    pub fn reset(&mut self) {
        self.current = 0;
    }
}