use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use sysinfo::{DiskExt, System, SystemExt};

use crate::layout::get_highlight_style;

pub struct DisksWidget<'a_> {
    pub chart: Paragraph<'a_>,
    pub max_scroll: usize,
}

pub fn create_disks_widget(
    sys: &System,
    layout_height: usize,
    scroll_position: usize,
    is_selected: bool,
) -> DisksWidget {
    // -2 for border
    let visible_lines = layout_height - 2;
    let highlight_style = get_highlight_style(is_selected);

    let mut disks: Vec<_> = sys.disks().iter().collect();
    disks.sort_by(|a, b| {
        let b_used = b.total_space() - b.available_space();
        let a_used = a.total_space() - a.available_space();

        b_used
            .partial_cmp(&a_used)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let disk_data: String = disks
        .iter()
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

    let all_lines_count = sys.disks().len();
    let max_scroll = all_lines_count.saturating_sub(visible_lines);
    let paragraph = Paragraph::new(disk_data)
        .block(
            Block::default()
                .title("Disk Usage")
                .style(Style::default().fg(Color::Yellow))
                .title_style(highlight_style.title)
                .borders(Borders::all())
                .border_style(highlight_style.border)
                .border_type(highlight_style.border_type),
        )
        .scroll((scroll_position as u16, 0));

    DisksWidget {
        chart: paragraph,
        max_scroll,
    }
}
