#![allow(dead_code)]

use crossterm::event::KeyCode;
use ratatui::{Frame, widgets::ListState};

/// Vim-style input mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
  Normal,
  Insert,
}

/// Helper for handling Esc-Esc quit pattern
pub struct EscapeHandler {
  pub pressed_once: bool,
}

impl EscapeHandler {
  pub fn new() -> Self {
    Self { pressed_once: false }
  }

  /// Returns true if should quit
  pub fn handle_escape(&mut self, input_mode: InputMode) -> (bool, InputMode) {
    match input_mode {
      InputMode::Insert => {
        // In Insert mode: Esc exits to Normal mode
        (false, InputMode::Normal)
      }
      InputMode::Normal => {
        // In Normal mode: Esc twice to quit
        if self.pressed_once {
          (true, InputMode::Normal) // Quit
        } else {
          self.pressed_once = true;
          (false, InputMode::Normal)
        }
      }
    }
  }

  pub fn reset(&mut self) {
    self.pressed_once = false;
  }
}

impl Default for EscapeHandler {
  fn default() -> Self {
    Self::new()
  }
}

/// Common form state that all forms should embed
pub struct FormState {
  pub input_mode: InputMode,
  pub escape_handler: EscapeHandler,
  pub should_quit: bool,
  pub error_message: Option<String>,
}

impl FormState {
  pub fn new() -> Self {
    Self {
      input_mode: InputMode::Insert, // Start in Insert mode
      escape_handler: EscapeHandler::new(),
      should_quit: false,
      error_message: None,
    }
  }
}

impl Default for FormState {
  fn default() -> Self {
    Self::new()
  }
}

/// Trait for common form behavior
/// All TUI forms should implement this trait to get shared functionality
pub trait FormBehavior {
  // ===== Required methods (must be implemented by each form) =====

  /// Get reference to the common form state
  fn form_state(&self) -> &FormState;

  /// Get mutable reference to the common form state
  fn form_state_mut(&mut self) -> &mut FormState;

  /// Move focus to the next field (form-specific)
  fn focus_next(&mut self);

  /// Move focus to the previous field (form-specific)
  fn focus_prev(&mut self);

  /// Called when entering insert mode (form-specific setup)
  /// The key parameter is either 'i' or 'a' to allow different behavior
  fn on_enter_insert_mode(&mut self, key: KeyCode);

  /// Called when Enter is pressed in Normal mode (form-specific)
  fn on_enter_pressed(&mut self);

  /// Handle field-specific input in Insert mode (form-specific)
  fn handle_field_insert(&mut self, key: KeyCode);

  /// Render the form (form-specific)
  fn render(&mut self, frame: &mut Frame);

  // ===== Default implementations (inherited by all forms) =====

  /// Get the current input mode
  fn input_mode(&self) -> InputMode {
    self.form_state().input_mode
  }

  /// Set the input mode
  fn set_input_mode(&mut self, mode: InputMode) {
    self.form_state_mut().input_mode = mode;
  }

  /// Get mutable reference to escape handler
  fn escape_handler_mut(&mut self) -> &mut EscapeHandler {
    &mut self.form_state_mut().escape_handler
  }

  /// Handle input in Normal mode - DEFAULT IMPLEMENTATION
  /// Common navigation: j/k/Tab for focus, i/a for insert mode, Enter to confirm
  /// Forms can override this if they need different behavior
  fn handle_normal_mode(&mut self, key: KeyCode) {
    match key {
      // Vim-style navigation
      KeyCode::Char('j') | KeyCode::Tab => self.focus_next(),
      KeyCode::Char('k') | KeyCode::BackTab => self.focus_prev(),
      // Enter insert mode
      KeyCode::Char('i') | KeyCode::Char('a') => {
        self.on_enter_insert_mode(key);
        self.set_input_mode(InputMode::Insert);
      }
      // Confirm action
      KeyCode::Enter => self.on_enter_pressed(),
      // Let forms handle other keys (like Up/Down for lists)
      _ => {}
    }
  }

  /// Handle input in Insert mode - DEFAULT IMPLEMENTATION
  /// Delegates to field-specific input handling
  /// Forms can override this if they need different behavior
  fn handle_insert_mode(&mut self, key: KeyCode) {
    self.handle_field_insert(key);
  }

  /// Main input handler - handles Esc, quit, and mode dispatching
  /// This is the "inherited" behavior that all forms get automatically
  fn handle_input(&mut self, key: KeyCode) -> bool {
    // Handle Esc key
    if let KeyCode::Esc = key {
      let mode = self.input_mode();
      let (should_quit, new_mode) = self.escape_handler_mut().handle_escape(mode);
      self.set_input_mode(new_mode);
      if should_quit {
        return true;
      }
      return false;
    }
    // Handle C-c
    if let KeyCode::Char('c') = key
      && self.input_mode() == InputMode::Insert
    {
      self.set_input_mode(InputMode::Normal);
      self.escape_handler_mut().reset();
      return false;
    }
    // Reset escape handler on any other key
    self.escape_handler_mut().reset();
    // Dispatch to mode-specific handler
    match self.input_mode() {
      InputMode::Normal => self.handle_normal_mode(key),
      InputMode::Insert => self.handle_insert_mode(key),
    }
    false // Don't quit
  }

  /// Generate a title with mode indicator
  fn generate_title(&self, title: &str, is_focused: bool) -> String {
    if is_focused && self.input_mode() == InputMode::Insert {
      format!("{} -- INSERT --", title)
    } else if is_focused {
      format!("{} -- NORMAL --", title)
    } else {
      title.to_string()
    }
  }

  /// Navigate through a list with up/down keys
  fn navigate_list(&mut self, key: &KeyCode, state: &mut ListState, len: usize) {
    let i = match state.selected() {
      Some(i) => match key {
        KeyCode::Up | KeyCode::Char('k') => {
          if i == 0 {
            len - 1
          } else {
            i - 1
          }
        }
        KeyCode::Down | KeyCode::Char('j') => {
          if i >= len - 1 {
            0
          } else {
            i + 1
          }
        }
        _ => i,
      },
      None => 0,
    };
    state.select(Some(i));
  }

  /// Handle standard navigation keys in Normal mode
  /// Returns true if the key was handled
  fn handle_standard_navigation(
    &mut self,
    key: KeyCode,
    on_next: impl FnOnce(&mut Self),
    on_prev: impl FnOnce(&mut Self),
  ) -> bool {
    match key {
      KeyCode::Char('j') | KeyCode::Tab => {
        on_next(self);
        true
      }
      KeyCode::Char('k') | KeyCode::BackTab => {
        on_prev(self);
        true
      }
      KeyCode::Char('i') | KeyCode::Char('a') => {
        self.set_input_mode(InputMode::Insert);
        true
      }
      _ => false,
    }
  }
}

pub mod helpers {
  use crossterm::event::KeyCode;
  use ratatui::widgets::ListState;
  use serde::Serialize;

  use super::{FormState, InputMode};
  use crate::responses::response::Response;

  /// Output a Response<T> as JSON and exit the process
  ///
  /// This helper function standardizes how UI forms handle command responses:
  /// - Prints JSON to stdout for the frontend to parse
  /// - Exits with code 0 on success, 1 on error
  /// - Updates form state error message on failure
  ///
  /// # Arguments
  /// * `response` - The Response<T> object from a command
  /// * `form_state` - Mutable reference to the form's state
  pub fn output_response_and_exit<T: Serialize>(response: Response<T>, form_state: &mut FormState) {
    // Output JSON to stdout
    if let Ok(json) = response.to_json() {
      println!("{}", json);
    }

    // Handle success vs error
    if response.is_success() {
      form_state.should_quit = true;
      std::process::exit(0);
    } else {
      // Set error message and exit with error code
      form_state.error_message = response.get_error().cloned();
      std::process::exit(1);
    }
  }

  /// Navigate a list - standalone function
  pub fn navigate_list_static(key: &KeyCode, state: &mut ListState, len: usize) {
    let i = match state.selected() {
      Some(i) => match key {
        KeyCode::Up => {
          if i == 0 {
            len - 1
          } else {
            i - 1
          }
        }
        KeyCode::Down => {
          if i >= len - 1 {
            0
          } else {
            i + 1
          }
        }
        _ => i,
      },
      None => 0,
    };
    state.select(Some(i));
  }

  /// Handle text input (any characters allowed)
  pub fn handle_text_input(
    key: KeyCode,
    text: &mut String,
    cursor: &mut usize,
    mode: &mut InputMode,
  ) {
    match key {
      KeyCode::Char(c) => {
        text.insert(*cursor, c);
        *cursor += 1;
      }
      KeyCode::Backspace => {
        if *cursor > 0 {
          text.remove(*cursor - 1);
          *cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if *cursor < text.len() {
          text.remove(*cursor);
        }
      }
      KeyCode::Left => {
        if *cursor > 0 {
          *cursor -= 1;
        }
      }
      KeyCode::Right => {
        if *cursor < text.len() {
          *cursor += 1;
        }
      }
      KeyCode::Home => {
        *cursor = 0;
      }
      KeyCode::End => {
        *cursor = text.len();
      }
      KeyCode::Enter => {
        *mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  /// Handle numeric input (only digits allowed)
  pub fn handle_numeric_input(
    key: KeyCode,
    text: &mut String,
    cursor: &mut usize,
    mode: &mut InputMode,
  ) {
    match key {
      KeyCode::Char(c) if c.is_ascii_digit() => {
        text.insert(*cursor, c);
        *cursor += 1;
      }
      KeyCode::Backspace => {
        if *cursor > 0 {
          text.remove(*cursor - 1);
          *cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if *cursor < text.len() {
          text.remove(*cursor);
        }
      }
      KeyCode::Left => {
        if *cursor > 0 {
          *cursor -= 1;
        }
      }
      KeyCode::Right => {
        if *cursor < text.len() {
          *cursor += 1;
        }
      }
      KeyCode::Home => {
        *cursor = 0;
      }
      KeyCode::End => {
        *cursor = text.len();
      }
      KeyCode::Enter => {
        *mode = InputMode::Normal;
      }
      _ => {}
    }
  }
}

/// Button rendering helpers for consistent UI across all forms
pub mod button_helpers {
  use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
  };

  /// Button type for rendering
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub enum ButtonType {
    Back,
    Confirm,
    Next,
  }

  /// Render a back button
  pub fn render_back_button(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    back_pressed_once: bool,
  ) {
    let color = if back_pressed_once { Color::Red } else { Color::Yellow };
    let text = if back_pressed_once { "Press again to go back" } else { "Back" };
    let style = if is_focused {
      Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(color)
    };
    let button = Paragraph::new(format!("[ {} ]", text))
      .alignment(Alignment::Center)
      .style(style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(button, area);
  }

  /// Render a confirm button
  pub fn render_confirm_button(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    escape_pressed_once: bool,
  ) {
    let color = if escape_pressed_once { Color::Red } else { Color::Green };
    let text =
      if escape_pressed_once { "Press esc again to close or any key to return" } else { "Confirm" };
    let style = if is_focused {
      Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(color)
    };
    let button = Paragraph::new(format!("[ {} ]", text))
      .alignment(Alignment::Center)
      .style(style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(button, area);
  }

  /// Render a next button
  pub fn render_next_button(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    escape_pressed_once: bool,
  ) {
    let color = if escape_pressed_once { Color::Red } else { Color::Green };
    let text =
      if escape_pressed_once { "Press esc again to close or any key to return" } else { "Next ->" };
    let style = if is_focused {
      Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(color)
    };
    let button = Paragraph::new(format!("[ {} ]", text))
      .alignment(Alignment::Center)
      .style(style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(button, area);
  }

  /// Render a two-button layout (Back + Confirm/Next)
  pub fn render_two_button_layout(
    frame: &mut Frame,
    area: Rect,
    back_focused: bool,
    right_focused: bool,
    back_pressed_once: bool,
    escape_pressed_once: bool,
    right_button_type: ButtonType,
  ) {
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(area);

    // Render back button
    render_back_button(frame, chunks[0], back_focused, back_pressed_once);

    // Render right button based on type
    match right_button_type {
      ButtonType::Confirm => {
        render_confirm_button(frame, chunks[1], right_focused, escape_pressed_once)
      }
      ButtonType::Next => render_next_button(frame, chunks[1], right_focused, escape_pressed_once),
      ButtonType::Back => {} // Not used for right button
    }
  }

  /// Render a single button (usually Confirm or Next)
  pub fn render_single_button(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    escape_pressed_once: bool,
    button_type: ButtonType,
  ) {
    match button_type {
      ButtonType::Confirm => render_confirm_button(frame, area, is_focused, escape_pressed_once),
      ButtonType::Next => render_next_button(frame, area, is_focused, escape_pressed_once),
      ButtonType::Back => render_back_button(frame, area, is_focused, false),
    }
  }
}
