use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
};
use sysinfo::{CpuExt, System, SystemExt};

pub fn create_top_cpu_barchart(
    sys: &System,
    layout_width: u16,
    scroll_position: usize,
) -> (BarChart<'_>, usize) {
    let bar_width: u16 = 7;
    let bar_gap: u16 = 2;
    let visible_bars = layout_width / (bar_width + bar_gap);

    let cpu_data: Vec<Bar> = sys
        .cpus()
        .iter()
        .enumerate()
        .skip(scroll_position)
        .take(visible_bars as usize)
        .map(|(cpu_count, cpu)| {
            let cpu_usage = cpu.cpu_usage() as u64;
            Bar::default()
                .value(cpu_usage)
                .label(Line::from(format!("CPU {}", cpu_count + 1)))
                .text_value(format!("{cpu_usage:>3}%"))
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();

    let content_length = sys.cpus().len().saturating_sub(visible_bars as usize);
    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(format!(
                    "CPU Usage, Total: {}%, Max Frequency: {} MHz",
                    sys.global_cpu_info().cpu_usage().round(),
                    sys.global_cpu_info().frequency()
                ))
                .borders(Borders::all()),
        )
        .data(BarGroup::default().bars(&cpu_data))
        .style(Style::default().fg(Color::Green))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(100);

    (barchart, content_length)
}
