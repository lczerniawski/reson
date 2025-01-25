use std::{io::Stdout, thread, time::Duration};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{prelude::CrosstermBackend, widgets::Block, Frame, Terminal};
use sysinfo::{System, SystemExt};

use crate::{
    cpu::create_top_cpu_barchart, disk::create_top_disks_barchart, memory::create_memory_gauges,
    network::create_top_networks_widget, processes::create_top_processes_table, ui::prepare_layout,
};

pub struct App {
    should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_exit: false }
    }

    pub fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        sys: &mut System,
    ) -> Result<()> {
        while !self.should_exit {
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
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.should_exit = true;
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame, sys: &System) {
        let app_layout = prepare_layout(frame);

        let top_cpu_barchart = create_top_cpu_barchart(sys);
        frame.render_widget(top_cpu_barchart, app_layout.inner_layout[0]);

        let memory_gauges = create_memory_gauges(sys);
        frame.render_widget(memory_gauges.main_memory_gauge, app_layout.memory_layout[0]);
        frame.render_widget(memory_gauges.swap_gauge, app_layout.memory_layout[1]);

        let top_processes_table = create_top_processes_table(sys);
        frame.render_widget(top_processes_table, app_layout.outer_layout[1]);

        let disk_barchart = create_top_disks_barchart(sys);
        frame.render_widget(disk_barchart, app_layout.outer_layout[2]);

        let network_widget = create_top_networks_widget(sys);
        frame.render_widget(network_widget, app_layout.outer_layout[3]);

        let exit_message = Block::default().title("Click 'q' to exit.");
        frame.render_widget(exit_message, app_layout.outer_layout[4]);
    }
}
