use ratatui::style::{Color, Style, Modifier};

/// 应用主题颜色配置
pub struct Theme;

impl Theme {
    // 主要颜色
    pub const PRIMARY: Color = Color::Cyan;           // 主要强调色
    pub const SECONDARY: Color = Color::White;        // 次要文本色
    pub const MUTED: Color = Color::DarkGray;         // 静音/未激活色
    pub const BACKGROUND: Color = Color::Black;       // 背景色
    pub const ACCENT: Color = Color::Blue;            // 重点色
    
    // 功能颜色
    pub const SUCCESS: Color = Color::Green;          // 成功/正常状态
    pub const WARNING: Color = Color::Yellow;         // 警告状态
    pub const ERROR: Color = Color::Red;              // 错误状态
    
    // 进度条颜色
    pub const GAUGE_NORMAL: Color = Color::Green;     // 正常范围 (0-70%)
    pub const GAUGE_WARNING: Color = Color::Rgb(255, 165, 0);  // 警告范围 (70-85%) 橙色
    pub const GAUGE_CRITICAL: Color = Color::Red;     // 危险范围 (85-100%)
    
    // 进度条背景颜色
    pub const GAUGE_BAR_NORMAL: Color = Color::Cyan;       // 正常状态的进度条
    pub const GAUGE_BAR_WARNING: Color = Color::Yellow;    // 警告状态的进度条
    pub const GAUGE_BAR_CRITICAL: Color = Color::Red;      // 危险状态的进度条
    
    // 预定义样式
    pub fn default() -> Style {
        Style::default().fg(Self::SECONDARY)
    }
    
    pub fn primary() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }
    
    pub fn secondary() -> Style {
        Style::default().fg(Self::SECONDARY)
    }
    
    pub fn muted() -> Style {
        Style::default().fg(Self::MUTED)
    }
    
    pub fn accent() -> Style {
        Style::default().fg(Self::ACCENT)
    }
    
    pub fn success() -> Style {
        Style::default().fg(Self::SUCCESS)
    }
    
    pub fn warning() -> Style {
        Style::default().fg(Self::WARNING)
    }
    
    pub fn error() -> Style {
        Style::default().fg(Self::ERROR)
    }
    
    // 边框样式
    pub fn border_focused() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }
    
    pub fn border_unfocused() -> Style {
        Style::default().fg(Self::MUTED)
    }
    
    // 标题样式
    pub fn title_focused() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }
    
    pub fn title_unfocused() -> Style {
        Style::default().fg(Self::MUTED)
    }
    
    // 列表选择样式
    pub fn list_selected() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn list_unselected() -> Style {
        Style::default().fg(Self::MUTED)
    }
    
    pub fn list_highlight() -> Style {
        Style::default()
            .bg(Color::Rgb(30, 30, 30))  // 深灰色背景
            .add_modifier(Modifier::BOLD)
    }
    
    // 进度条样式
    // 系统信息使用率颜色 - 使用率越高越危险
    pub fn gauge_style(percentage: f64) -> Style {
        let color = if percentage < 70.0 {
            Self::SUCCESS           // 健康: 绿色
        } else if percentage < 85.0 {
            Self::WARNING           // 警告: 黄色
        } else {
            Self::ERROR             // 危险: 红色
        };
        
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }
    
    // 系统信息使用率进度条颜色 - 使用率越高越危险
    pub fn gauge_bar_style(percentage: f64) -> Style {
        let color = if percentage < 70.0 {
            Self::SUCCESS           // 健康: 绿色
        } else if percentage < 85.0 {
            Self::WARNING           // 警告: 黄色
        } else {
            Self::ERROR             // 危险: 红色
        };
        
        Style::default().fg(color)
    }
    
    // 磁盘测试进度条颜色 - 进度越高越接近完成(健康)
    pub fn disk_test_progress_style(percentage: f64) -> Style {
        let color = if percentage < 30.0 {
            Self::ERROR             // 0-29%: 红色（刚开始）
        } else if percentage < 70.0 {
            Self::WARNING           // 30-69%: 黄色（进行中）
        } else {
            Self::SUCCESS           // 70-100%: 绿色（接近完成）
        };
        
        Style::default().fg(color)
    }
    
    // 滚动条样式
    pub fn scrollbar_focused() -> Style {
        Style::default().fg(Self::PRIMARY)
    }
    
    pub fn scrollbar_unfocused() -> Style {
        Style::default().fg(Self::MUTED)
    }
    
    pub fn scrollbar_track() -> Style {
        Style::default().fg(Color::Rgb(20, 20, 20))
    }
    
    // 帮助栏样式
    pub fn help_bar() -> Style {
        Style::default().fg(Self::MUTED)
    }
}
