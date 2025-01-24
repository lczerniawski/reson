use std::{io::Stdout, rc::Rc, thread, time::Duration};

use color_eyre::{eyre::Ok, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};
use sysinfo::{
    Cpu, CpuExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, System, SystemExt, UserExt,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut sys = System::new_all();

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    App::new().run(&mut terminal, &mut sys)?;

    disable_raw_mode().unwrap();
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .unwrap();
    Ok(())
}

struct App {
    should_exit: bool,
}

impl App {
    fn new() -> Self {
        Self { should_exit: false }
    }

    fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        while !self.should_exit {
            sys.refresh_all();
            terminal.draw(|frame| self.draw(frame, sys))?;
            self.handle_events()?;
            thread::sleep(Duration::from_millis(250));
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.should_exit = true;
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame, sys: &System) {
        let app_layout = prepare_layout(frame);

        let top_cpu_barchart = create_top_cpu_barchart(sys);
        frame.render_widget(top_cpu_barchart, app_layout.inner_layout[0]);

        let memory_gauges = create_memory_gauges(sys);
        frame.render_widget(memory_gauges.main_memory_gauge, app_layout.memory_layout[0]);
        frame.render_widget(memory_gauges.swap_gauge, app_layout.memory_layout[1]);

        let top_processes_table = create_top_processes_table(sys);
        frame.render_widget(top_processes_table, app_layout.outer_layout[1]);

        let disk_barchart = create_top_disks_barchart(sys);
        frame.render_widget(disk_barchart, app_layout.outer_layout[2]);

        let network_widget = create_top_networks_widget(sys);
        frame.render_widget(network_widget, app_layout.outer_layout[3]);

        let exit_message = Block::default().title("Click 'q' to exit.");
        frame.render_widget(exit_message, app_layout.outer_layout[4]);
    }
}

fn format_bytes_per_second(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB/s", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB/s", bytes / 1024)
    }
}

struct TotalNetworkStats {
    transmited_bytes: u64,
    received_bytes: u64,
    transmited_packets: u64,
    received_packets: u64,
}

fn create_top_networks_widget(sys: &System) -> Paragraph<'_> {
    let mut networks: Vec<_> = sys.networks().iter().collect();
    networks.sort_by(|a, b| {
        let a_transmited = a.1.transmitted();
        let b_transmited = b.1.transmitted();

        let a_received = a.1.received();
        let b_received = b.1.received();

        let a_combined = a_transmited + a_received;
        let b_combined = b_transmited + b_received;

        b_combined.partial_cmp(&a_combined).unwrap()
    });

    let network_text = networks
        .iter()
        .take(5)
        .map(|(network, data)| {
            format!(
                "{}: ↑ {} KB/s ↓ {} KB/s | Packets: TX {} RX {} | MAC: {}",
                network,
                format_bytes_per_second(data.transmitted()),
                format_bytes_per_second(data.received()),
                data.packets_transmitted(),
                data.packets_transmitted(),
                data.mac_address()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let total_stats = sys.networks().iter().fold(
        TotalNetworkStats {
            transmited_bytes: 0,
            received_bytes: 0,
            transmited_packets: 0,
            received_packets: 0,
        },
        |mut stats, (_name, data)| {
            stats.transmited_bytes += data.transmitted();
            stats.received_bytes += data.received();
            stats.transmited_packets += data.packets_transmitted();
            stats.received_packets += data.packets_received();
            stats
        },
    );

    let title = format!(
        "Top 5 Network Usage, Total: ↑ {} KB/s ↓ {} KB/s | Packets: TX {} RX {}",
        format_bytes_per_second(total_stats.transmited_bytes),
        format_bytes_per_second(total_stats.received_bytes),
        total_stats.transmited_packets,
        total_stats.received_packets
    );

    Paragraph::new(network_text).block(
        Block::default()
            .title(title)
            .style(Style::default().fg(Color::LightRed))
            .borders(Borders::all()),
    )
}

fn create_top_disks_barchart(sys: &System) -> Paragraph<'_> {
    let mut disks: Vec<_> = sys.disks().iter().collect();
    disks.sort_by(|a, b| {
        b.available_space()
            .partial_cmp(&a.available_space())
            .unwrap()
    });

    let disk_data: String = disks
        .iter()
        .take(5)
        .enumerate()
        .map(|(n, disk)| {
            let used = disk.total_space() - disk.available_space();
            let total = disk.total_space();
            let usage_percentage = (used as f64 / total as f64 * 100.0) as u64;
            let free_percentage = (disk.available_space() as f64 / total as f64 * 100.0) as u64;

            format!(
                "{}. {} [Free: {}%({} GB), Used: {}%({} GB), Total: {} GB]",
                n + 1,
                disk.name().to_string_lossy(),
                free_percentage,
                disk.available_space() / 1024 / 1024 / 1024,
                usage_percentage,
                used / 1024 / 1024 / 1024,
                disk.total_space() / 1024 / 1024 / 1024
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    Paragraph::new(disk_data).block(
        Block::default()
            .title("Top 5 Disk Usage")
            .style(Style::default().fg(Color::Yellow))
            .borders(Borders::all()),
    )
}

struct MemoryGauges<'a> {
    main_memory_gauge: Gauge<'a>,
    swap_gauge: Gauge<'a>,
}

fn create_memory_gauges(sys: &System) -> MemoryGauges {
    let total_memory_gb = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let used_memory_gb = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let memory_percentage = (used_memory_gb / total_memory_gb) * 100.0;

    let total_swap_gb = sys.total_swap() as f64 / 1024.0 / 1024.0;
    let used_swap_gb = sys.used_swap() as f64 / 1024.0 / 1024.;
    let swap_percentage = (used_swap_gb / total_swap_gb) * 100.0;

    let memory_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(
                    "Memory Usage, Total: {} MB, Used: {} MB",
                    total_memory_gb.round(),
                    used_memory_gb.round(),
                ))
                .borders(Borders::all()),
        )
        .gauge_style(Style::default().fg(Color::Blue))
        .style(Style::default().fg(Color::Blue))
        .percent(memory_percentage as u16);

    let swap_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(
                    "Swap Usage, Total: {} MB, Used: {} MB",
                    total_swap_gb, used_swap_gb
                ))
                .borders(Borders::all()),
        )
        .gauge_style(Style::default().fg(Color::LightMagenta))
        .style(Style::default().fg(Color::LightMagenta))
        .percent(swap_percentage as u16);

    MemoryGauges {
        main_memory_gauge: memory_gauge,
        swap_gauge,
    }
}

fn create_top_processes_table(sys: &System) -> Table<'_> {
    let mut processes: Vec<_> = sys.processes().values().collect();

    let total_memory = sys.total_memory() as f64;
    processes.sort_by(|a, b| {
        let a_cpu_score = a.cpu_usage() as f64;
        let b_cpu_score = b.cpu_usage() as f64;

        let a_mem_score = (a.memory() as f64 / total_memory) * 100.0;
        let b_mem_score = (b.memory() as f64 / total_memory) * 100.0;

        let a_combined = a_cpu_score + a_mem_score;
        let b_combined = b_cpu_score + b_mem_score;

        b_combined.partial_cmp(&a_combined).unwrap()
    });

    let header = Row::new(vec!["User", "PID", "CPU%", "MEM(MB)", "Time", "Command"])
        .style(Style::default().fg(Color::Gray));

    let rows: Vec<Row> = processes
        .iter()
        .take(10)
        .map(|process| {
            Row::new(vec![
                process
                    .user_id()
                    .and_then(|id| sys.get_user_by_id(&id))
                    .map(|user| user.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                process.pid().to_string(),
                format!("{:.1}", process.cpu_usage()),
                format!("{}", process.memory() / 1024 / 1024),
                format!(
                    "{:02}:{:02}:{:02}",
                    process.run_time() / 60 / 60,
                    process.run_time() / 60 % 60,
                    process.run_time() % 60
                ),
                process.name().to_string(),
            ])
        })
        .collect();

    Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Top 10 Processes")
                .borders(Borders::all()),
        )
        .style(Style::default().fg(Color::Cyan))
        .widths(&[
            Constraint::Percentage(15), // User
            Constraint::Percentage(10), // PID
            Constraint::Percentage(10), // CPU%
            Constraint::Percentage(10), // MEM
            Constraint::Percentage(15), // Time
            Constraint::Percentage(40), // Command
        ])
        .column_spacing(1)
}

fn create_top_cpu_barchart(sys: &System) -> BarChart<'_> {
    let mut cpus: Vec<(&Cpu, usize)> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| (cpu, i + 1))
        .collect();
    cpus.sort_by(|a, b| b.0.cpu_usage().partial_cmp(&a.0.cpu_usage()).unwrap());

    let cpu_data: Vec<Bar> = cpus
        .iter()
        .take(5)
        .map(|(cpu, cpu_count)| {
            let cpu_usage = cpu.cpu_usage() as u64;
            Bar::default()
                .value(cpu_usage)
                .label(Line::from(format!("CPU {}", cpu_count)))
                .text_value(format!("{cpu_usage:>3}%"))
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();

    let global_cpu_usage = sys.global_cpu_info().cpu_usage();
    BarChart::default()
        .block(
            Block::default()
                .title(format!(
                    "Top 5 CPU Usage, Total: {}%",
                    global_cpu_usage.round()
                ))
                .borders(Borders::all()),
        )
        .data(BarGroup::default().bars(&cpu_data))
        .style(Style::default().fg(Color::Green))
        .bar_width(7)
        .bar_gap(2)
        .max(100)
}

struct AppLayout {
    outer_layout: Rc<[Rect]>,
    inner_layout: Rc<[Rect]>,
    memory_layout: Rc<[Rect]>,
}

fn prepare_layout(f: &mut ratatui::Frame<'_>) -> AppLayout {
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
        .split(f.size());

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

    AppLayout {
        outer_layout,
        inner_layout,
        memory_layout,
    }
}
