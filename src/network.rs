use sysinfo::{NetworkExt, NetworksExt, System, SystemExt};

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

struct TotalNetworkStats {
    transmited_bytes: u64,
    received_bytes: u64,
    transmited_packets: u64,
    received_packets: u64,
}

fn format_bytes_per_second(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB/s", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB/s", bytes / 1024)
    }
}

pub fn create_top_networks_widget(sys: &System) -> Paragraph<'_> {
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
