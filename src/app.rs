use std::{io::Stdout, thread, time::Duration};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    layout::{Alignment, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Tabs},
    Frame, Terminal,
};
use sysinfo::{System, SystemExt};

use crate::ui::{prepare_layout, render_summary_tab, AppLayout};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
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

    #[strum(to_string = "Processess Details")]
    ProcessessDetails,

    #[strum(to_string = "Network Details")]
    NetworkDetails,
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
        }
    }

    pub fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        while self.state == AppState::Running {
            sys.refresh_all();
            terminal.draw(|frame| self.draw(frame, sys))?;
            self.handle_events()?;
            thread::sleep(Duration::from_millis(250));
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                        KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                        KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn quit(&mut self) {
        self.state = AppState::Exiting;
    }

    fn draw(&self, frame: &mut Frame, sys: &System) {
        let app_layout = prepare_layout(frame);

        self.render_tab_headers(frame, app_layout.header_area);
        self.render_selected_tab(frame, sys, &app_layout);
        self.render_footer(frame, app_layout.footer_area);
    }

    fn render_selected_tab(&self, frame: &mut Frame, sys: &System, app_layout: &AppLayout) {
        match self.selected_tab {
            SelectedTab::Summary => render_summary_tab(frame, sys, &app_layout.summary_tab_layout),
            _ => (),
        }
    }

    fn render_tab_headers(&self, frame: &mut Frame, header_area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title).collect();
        let selected_tab_index = self.selected_tab as usize;
        let tabs_widget = Tabs::new(titles)
            .select(selected_tab_index)
            .divider(" ")
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(Style::default().fg(Color::Gray).bg(Color::Red));
        frame.render_widget(tabs_widget, header_area);
    }

    fn render_footer(&self, frame: &mut Frame, footer_area: Rect) {
        let footer = Block::default()
            .title("◄ ► or h l to change tab | Press q to quit")
            .title_alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}
