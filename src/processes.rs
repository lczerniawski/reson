use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
};
use sysinfo::{ProcessExt, System, SystemExt, UserExt};

use crate::layout::get_highlight_style;

pub struct ProcessesTable<'a_> {
    pub chart: Table<'a_>,
    pub max_scroll: usize,
    pub real_content_length: usize,
}

pub fn create_processes_table(
    sys: &System,
    layout_height: usize,
    scroll_position: usize,
    is_selected: bool,
) -> ProcessesTable<'_> {
    // -2 for border
    let visible_lines = layout_height - 2;
    let highlight_style = get_highlight_style(is_selected);

    let mut processes: Vec<_> = sys.processes().values().collect();
    let total_memory = sys.total_memory() as f64;
    processes.sort_by(|a, b| {
        let a_cpu_score = a.cpu_usage() as f64;
        let b_cpu_score = b.cpu_usage() as f64;

        let a_mem_score = (a.memory() as f64 / total_memory) * 100.0;
        let b_mem_score = (b.memory() as f64 / total_memory) * 100.0;

        let a_combined = a_cpu_score + a_mem_score;
        let b_combined = b_cpu_score + b_mem_score;

        b_combined
            .partial_cmp(&a_combined)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let header = Row::new(vec!["User", "PID", "CPU%", "MEM(MB)", "Time", "Command"])
        .style(Style::default().fg(Color::Gray));

    let rows: Vec<Row> = processes
        .iter()
        .skip(scroll_position)
        .take(visible_lines)
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

    let all_lines_count = processes.len();
    let max_scroll = all_lines_count.saturating_sub(visible_lines);
    let real_content_length = if visible_lines == all_lines_count {
        0
    } else {
        all_lines_count
    };

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Processes")
                .title_style(highlight_style.title)
                .borders(Borders::all())
                .border_style(highlight_style.border)
                .border_type(highlight_style.border_type),
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
        .column_spacing(1);

    ProcessesTable {
        chart: table,
        max_scroll,
        real_content_length,
    }
}
