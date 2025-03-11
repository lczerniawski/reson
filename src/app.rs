use std::{io::Stdout, time::Duration};
use strum::{Display, EnumIter, FromRepr};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use ratatui::{
    layout::{Alignment, Rect},
    prelude::CrosstermBackend,
    widgets::{Block, ScrollbarState},
    Frame, Terminal,
};
use sysinfo::{System, SystemExt};
use tokio::{sync::mpsc::Sender, time::interval};

use crate::memory::create_memory_gauges;
use crate::network::create_networks_widget;
use crate::processes::create_processes_table;
use crate::{
    cpu::create_cpu_barchart,
    layout::{is_within_rect, prepare_layout, AppLayout},
};
use crate::{disk::create_disks_widget, layout::get_vertical_scrollbar};
use crate::{
    layout::{get_horizontal_scrollbar, MemoryLayout},
    processes::{ProcessColumn, SortDirection},
};

pub struct App {
    state: AppState,
    layout_clone: AppLayout,
    selected_tab: SelectedTab,
    cpu_scrollbar_state: HorizontalScrollbarState,
    processes_scrollbar_state: VerticalScrollbarState,
    process_sort_state: Option<(ProcessColumn, SortDirection)>,
    disks_scrollbar_state: VerticalScrollbarState,
    networks_scrollbar_state: VerticalScrollbarState,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Exiting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[strum(to_string = "CPU")]
    Cpu,

    #[strum(to_string = "Processes")]
    Processes,

    #[strum(to_string = "Disks")]
    Disks,

    #[strum(to_string = "Networks")]
    Networks,

    #[default]
    #[strum(to_string = "None")]
    None,
}

impl SelectedTab {
    fn next(&self) -> Self {
        match self {
            Self::Cpu => Self::Processes,
            Self::Processes => Self::Disks,
            Self::Disks => Self::Networks,
            Self::Networks => Self::None,
            Self::None => Self::Cpu,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::None => Self::Networks,
            Self::Cpu => Self::None,
            Self::Processes => Self::Cpu,
            Self::Disks => Self::Processes,
            Self::Networks => Self::Disks,
        }
    }

    fn is_cpu(&self) -> bool {
        matches!(self, SelectedTab::Cpu)
    }

    fn is_processes(&self) -> bool {
        matches!(self, SelectedTab::Processes)
    }

    fn is_network(&self) -> bool {
        matches!(self, SelectedTab::Networks)
    }

    fn is_disks(&self) -> bool {
        matches!(self, SelectedTab::Disks)
    }
}

struct HorizontalScrollbarState {
    state: ScrollbarState,
    pos: usize,
    max_scroll: usize,
    real_content_length: usize,
}

impl HorizontalScrollbarState {
    fn scroll_next(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.pos = self.pos.saturating_add(1).clamp(0, self.max_scroll);
        self.current_pos_scroll_update();
    }

    fn scroll_prev(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.pos = self.pos.saturating_sub(1);
        self.current_pos_scroll_update();
    }

    fn set_values(&mut self, max_scroll: usize, real_content_length: usize) {
        self.max_scroll = max_scroll;
        self.real_content_length = real_content_length;

        self.state = self.state.content_length(real_content_length);
    }

    fn current_pos_scroll_update(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.state = self
            .state
            .position(self.pos * (self.real_content_length / self.max_scroll));
    }
}

struct VerticalScrollbarState {
    state: ScrollbarState,
    pos: usize,
    max_scroll: usize,
}

impl VerticalScrollbarState {
    fn scroll_next(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.pos = self.pos.saturating_add(1).clamp(0, self.max_scroll);
        self.current_pos_scroll_update();
    }

    fn scroll_prev(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.pos = self.pos.saturating_sub(1);
        self.current_pos_scroll_update();
    }

    fn set_values(&mut self, max_scroll: usize) {
        self.max_scroll = max_scroll;
        self.state = self.state.content_length(max_scroll);
    }

    fn current_pos_scroll_update(&mut self) {
        if self.max_scroll == 0 {
            return;
        }

        self.state = self.state.position(self.pos);
    }
}

#[derive(Debug)]
enum InputMessage {
    KeyPress(KeyCode),
    MouseScroll { direction: MouseScrollDirection },
    MouseMoved { position: (u16, u16) },
    Quit,
}

#[derive(Debug)]
enum MouseScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Running,
            layout_clone: AppLayout::empty(),
            selected_tab: SelectedTab::None,
            cpu_scrollbar_state: HorizontalScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
                real_content_length: 0,
            },
            processes_scrollbar_state: VerticalScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
            },
            process_sort_state: None,
            disks_scrollbar_state: VerticalScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
            },
            networks_scrollbar_state: VerticalScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
            },
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<InputMessage>(10);

        let input_handler = tokio::spawn(read_input_events(tx.clone()));

        let mut draw_ticker = interval(Duration::from_millis(150));
        let mut refresh_ticker = interval(Duration::from_millis(1000));
        while self.state == AppState::Running {
            tokio::select! {
                _ = refresh_ticker.tick() => {
                    sys.refresh_all();
                }
                _ = draw_ticker.tick() => {
                    terminal.draw(|frame| self.draw(frame, sys))?;
                }
                Some(message) = rx.recv() => {
                    self.handle_events(&message);
                }
            }
        }

        input_handler.abort();
        Ok(())
    }

    fn handle_events(&mut self, message: &InputMessage) {
        match message {
            InputMessage::KeyPress(code) => match code {
                KeyCode::Char('l') | KeyCode::Right => self.scroll_right(),
                KeyCode::Char('h') | KeyCode::Left => self.scroll_left(),
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Tab => self.next_tab(),
                KeyCode::BackTab => self.prev_tab(),
                KeyCode::Char('1') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::User)
                }
                KeyCode::Char('2') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::PID)
                }
                KeyCode::Char('3') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::PPID)
                }
                KeyCode::Char('4') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::CPU)
                }
                KeyCode::Char('5') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::Memory)
                }
                KeyCode::Char('6') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::Time)
                }
                KeyCode::Char('7') if self.selected_tab.is_processes() => {
                    self.toggle_sort_column(ProcessColumn::Command)
                }
                // Reset sorting if 'r' is pressed
                KeyCode::Char('r') if self.selected_tab.is_processes() => {
                    self.process_sort_state = None;
                }
                _ => {}
            },
            InputMessage::MouseScroll { direction } => match direction {
                MouseScrollDirection::Up => self.scroll_up(),
                MouseScrollDirection::Down => self.scroll_down(),
                MouseScrollDirection::Left => self.scroll_left(),
                MouseScrollDirection::Right => self.scroll_right(),
            },
            InputMessage::MouseMoved { position } => self.handle_mouse_moved(*position),
            InputMessage::Quit => self.quit(),
        }
    }

    fn toggle_sort_column(&mut self, column: ProcessColumn) {
        match &self.process_sort_state {
            Some((current_column, direction)) if *current_column == column => match direction {
                SortDirection::Ascending => {
                    self.process_sort_state = Some((column, SortDirection::Descending));
                }
                SortDirection::Descending => {
                    self.process_sort_state = None;
                }
            },
            _ => {
                self.process_sort_state = Some((column, SortDirection::Ascending));
            }
        }
    }

    fn handle_mouse_moved(&mut self, position: (u16, u16)) {
        if is_within_rect(
            position,
            &self
                .layout_clone
                .main_layout
                .cpu_plus_memory_layout
                .cpu_layout,
        ) {
            self.selected_tab = SelectedTab::Cpu;
        } else if is_within_rect(position, &self.layout_clone.main_layout.processes_layout) {
            self.selected_tab = SelectedTab::Processes;
        } else if is_within_rect(position, &self.layout_clone.main_layout.disk_layout) {
            self.selected_tab = SelectedTab::Disks;
        } else if is_within_rect(position, &self.layout_clone.main_layout.network_layout) {
            self.selected_tab = SelectedTab::Networks;
        } else {
            self.selected_tab = SelectedTab::None;
        }
    }

    fn scroll_right(&mut self) {
        if self.selected_tab.is_cpu() {
            self.cpu_scrollbar_state.scroll_next();
        }
    }

    fn scroll_left(&mut self) {
        if self.selected_tab.is_cpu() {
            self.cpu_scrollbar_state.scroll_prev();
        }
    }

    fn scroll_down(&mut self) {
        if self.selected_tab.is_processes() {
            self.processes_scrollbar_state.scroll_next();
            return;
        }

        if self.selected_tab.is_disks() {
            self.disks_scrollbar_state.scroll_next();
            return;
        }

        if self.selected_tab.is_network() {
            self.networks_scrollbar_state.scroll_next();
            return;
        }
    }

    fn scroll_up(&mut self) {
        if self.selected_tab.is_processes() {
            self.processes_scrollbar_state.scroll_prev();
            return;
        }

        if self.selected_tab.is_disks() {
            self.disks_scrollbar_state.scroll_prev();
            return;
        }

        if self.selected_tab.is_network() {
            self.networks_scrollbar_state.scroll_prev();
            return;
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn prev_tab(&mut self) {
        self.selected_tab = self.selected_tab.prev();
    }

    fn quit(&mut self) {
        self.state = AppState::Exiting;
    }

    fn draw(&mut self, frame: &mut Frame, sys: &System) {
        let layout = prepare_layout(frame);
        self.layout_clone = layout.clone();

        self.render_main_layout(frame, sys, &layout);
        self.render_footer(frame, &layout.footer_area);
    }

    fn render_main_layout(&mut self, frame: &mut Frame, sys: &System, app_layout: &AppLayout) {
        self.render_cpu(
            frame,
            sys,
            &app_layout.main_layout.cpu_plus_memory_layout.cpu_layout,
        );
        self.render_memory_gauges(
            frame,
            sys,
            &app_layout.main_layout.cpu_plus_memory_layout.memory_layout,
        );
        self.render_processes(frame, sys, &app_layout.main_layout.processes_layout);
        self.render_disks(frame, sys, &app_layout.main_layout.disk_layout);
        self.render_networks(frame, sys, &app_layout.main_layout.network_layout);
    }

    fn render_cpu(&mut self, frame: &mut Frame, sys: &System, cpu_layout: &Rect) {
        let is_selected = self.selected_tab.is_cpu();

        let cpu_barchart = create_cpu_barchart(
            sys,
            cpu_layout.width.into(),
            self.cpu_scrollbar_state.pos,
            is_selected,
        );

        frame.render_widget(cpu_barchart.chart, *cpu_layout);

        // When window is growing and user is at the end of the CPUs we need to remove pos in order to keep on displaying more
        // of the CPUs from left side
        if self.cpu_scrollbar_state.pos == self.cpu_scrollbar_state.max_scroll
            && cpu_barchart.max_scroll < self.cpu_scrollbar_state.max_scroll
        {
            self.cpu_scrollbar_state.pos = self.cpu_scrollbar_state.pos.saturating_sub(1);
        }

        self.cpu_scrollbar_state
            .set_values(cpu_barchart.max_scroll, cpu_barchart.real_content_length);
        self.cpu_scrollbar_state.current_pos_scroll_update();

        frame.render_stateful_widget(
            get_horizontal_scrollbar(),
            *cpu_layout,
            &mut self.cpu_scrollbar_state.state,
        );
    }

    fn render_memory_gauges(&self, frame: &mut Frame, sys: &System, memory_layout: &MemoryLayout) {
        let memory_gauges = create_memory_gauges(sys);
        frame.render_widget(memory_gauges.ram_gauge, memory_layout.ram_layout);
        frame.render_widget(memory_gauges.swap_gauge, memory_layout.swap_layout);
    }

    fn render_processes(&mut self, frame: &mut Frame, sys: &System, processes_layout: &Rect) {
        let is_selected = self.selected_tab.is_processes();
        let processes_table = create_processes_table(
            sys,
            processes_layout.height.into(),
            self.processes_scrollbar_state.pos,
            is_selected,
            self.process_sort_state,
        );

        frame.render_widget(processes_table.chart, *processes_layout);

        self.processes_scrollbar_state
            .set_values(processes_table.max_scroll);
        self.processes_scrollbar_state.current_pos_scroll_update();

        frame.render_stateful_widget(
            get_vertical_scrollbar(),
            *processes_layout,
            &mut self.processes_scrollbar_state.state,
        );
    }

    fn render_disks(&mut self, frame: &mut Frame, sys: &System, disks_layout: &Rect) {
        let is_selected = self.selected_tab.is_disks();
        let disk_widget = create_disks_widget(
            sys,
            disks_layout.height.into(),
            self.disks_scrollbar_state.pos,
            is_selected,
        );
        frame.render_widget(disk_widget.chart, *disks_layout);

        self.disks_scrollbar_state
            .set_values(disk_widget.max_scroll);
        self.processes_scrollbar_state.current_pos_scroll_update();

        frame.render_stateful_widget(
            get_vertical_scrollbar(),
            *disks_layout,
            &mut self.disks_scrollbar_state.state,
        );
    }

    fn render_networks(&mut self, frame: &mut Frame, sys: &System, network_layout: &Rect) {
        let is_selected = self.selected_tab.is_network();
        let network_widget = create_networks_widget(
            sys,
            network_layout.height.into(),
            self.networks_scrollbar_state.pos,
            is_selected,
        );
        frame.render_widget(network_widget.chart, *network_layout);

        self.networks_scrollbar_state
            .set_values(network_widget.max_scroll);
        self.networks_scrollbar_state.current_pos_scroll_update();

        frame.render_stateful_widget(
            get_vertical_scrollbar(),
            *network_layout,
            &mut self.networks_scrollbar_state.state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, footer_area: &Rect) {
        let footer_text = if self.selected_tab.is_processes() {
            "1-7: Sort columns | r: Reset sort | Tab: Next tab | h/j/k/l: Scroll | q: Quit"
        } else {
            // Regular footer text
            "Tab: Next tab | h/j/k/l: Scroll | q: Quit"
        };

        let footer = Block::default()
            .title(footer_text)
            .title_alignment(Alignment::Center);
        frame.render_widget(footer, *footer_area);
    }
}

async fn read_input_events(tx: Sender<InputMessage>) {
    loop {
        if let Result::Ok(event) = event::read() {
            match event {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        let msg = match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => InputMessage::Quit,
                            KeyCode::Char('c') => {
                                if key.modifiers == KeyModifiers::CONTROL {
                                    InputMessage::Quit
                                } else {
                                    InputMessage::KeyPress(key.code)
                                }
                            }
                            code => InputMessage::KeyPress(code),
                        };
                        if tx.send(msg).await.is_err() {
                            return;
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    let msg = match mouse.kind {
                        event::MouseEventKind::ScrollDown => InputMessage::MouseScroll {
                            direction: MouseScrollDirection::Down,
                        },
                        event::MouseEventKind::ScrollUp => InputMessage::MouseScroll {
                            direction: MouseScrollDirection::Up,
                        },
                        event::MouseEventKind::ScrollLeft => InputMessage::MouseScroll {
                            direction: MouseScrollDirection::Left,
                        },
                        event::MouseEventKind::ScrollRight => InputMessage::MouseScroll {
                            direction: MouseScrollDirection::Right,
                        },
                        event::MouseEventKind::Moved => InputMessage::MouseMoved {
                            position: (mouse.column, mouse.row),
                        },
                        _ => continue,
                    };
                    if tx.send(msg).await.is_err() {
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}
