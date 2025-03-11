use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{BorderType, Scrollbar, ScrollbarOrientation},
};

#[derive(Clone)]
pub struct AppLayout {
    pub main_layout: MainLayout,
    pub footer_area: Rect,
}

impl AppLayout {
    pub fn empty() -> AppLayout {
        AppLayout {
            main_layout: MainLayout {
                cpu_plus_memory_layout: CpuMemoryLayout {
                    cpu_layout: Rect::default(),
                    memory_layout: MemoryLayout {
                        ram_layout: Rect::default(),
                        swap_layout: Rect::default(),
                    },
                },
                processes_layout: Rect::default(),
                disk_layout: Rect::default(),
                network_layout: Rect::default(),
            },
            footer_area: Rect::default(),
        }
    }
}

#[derive(Clone)]
pub struct MainLayout {
    pub cpu_plus_memory_layout: CpuMemoryLayout,
    pub processes_layout: Rect,
    pub disk_layout: Rect,
    pub network_layout: Rect,
}

#[derive(Clone)]
pub struct CpuMemoryLayout {
    pub cpu_layout: Rect,
    pub memory_layout: MemoryLayout,
}

#[derive(Clone)]
pub struct MemoryLayout {
    pub ram_layout: Rect,
    pub swap_layout: Rect,
}

pub fn prepare_layout(f: &mut ratatui::Frame<'_>) -> AppLayout {
    use Constraint::{Length, Min};
    let app_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Min(0), Length(1)])
        .split(f.size());

    let main_area = app_layout[0];
    let footer_area = app_layout[1];

    AppLayout {
        main_layout: prepare_main_layout(main_area),
        footer_area,
    }
}

pub fn is_within_rect(pos: (u16, u16), rect: &Rect) -> bool {
    let (x, y) = pos;
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

fn prepare_main_layout(inner_area: Rect) -> MainLayout {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(1)
        .constraints([
            Constraint::Percentage(30), // CPU + Memory
            Constraint::Percentage(30), // Top Processes
            Constraint::Percentage(18), // Disk
            Constraint::Percentage(20), // Network
        ])
        .split(inner_area);

    let cpu_plus_memory_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cpu_plus_memory_layout[1]);

    MainLayout {
        cpu_plus_memory_layout: CpuMemoryLayout {
            cpu_layout: cpu_plus_memory_layout[0],
            memory_layout: MemoryLayout {
                ram_layout: memory_layout[0],
                swap_layout: memory_layout[1],
            },
        },
        processes_layout: main_layout[1],
        disk_layout: main_layout[2],
        network_layout: main_layout[3],
    }
}

pub struct HighlightStyle {
    pub border: Style,
    pub title: Style,
    pub border_type: BorderType,
}

pub fn get_highlight_style(is_selected: bool) -> HighlightStyle {
    let border_style = if is_selected {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };

    let title_style = if is_selected {
        Style::default().bold()
    } else {
        Style::default()
    };

    let border_type = if is_selected {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    HighlightStyle {
        border: border_style,
        title: title_style,
        border_type,
    }
}

pub fn get_horizontal_scrollbar<'a>() -> Scrollbar<'a> {
    Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
        .track_symbol(Some("═"))
        .thumb_symbol("■")
        .begin_symbol(Some("◀"))
        .end_symbol(Some("▶"))
}

pub fn get_vertical_scrollbar<'a>() -> Scrollbar<'a> {
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
}
