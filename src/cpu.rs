use ratatui::{
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders},
};
use sysinfo::{CpuExt, System, SystemExt};

pub struct CpuBarchart<'a_> {
    pub chart: BarChart<'a_>,
    pub max_scroll: usize,
    pub real_content_length: usize,
}
pub fn create_cpu_barchart(
    sys: &System,
    layout_width: usize,
    scroll_position: usize,
    is_selected: bool,
) -> CpuBarchart<'_> {
    let bar_width: u16 = 7;
    let bar_gap: u16 = 2;
    let visible_bars = layout_width / (bar_width + bar_gap) as usize;

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

    let cpu_data: Vec<Bar> = sys
        .cpus()
        .iter()
        .enumerate()
        .skip(scroll_position)
        .take(visible_bars)
        .map(|(cpu_count, cpu)| {
            let cpu_usage = cpu.cpu_usage() as u64;
            Bar::default()
                .value(cpu_usage)
                .label(Line::from(format!("CPU {}", cpu_count + 1)))
                .text_value(format!("{cpu_usage:>3}%"))
                .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        })
        .collect();

    let bar_count = sys.cpus().len();
    let max_scroll = bar_count.saturating_sub(visible_bars as usize);
    let real_content_length = if visible_bars == bar_count {
        0
    } else {
        bar_count * (bar_width + bar_gap) as usize
    };

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(format!(
                    "CPU Usage, Total: {}%, Max Frequency: {} MHz",
                    sys.global_cpu_info().cpu_usage().round(),
                    sys.global_cpu_info().frequency()
                ))
                .title_style(title_style)
                .borders(Borders::all())
                .border_style(border_style)
                .border_type(border_type),
        )
        .data(BarGroup::default().bars(&cpu_data))
        .style(Style::default().fg(Color::Green))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .max(100);

    CpuBarchart {
        chart: barchart,
        max_scroll,
        real_content_length,
    }
}
