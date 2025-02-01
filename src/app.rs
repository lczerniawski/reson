use std::{
    io::Stdout,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    layout::{Alignment, Margin, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs},
    Frame, Terminal,
};
use sysinfo::{System, SystemExt};
use tokio::time::interval;

use crate::ui::{prepare_layout, render_cpu_details_tab, render_summary_tab, AppLayout};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    scrollbar_state: AppScrollbarState,
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
    #[strum(to_string = "Summary")]
    Summary,

    #[strum(to_string = "CPU Details")]
    CpuDetails,

    #[strum(to_string = "Disk Details")]
    DiskDetails,

    #[strum(to_string = "Processes Details")]
    ProcessessDetails,

    #[strum(to_string = "Network Details")]
    NetworkDetails,
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

impl SelectedTab {
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    fn previous(self) -> Self {
        let current_index = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn title(self) -> Line<'static> {
        Line::from(format!("  {self}  "))
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Running,
            selected_tab: SelectedTab::Summary,
            scrollbar_state: AppScrollbarState {
                state: ScrollbarState::new(0),
                pos: 0,
                content_length: 0,
                scale_modificator: 10,
            },
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        let (tx, rx) = mpsc::channel::<KeyboardMessage>();

        let input_handler = tokio::spawn(async move {
            read_input_events(tx).await;
        });

        let mut render_ticker = interval(Duration::from_millis(500));

        while self.state == AppState::Running {
            render_ticker.tick().await;

            sys.refresh_all();
            terminal.draw(|frame| self.draw(frame, sys))?;
            self.handle_events(&rx);
        }

        input_handler.abort();
        Ok(())
    }

    fn handle_events(&mut self, rx: &Receiver<KeyboardMessage>) {
        while let Result::Ok(message) = rx.try_recv() {
            match message {
                KeyboardMessage::KeyPress(code) => match code {
                    KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                    KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                    KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                    _ => {}
                },
                KeyboardMessage::Quit => self.quit(),
            }
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
        self.scrollbar_state.pos = 0;
        self.scrollbar_state.state = self.scrollbar_state.state.position(0);
    }

    fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
        self.scrollbar_state.pos = 0;
        self.scrollbar_state.state = self.scrollbar_state.state.position(0);
    }

    fn scroll_down(&mut self) {
        self.scrollbar_state.pos = self
            .scrollbar_state
            .pos
            .saturating_add(1)
            .clamp(0, self.scrollbar_state.content_length);
        self.scrollbar_state.state = self
            .scrollbar_state
            .state
            .position(self.scrollbar_state.pos * self.scrollbar_state.scale_modificator);
    }

    fn scroll_up(&mut self) {
        self.scrollbar_state.pos = self.scrollbar_state.pos.saturating_sub(1);
        self.scrollbar_state.state = self
            .scrollbar_state
            .state
            .position(self.scrollbar_state.pos * self.scrollbar_state.scale_modificator);
    }

    fn quit(&mut self) {
        self.state = AppState::Exiting;
    }

    fn draw(&mut self, frame: &mut Frame, sys: &System) {
        let app_layout = prepare_layout(frame);

        self.render_tab_headers(frame, app_layout.header_area);
        self.render_selected_tab(frame, sys, &app_layout);
        self.render_footer(frame, app_layout.footer_area);
    }

    fn render_selected_tab(&mut self, frame: &mut Frame, sys: &System, app_layout: &AppLayout) {
        match self.selected_tab {
            SelectedTab::Summary => render_summary_tab(frame, sys, &app_layout.summary_tab_layout),
            SelectedTab::CpuDetails => {
                let content_length = render_cpu_details_tab(
                    frame,
                    sys,
                    &app_layout.cpu_details_tab_layout,
                    self.scrollbar_state.pos,
                );

                self.scrollbar_state.content_length = content_length;
                self.scrollbar_state.state = self
                    .scrollbar_state
                    .state
                    .content_length(content_length * self.scrollbar_state.scale_modificator);
                frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                    app_layout
                        .cpu_details_tab_layout
                        .main_layout
                        .inner(&Margin {
                            vertical: 1,
                            horizontal: 0,
                        }),
                    &mut self.scrollbar_state.state,
                );
            }
            _ => (),
        }
    }

    fn render_tab_headers(&self, frame: &mut Frame, header_area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title).collect();
        let selected_tab_index = self.selected_tab as usize;
        let tabs_widget = Tabs::new(titles)
            .select(selected_tab_index)
            .divider("|")
            .style(Style::default().fg(Color::Blue))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Blue));
        frame.render_widget(tabs_widget, header_area);
    }

    fn render_footer(&self, frame: &mut Frame, footer_area: Rect) {
        let footer = Block::default()
            .title("◄ ► or h l to change tab | ▲ ▼ or j k to scroll | Press q to quit")
            .title_alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}

async fn read_input_events(tx: Sender<KeyboardMessage>) {
    let mut ticker = interval(Duration::from_millis(50));

    loop {
        ticker.tick().await;

        while let Result::Ok(true) = event::poll(Duration::ZERO) {
            if let Result::Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    let msg = match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => KeyboardMessage::Quit,
                        code => KeyboardMessage::KeyPress(code),
                    };
                    if tx.send(msg).is_err() {
                        return;
                    }
                }
            }
        }
    }
}
