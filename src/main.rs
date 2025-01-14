use std::{thread, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph},
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
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(30), // CPU + Top Processes
                            Constraint::Percentage(10), // Memory
                            Constraint::Percentage(20), // Disk
                            Constraint::Percentage(20), // Network
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let inner_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(outer_layout[0]);

                let cpu_data: Vec<(String, u64)> = sys
                    .cpus()
                    .iter()
                    .enumerate()
                    .map(|(i, cpu)| {
                        (
                            format!("CPU {}", i).as_str().to_string(),
                            cpu.cpu_usage() as u64,
                        )
                    })
                    .collect();

                let cpu_chart_data: Vec<(&str, u64)> = cpu_data
                    .iter()
                    .map(|(cpu, usage)| (cpu.as_str(), *usage))
                    .collect();

                let cpu_barchart = BarChart::default()
                    .block(Block::default().title("CPU Usage").borders(Borders::all()))
                    .data(&cpu_chart_data)
                    .bar_width(5)
                    .bar_gap(2)
                    .style(Style::default().fg(Color::Green))
                    .value_style(Style::default().fg(Color::White));

                f.render_widget(cpu_barchart, inner_layout[0]);

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

                let process_widget = List::new(top_processes)
                    .block(
                        Block::default()
                            .title("Top Processes")
                            .borders(Borders::all()),
                    )
                    .style(Style::default().fg(Color::Cyan));

                f.render_widget(process_widget, inner_layout[1]);

                let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0;
                let used_memory =
                    (sys.total_memory() - sys.available_memory()) as f64 / 1024.0 / 1024.0;
                let memory_percentage = (used_memory / total_memory) * 100.0;

                let memory_gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title("Memory Usage")
                            .borders(Borders::all()),
                    )
                    .gauge_style(Style::default().fg(Color::Blue))
                    .ratio(memory_percentage / 100.0);

                f.render_widget(memory_gauge, outer_layout[1]);

                let disk_data: Vec<(String, u64)> = sys
                    .disks()
                    .iter()
                    .map(|disk| {
                        let used = (disk.total_space() - disk.available_space()) as f64;
                        let total = disk.total_space() as f64;
                        let usage = (used / total * 100.0) as u64;
                        (disk.name().to_string_lossy().to_string(), usage)
                    })
                    .collect();

                let disk_chart_data: Vec<(&str, u64)> = disk_data
                    .iter()
                    .map(|(name, usage)| (name.as_str(), *usage))
                    .collect();

                let disk_chart = BarChart::default()
                    .block(
                        Block::default()
                            .title("Disk Usage %")
                            .borders(Borders::all()),
                    )
                    .data(&disk_chart_data)
                    .bar_width(7)
                    .bar_gap(3)
                    .style(Style::default().fg(Color::Yellow));

                f.render_widget(disk_chart, outer_layout[2]);

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

                let network_widget = Paragraph::new(network_text).block(
                    Block::default()
                        .title("Network Usage")
                        .borders(Borders::all()),
                );

                f.render_widget(network_widget, outer_layout[3]);
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
