use crossterm::{
  event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
  },
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io};

use super::form_trait::FormBehavior;

pub fn run_ui_command<F: FormBehavior>(mut form: F) -> Result<(), Box<dyn Error>> {
  // Setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // Run the app
  let res = run_app(&mut terminal, &mut form);

  // Restore terminal
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{err:?}");
  }

  Ok(())
}

fn run_app<B: ratatui::backend::Backend, F: FormBehavior>(
  terminal: &mut Terminal<B>,
  app: &mut F,
) -> io::Result<()> {
  loop {
    terminal.draw(|f| app.render(f))?;

    if let Event::Key(key) = event::read()?
      && key.kind == KeyEventKind::Press
    {
      if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if app.handle_input(KeyCode::Char('c')) {
          return Ok(());
        }
        continue;
      }

      if app.handle_input(key.code) {
        return Ok(());
      }
    }
  }
}
