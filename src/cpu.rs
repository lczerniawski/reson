use ratatui::{
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
