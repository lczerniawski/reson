use ratatui::{
    layout::Direction,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
};
use sysinfo::{Cpu, CpuExt, System, SystemExt};

pub fn create_top_cpu_barchart(sys: &System) -> BarChart<'_> {
    let mut cpus: Vec<(&Cpu, usize)> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| (cpu, i + 1))
        .collect();
    cpus.sort_by(|a, b| {
        b.0.cpu_usage()
            .partial_cmp(&a.0.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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

pub fn create_details_cpu_barchart(
    sys: &System,
    layout_height: u16,
    scroll_position: usize,
) -> (BarChart<'_>, usize) {
    let bar_width: u16 = 3;
    let bar_gap: u16 = 1;

    let title = Block::default()
        .title(format!(
            "CPU: {}, Total Usage: {}%, Max Frequence: {} MHz",
            sys.global_cpu_info().brand(),
            sys.global_cpu_info().cpu_usage().round(),
            sys.global_cpu_info().frequency()
        ))
        .borders(Borders::all());

    // -2 for borders
    let visible_bars = layout_height.saturating_sub(2) / (bar_width + bar_gap);

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
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();

    let barchart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .block(title)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(100)
        .style(Style::default().fg(Color::Green))
        .direction(Direction::Horizontal);

    let content_length = sys.cpus().len() - visible_bars as usize;

    (barchart, content_length)
}
