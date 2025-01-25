use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub outer_layout: Rc<[Rect]>,
    pub inner_layout: Rc<[Rect]>,
    pub memory_layout: Rc<[Rect]>,
}

pub fn prepare_layout(f: &mut ratatui::Frame<'_>) -> AppLayout {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(30), // CPU + Memory
            Constraint::Percentage(30), // Top Processes
            Constraint::Percentage(18), // Disk
            Constraint::Percentage(20), // Network
            Constraint::Percentage(2),  // Exit Message
        ])
        .split(f.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer_layout[0]);

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_layout[1]);

    AppLayout {
        outer_layout,
        inner_layout,
        memory_layout,
    }
}
