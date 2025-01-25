use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use sysinfo::{DiskExt, System, SystemExt};

pub fn create_top_disks_barchart(sys: &System) -> Paragraph<'_> {
    let mut disks: Vec<_> = sys.disks().iter().collect();
    disks.sort_by(|a, b| {
        b.available_space()
            .partial_cmp(&a.available_space())
            .unwrap_or(std::cmp::Ordering::Equal)
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
