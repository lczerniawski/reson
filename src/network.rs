use sysinfo::{NetworkExt, NetworksExt, System, SystemExt};

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::layout::get_highlight_style;

struct TotalNetworkStats {
    transmited_bytes: u64,
    received_bytes: u64,
    transmited_packets: u64,
    received_packets: u64,
}

pub struct NetworksWidget<'a_> {
    pub chart: Paragraph<'a_>,
    pub max_scroll: usize,
}

fn format_bytes_per_second(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB/s", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB/s", bytes / 1024)
    }
}

pub fn create_networks_widget(
    sys: &System,
    layout_height: usize,
    scroll_position: usize,
    is_selected: bool,
) -> NetworksWidget<'_> {
    // -2 for border
    let visible_lines = layout_height - 2;
    let highlight_style = get_highlight_style(is_selected);

    let mut networks: Vec<_> = sys.networks().iter().collect();
    networks.sort_by(|a, b| {
        let a_transmited = a.1.transmitted();
        let b_transmited = b.1.transmitted();

        let a_received = a.1.received();
        let b_received = b.1.received();

        let a_combined = a_transmited + a_received;
        let b_combined = b_transmited + b_received;

        b_combined
            .partial_cmp(&a_combined)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let network_text = networks
        .iter()
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
        "Network Usage, Total: ↑ {} KB/s ↓ {} KB/s | Packets: TX {} RX {}",
        format_bytes_per_second(total_stats.transmited_bytes),
        format_bytes_per_second(total_stats.received_bytes),
        total_stats.transmited_packets,
        total_stats.received_packets
    );

    let all_lines_count = networks.len();
    let max_scroll = all_lines_count.saturating_sub(visible_lines);
    let paragraph = Paragraph::new(network_text)
        .block(
            Block::default()
                .title(title)
                .style(Style::default().fg(Color::Gray))
                .title_style(highlight_style.title)
                .borders(Borders::all())
                .border_style(highlight_style.border)
                .border_type(highlight_style.border_type),
        )
        .scroll((scroll_position as u16, 0));

    NetworksWidget {
        chart: paragraph,
        max_scroll,
    }
}
