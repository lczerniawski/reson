use color_eyre::{eyre::Ok, Result};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use reson::App;
use sysinfo::{System, SystemExt};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut sys = System::new_all();

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    App::new().run(&mut terminal, &mut sys)?;

    disable_raw_mode().unwrap();
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .unwrap();
    Ok(())
}
