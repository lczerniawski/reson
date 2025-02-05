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

use crate::layout::{get_horizontal_scrollbar, MainLayout, MemoryLayout};
use crate::memory::create_memory_gauges;
use crate::network::create_top_networks_widget;
use crate::processes::create_processes_table;
use crate::{
    cpu::create_cpu_barchart,
    layout::{prepare_layout, AppLayout},
};
use crate::{disk::create_top_disks_widget, layout::get_vertical_scrollbar};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    cpu_scrollbar_state: CustomScrollbarState,
    processes_scrollbar_state: CustomScrollbarState,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Exiting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "CPU")]
    Cpu,

    #[strum(to_string = "Processes")]
    Processes,

    #[strum(to_string = "Networks")]
    Networks,

    #[strum(to_string = "Disks")]
    Disks,
}

impl SelectedTab {
    fn next(&self) -> Self {
        match self {
            Self::Cpu => Self::Processes,
            Self::Processes => Self::Networks,
            Self::Networks => Self::Disks,
            Self::Disks => Self::Cpu,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Cpu => Self::Disks,
            Self::Processes => Self::Cpu,
            Self::Networks => Self::Processes,
            Self::Disks => Self::Networks,
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

struct CustomScrollbarState {
    state: ScrollbarState,
    pos: usize,
    max_scroll: usize,
    real_content_length: usize,
}

impl CustomScrollbarState {
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

#[derive(Debug)]
enum KeyboardMessage {
    KeyPress(KeyCode),
    Quit,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Running,
            selected_tab: SelectedTab::Cpu,
            cpu_scrollbar_state: CustomScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
                real_content_length: 0,
            },
            processes_scrollbar_state: CustomScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                max_scroll: 0,
                real_content_length: 0,
            },
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<KeyboardMessage>(10);

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

    fn handle_events(&mut self, message: &KeyboardMessage) {
        match message {
            KeyboardMessage::KeyPress(code) => match code {
                KeyCode::Char('l') | KeyCode::Right => self.scroll_right(),
                KeyCode::Char('h') | KeyCode::Left => self.scroll_left(),
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Tab => self.next_tab(),
                KeyCode::BackTab => self.prev_tab(),
                _ => {}
            },
            KeyboardMessage::Quit => self.quit(),
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
        }
    }

    fn scroll_up(&mut self) {
        if self.selected_tab.is_processes() {
            self.processes_scrollbar_state.scroll_prev();
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
        let app_layout = prepare_layout(frame);

        self.render_main_layout(frame, sys, &app_layout);
        self.render_footer(frame, app_layout.footer_area);
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
        render(frame, sys, &app_layout.main_layout);
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
        );

        frame.render_widget(processes_table.chart, *processes_layout);

        self.processes_scrollbar_state.set_values(
            processes_table.max_scroll,
            processes_table.real_content_length,
        );
        self.processes_scrollbar_state.current_pos_scroll_update();

        frame.render_stateful_widget(
            get_vertical_scrollbar(),
            *processes_layout,
            &mut self.processes_scrollbar_state.state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, footer_area: Rect) {
        let footer = Block::default()
            .title(
                "Press Tab for next tab, Shift + Tab for previous tab | Press ◄ ▼ ▲ ► or h j k l to scroll | Press q to quit",
            )
            .title_alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}

async fn read_input_events(tx: Sender<KeyboardMessage>) {
    loop {
        if let Result::Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                let msg = match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => KeyboardMessage::Quit,
                    KeyCode::Char('c') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            KeyboardMessage::Quit
                        } else {
                            KeyboardMessage::KeyPress(key.code)
                        }
                    }
                    code => KeyboardMessage::KeyPress(code),
                };
                if tx.send(msg).await.is_err() {
                    return;
                }
            }
        }
    }
}

fn render(frame: &mut Frame, sys: &System, main_layout: &MainLayout) {
    let disk_widget = create_top_disks_widget(sys);
    frame.render_widget(disk_widget, main_layout.disk_layout);

    let network_widget = create_top_networks_widget(sys);
    frame.render_widget(network_widget, main_layout.network_layout);
}
