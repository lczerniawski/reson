use std::{io::Stdout, time::Duration};
use strum::{Display, EnumIter, FromRepr};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use ratatui::widgets::ScrollbarOrientation;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::CrosstermBackend,
    widgets::{Block, Scrollbar, ScrollbarState},
    Frame, Terminal,
};
use sysinfo::{System, SystemExt};
use tokio::{sync::mpsc::Sender, time::interval};

use crate::{
    cpu::create_top_cpu_barchart,
    layout::{prepare_layout, render, AppLayout},
};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    cpu_scrollbar_state: AppScrollbarState,
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

pub struct AppScrollbarState {
    state: ScrollbarState,
    pos: usize,
    content_length: usize,
    scale_modificator: usize,
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
            cpu_scrollbar_state: AppScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                content_length: 0,
                scale_modificator: 15,
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
                _ => {}
            },
            KeyboardMessage::Quit => self.quit(),
        }
    }

    fn scroll_right(&mut self) {}

    fn scroll_left(&mut self) {}

    fn scroll_down(&mut self) {
        self.cpu_scrollbar_state.pos = self
            .cpu_scrollbar_state
            .pos
            .saturating_add(1)
            .clamp(0, self.cpu_scrollbar_state.content_length);

        self.cpu_scrollbar_state.state = self
            .cpu_scrollbar_state
            .state
            .position(self.cpu_scrollbar_state.pos * self.cpu_scrollbar_state.scale_modificator);
    }

    fn scroll_up(&mut self) {
        self.cpu_scrollbar_state.pos = self.cpu_scrollbar_state.pos.saturating_sub(1);
        self.cpu_scrollbar_state.state = self
            .cpu_scrollbar_state
            .state
            .position(self.cpu_scrollbar_state.pos * self.cpu_scrollbar_state.scale_modificator);
    }

    fn quit(&mut self) {
        self.state = AppState::Exiting;
    }

    fn draw(&mut self, frame: &mut Frame, sys: &System) {
        let app_layout = prepare_layout(frame);

        self.render_window(frame, sys, &app_layout);
        self.render_footer(frame, app_layout.footer_area);
    }

    fn render_window(&mut self, frame: &mut Frame, sys: &System, app_layout: &AppLayout) {
        self.render_cpu(frame, sys, app_layout);
        render(frame, sys, &app_layout.main_layout);
    }

    fn render_cpu(&mut self, frame: &mut Frame<'_>, sys: &System, app_layout: &AppLayout) {
        let (barchart, content_length) = create_top_cpu_barchart(
            sys,
            app_layout
                .main_layout
                .cpu_plus_memory_layout
                .cpu_layout
                .width,
            self.cpu_scrollbar_state.pos,
        );
        frame.render_widget(
            barchart,
            app_layout.main_layout.cpu_plus_memory_layout.cpu_layout,
        );

        self.cpu_scrollbar_state.content_length = content_length;
        self.cpu_scrollbar_state.state = self
            .cpu_scrollbar_state
            .state
            .content_length(content_length * self.cpu_scrollbar_state.scale_modificator);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
            app_layout.main_layout.cpu_plus_memory_layout.cpu_layout,
            &mut self.cpu_scrollbar_state.state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, footer_area: Rect) {
        let footer = Block::default()
            .title(
                "Press Tab to change between tabs | Press ◄ ▼ ▲ ► or h j k l to scroll | Press q to quit",
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
