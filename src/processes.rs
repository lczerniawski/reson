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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessColumn {
    User,
    PID,
    PPID,
    CPU,
    Memory,
    Time,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub fn create_processes_table(
    sys: &System,
    layout_height: usize,
    scroll_position: usize,
    is_selected: bool,
    sort_by: Option<(ProcessColumn, SortDirection)>,
) -> ProcessesTable<'_> {
    // -2 for border
    let visible_lines = layout_height - 2;
    let highlight_style = get_highlight_style(is_selected);

    let mut processes: Vec<_> = sys.processes().values().collect();
    let total_memory = sys.total_memory() as f64;

    match sort_by {
        Some((ProcessColumn::User, direction)) => {
            processes.sort_by(|a, b| {
                let a_user = a
                    .user_id()
                    .and_then(|id| sys.get_user_by_id(&id))
                    .map(|user| user.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let b_user = b
                    .user_id()
                    .and_then(|id| sys.get_user_by_id(&id))
                    .map(|user| user.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                match direction {
                    SortDirection::Ascending => a_user.cmp(&b_user),
                    SortDirection::Descending => b_user.cmp(&a_user),
                }
            });
        }
        Some((ProcessColumn::PID, direction)) => {
            processes.sort_by(|a, b| {
                let a_pid = a.pid();
                let b_pid = b.pid();

                match direction {
                    SortDirection::Ascending => a_pid.cmp(&b_pid),
                    SortDirection::Descending => b_pid.cmp(&a_pid),
                }
            });
        }
        Some((ProcessColumn::PPID, direction)) => {
            processes.sort_by(|a, b| {
                let a_ppid = a.parent();
                let b_ppid = b.parent();

                match direction {
                    SortDirection::Ascending => a_ppid.cmp(&b_ppid),
                    SortDirection::Descending => b_ppid.cmp(&a_ppid),
                }
            });
        }
        Some((ProcessColumn::CPU, direction)) => {
            processes.sort_by(|a, b| {
                let a_cpu = a.cpu_usage();
                let b_cpu = b.cpu_usage();
                match direction {
                    SortDirection::Ascending => a_cpu
                        .partial_cmp(&b_cpu)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    SortDirection::Descending => b_cpu
                        .partial_cmp(&a_cpu)
                        .unwrap_or(std::cmp::Ordering::Equal),
                }
            });
        }
        Some((ProcessColumn::Memory, direction)) => {
            processes.sort_by(|a, b| {
                let a_mem = a.memory();
                let b_mem = b.memory();

                match direction {
                    SortDirection::Ascending => a_mem.cmp(&b_mem),
                    SortDirection::Descending => b_mem.cmp(&a_mem),
                }
            });
        }
        Some((ProcessColumn::Time, direction)) => {
            processes.sort_by(|a, b| {
                let a_time = a.start_time();
                let b_time = b.start_time();

                match direction {
                    SortDirection::Ascending => a_time.cmp(&b_time),
                    SortDirection::Descending => b_time.cmp(&a_time),
                }
            });
        }
        Some((ProcessColumn::Command, direction)) => {
            processes.sort_by(|a, b| {
                let a_cmd = a.name();
                let b_cmd = b.name();

                match direction {
                    SortDirection::Ascending => a_cmd.cmp(&b_cmd),
                    SortDirection::Descending => b_cmd.cmp(&a_cmd),
                }
            });
        }
        None => {
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
        }
    }

    let headers = vec!["User", "PID", "PPID", "CPU%", "MEM(MB)", "Time", "Command"];

    let mut header_cells = Vec::new();
    for (i, &header_text) in headers.iter().enumerate() {
        let column = match i {
            0 => ProcessColumn::User,
            1 => ProcessColumn::PID,
            2 => ProcessColumn::PPID,
            3 => ProcessColumn::CPU,
            4 => ProcessColumn::Memory,
            5 => ProcessColumn::Time,
            6 => ProcessColumn::Command,
            _ => ProcessColumn::User, // Fallback
        };

        let header_with_indicator = match sort_by {
            Some((current_col, direction)) if current_col == column => match direction {
                SortDirection::Ascending => format!("{}↑", header_text),
                SortDirection::Descending => format!("{}↓", header_text),
            },
            _ => header_text.to_string(),
        };

        header_cells.push(header_with_indicator);
    }

    let header = Row::new(header_cells).style(Style::default().fg(Color::Gray));
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
                process
                    .parent()
                    .map_or("-".to_string(), |ppid| ppid.to_string()),
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
    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title(format!("Processes ({})", all_lines_count))
                .title_style(highlight_style.title)
                .borders(Borders::all())
                .border_style(highlight_style.border)
                .border_type(highlight_style.border_type),
        )
        .style(Style::default().fg(Color::Cyan))
        .widths(&[
            Constraint::Percentage(15), // User
            Constraint::Percentage(10), // PID
            Constraint::Percentage(10), // PPID
            Constraint::Percentage(10), // CPU%
            Constraint::Percentage(10), // MEM
            Constraint::Percentage(15), // Time
            Constraint::Percentage(40), // Command
        ])
        .column_spacing(1);

    ProcessesTable {
        chart: table,
        max_scroll,
    }
}
