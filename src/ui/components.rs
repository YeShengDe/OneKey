use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation},
    Frame,
};

use crate::{app::App, theme::Theme};

/// 绘制左侧菜单
pub fn draw_menu(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
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

/// 绘制滚动条
pub fn draw_scrollbar(f: &mut Frame, app: &mut App, area: Rect, is_focused: bool) {
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
