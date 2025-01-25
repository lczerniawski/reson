use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};
use sysinfo::{System, SystemExt};

pub struct MemoryGauges<'a> {
    pub main_memory_gauge: Gauge<'a>,
    pub swap_gauge: Gauge<'a>,
}

pub fn create_memory_gauges(sys: &System) -> MemoryGauges {
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
