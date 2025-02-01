use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
    Frame,
};
use sysinfo::{CpuExt, System, SystemExt};

use crate::{
    cpu::create_top_cpu_barchart, disk::create_top_disks_barchart, memory::create_memory_gauges,
    network::create_top_networks_widget, processes::create_top_processes_table,
};

pub struct AppLayout {
    pub header_area: Rect,
    pub footer_area: Rect,
    pub summary_tab_layout: SummaryTabLayout,
    pub cpu_details_tab_layout: CpuDetailsTabLayout,
}

pub struct SummaryTabLayout {
    pub outer_layout: Rc<[Rect]>,
    pub inner_layout: Rc<[Rect]>,
    pub memory_layout: Rc<[Rect]>,
}

pub struct CpuDetailsTabLayout {
    pub main_layout: Rect,
}

pub fn prepare_layout(f: &mut ratatui::Frame<'_>) -> AppLayout {
    use Constraint::{Length, Min};
    let app_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Length(1), Min(0), Length(1)])
        .split(f.size());

    let header_area = app_layout[0];
    let inner_area = app_layout[1];
    let footer_area = app_layout[2];

    AppLayout {
        header_area,
        footer_area,
        summary_tab_layout: prepare_summary_tab_layout(inner_area),
        cpu_details_tab_layout: CpuDetailsTabLayout {
            main_layout: inner_area,
        },
    }
}

fn prepare_summary_tab_layout(inner_area: Rect) -> SummaryTabLayout {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(30), // CPU + Memory
            Constraint::Percentage(30), // Top Processes
            Constraint::Percentage(18), // Disk
            Constraint::Percentage(20), // Network
            Constraint::Percentage(2),  // Exit Message
        ])
        .split(inner_area);

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer_layout[0]);

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_layout[1]);

    SummaryTabLayout {
        outer_layout,
        inner_layout,
        memory_layout,
    }
}

// fn prepare_cpu_tab_layout(inner_area: Rect) -> CpuDetailsTabLayout {}

pub fn render_summary_tab(frame: &mut Frame, sys: &System, summary_tab_layout: &SummaryTabLayout) {
    let top_cpu_barchart = create_top_cpu_barchart(sys);
    frame.render_widget(top_cpu_barchart, summary_tab_layout.inner_layout[0]);

    let memory_gauges = create_memory_gauges(sys);
    frame.render_widget(
        memory_gauges.main_memory_gauge,
        summary_tab_layout.memory_layout[0],
    );
    frame.render_widget(
        memory_gauges.swap_gauge,
        summary_tab_layout.memory_layout[1],
    );

    let top_processes_table = create_top_processes_table(sys);
    frame.render_widget(top_processes_table, summary_tab_layout.outer_layout[1]);

    let disk_barchart = create_top_disks_barchart(sys);
    frame.render_widget(disk_barchart, summary_tab_layout.outer_layout[2]);

    let network_widget = create_top_networks_widget(sys);
    frame.render_widget(network_widget, summary_tab_layout.outer_layout[3]);
}

pub fn render_cpu_details_tab(
    frame: &mut Frame,
    sys: &System,
    cpu_tab_layout: &CpuDetailsTabLayout,
    scroll_position: usize,
) -> usize {
    let bar_width: u16 = 5;
    let bar_gap: u16 = 1;

    let title = Block::default()
        .title(format!(
            "CPU: {}, Frequence: {} MHz",
            sys.global_cpu_info().brand(),
            sys.global_cpu_info().frequency()
        ))
        .borders(Borders::all());

    // -2 for borders
    let visible_bars =
        (cpu_tab_layout.main_layout.height).saturating_sub(2) / (bar_width + bar_gap);

    let visible_cpus = sys
        .cpus()
        .iter()
        .enumerate()
        .skip(scroll_position)
        .take(visible_bars as usize);

    let bars: Vec<_> = visible_cpus
        .map(|(i, cpu)| {
            let cpu_usage = cpu.cpu_usage() as u64;

            Bar::default()
                .value(cpu_usage)
                .text_value(format!("{cpu_usage:>3}%"))
                .label(Line::from(format!("Core #{}", i + 1)))
        })
        .collect();

    let barchart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .block(title)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(100)
        .direction(Direction::Horizontal);

    let content_length = sys.cpus().len() - visible_bars as usize;
    frame.render_widget(barchart, cpu_tab_layout.main_layout);

    content_length
}
