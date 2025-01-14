use std::{rc::Rc, thread, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, System, SystemExt};

fn main() {
    enable_raw_mode().unwrap();
    let mut sys = System::new_all();
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let tick_rate = Duration::from_millis(1000);

    loop {
        sys.refresh_all();

        terminal
            .draw(|f| {
                let reson_layout = prepare_layout(f);

                let cpu_barchart = create_cpu_barchart(&sys);
                f.render_widget(cpu_barchart, reson_layout.inner_layout[0]);

                let processes_widget = create_processes_widget(&sys);
                f.render_widget(processes_widget, reson_layout.inner_layout[1]);

                let memory_gauge = create_memory_gauge(&sys);
                f.render_widget(memory_gauge, reson_layout.outer_layout[1]);

                let disk_barchart = create_disk_barchart(&sys);
                f.render_widget(disk_barchart, reson_layout.outer_layout[2]);

                let network_widget = create_network_widget(&sys);
                f.render_widget(network_widget, reson_layout.outer_layout[3]);
            })
            .unwrap();

        if event::poll(tick_rate).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    disable_raw_mode().unwrap();
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .unwrap();
}

fn create_network_widget(sys: &System) -> Paragraph<'_> {
    let network_text = sys
        .networks()
        .iter()
        .map(|(network, data)| {
            format!(
                "{}: ↑ {} KB/s ↓ {} KB/",
                network,
                data.transmitted() / 1024,
                data.received() / 1024
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    Paragraph::new(network_text).block(
        Block::default()
            .title("Network Usage")
            .borders(Borders::all()),
    )
}

fn create_disk_barchart(sys: &System) -> BarChart<'_> {
    let disk_data: Vec<Bar> = sys
        .disks()
        .iter()
        .map(|disk| {
            let used = (disk.total_space() - disk.available_space()) as f64;
            let total = disk.total_space() as f64;
            let usage = (used / total * 100.0) as u64;
            Bar::default()
                .value(usage)
                .label(Line::from(format!("{}", disk.name().to_string_lossy())))
                .style(Style::default().fg(Color::Yellow))
        })
        .collect();

    BarChart::default()
        .block(
            Block::default()
                .title("Disk Usage %")
                .borders(Borders::all()),
        )
        .data(BarGroup::default().bars(&disk_data))
        .bar_width(7)
        .bar_gap(3)
}

fn create_memory_gauge(sys: &System) -> Gauge<'_> {
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let used_memory = (sys.total_memory() - sys.available_memory()) as f64 / 1024.0 / 1024.0;
    let memory_percentage = (used_memory / total_memory) * 100.0;

    Gauge::default()
        .block(
            Block::default()
                .title("Memory Usage")
                .borders(Borders::all()),
        )
        .gauge_style(Style::default().fg(Color::Blue))
        .ratio(memory_percentage / 100.0)
}

fn create_processes_widget(sys: &System) -> List<'_> {
    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

    let top_processes: Vec<ListItem> = processes
        .iter()
        .take(10)
        .map(|process| {
            ListItem::new(format!(
                "{:<30} CPU: {:>5.1}% MEM: {:>5.1}MB",
                process.name(),
                process.cpu_usage(),
                process.memory() / 1024 / 1024
            ))
        })
        .collect();

    List::new(top_processes)
        .block(
            Block::default()
                .title("Top Processes")
                .borders(Borders::all()),
        )
        .style(Style::default().fg(Color::Cyan))
}

fn create_cpu_barchart(sys: &System) -> BarChart<'_> {
    let cpu_data: Vec<Bar> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| {
            let cpu_usage = cpu.cpu_usage() as u64;
            Bar::default()
                .value(cpu_usage)
                .label(Line::from(format!("CPU {}", i)))
                .text_value(format!("{cpu_usage:>3}%"))
                .style(Style::default().fg(Color::Green))
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();

    BarChart::default()
        .block(Block::default().title("CPU Usage").borders(Borders::all()))
        .data(BarGroup::default().bars(&cpu_data))
        .bar_width(5)
        .bar_gap(2)
}

struct AppLayout {
    outer_layout: Rc<[Rect]>,
    inner_layout: Rc<[Rect]>,
}

fn prepare_layout(f: &mut ratatui::Frame<'_>) -> AppLayout {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(30), // CPU + Top Processes
            Constraint::Percentage(10), // Memory
            Constraint::Percentage(20), // Disk
            Constraint::Percentage(20), // Network
        ])
        .split(f.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer_layout[0]);

    AppLayout {
        outer_layout,
        inner_layout,
    }
}
