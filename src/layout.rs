use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub main_layout: MainLayout,
    pub footer_area: Rect,
}

pub struct MainLayout {
    pub cpu_plus_memory_layout: CpuMemoryLayout,
    pub processes_layout: Rect,
    pub disk_layout: Rect,
    pub network_layout: Rect,
}

pub struct CpuMemoryLayout {
    pub cpu_layout: Rect,
    pub memory_layout: MemoryLayout,
}

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
