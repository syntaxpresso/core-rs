#![allow(dead_code)]

use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::ui::form_trait::InputMode;

/// Generic text input widget with cursor
pub struct TextInput {
  pub value: String,
  pub cursor: usize,
  pub label: String,
}

impl TextInput {
  pub fn new(label: impl Into<String>, default_value: impl Into<String>) -> Self {
    let value = default_value.into();
    let cursor = value.len();
    Self { value, cursor, label: label.into() }
  }

  pub fn handle_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
      }
      KeyCode::Backspace => {
        if self.cursor > 0 {
          self.value.remove(self.cursor - 1);
          self.cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if self.cursor < self.value.len() {
          self.value.remove(self.cursor);
        }
      }
      KeyCode::Left => {
        if self.cursor > 0 {
          self.cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.cursor < self.value.len() {
          self.cursor += 1;
        }
      }
      KeyCode::Home => {
        self.cursor = 0;
      }
      KeyCode::End => {
        self.cursor = self.value.len();
      }
      _ => {}
    }
  }

  pub fn render(&self, frame: &mut Frame, area: Rect, is_focused: bool, input_mode: InputMode) {
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = if is_focused && input_mode == InputMode::Insert {
      format!("{} -- INSERT --", self.label)
    } else if is_focused {
      format!("{} -- NORMAL --", self.label)
    } else {
      self.label.clone()
    };

    let input = Paragraph::new(self.value.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));

    frame.render_widget(input, area);

    // Show cursor if focused and in insert mode
    if is_focused && input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.cursor as u16 + 1, area.y + 1));
    }
  }
}

/// Generic list selector widget
pub struct ListSelector {
  pub options: Vec<String>,
  pub state: ListState,
  pub label: String,
}

impl ListSelector {
  pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
    let mut state = ListState::default();
    state.select(Some(0));
    Self { options, state, label: label.into() }
  }

  pub fn handle_input(&mut self, key: KeyCode) {
    let len = self.options.len();
    let i = match self.state.selected() {
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
    self.state.select(Some(i));
  }

  pub fn selected_value(&self) -> Option<&String> {
    self.state.selected().and_then(|i| self.options.get(i))
  }

  pub fn selected_index(&self) -> Option<usize> {
    self.state.selected()
  }

  pub fn render(&mut self, frame: &mut Frame, area: Rect, is_focused: bool, input_mode: InputMode) {
    let items: Vec<ListItem> = self
      .options
      .iter()
      .enumerate()
      .map(|(i, opt)| {
        let is_selected = self.state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, opt))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = if is_focused && input_mode == InputMode::Insert {
      format!("{} -- INSERT --", self.label)
    } else if is_focused {
      format!("{} -- NORMAL --", self.label)
    } else {
      self.label.clone()
    };

    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.state);
  }
}

/// Generic button widget
pub struct Button {
  pub label: String,
  pub color: Color,
}

impl Button {
  pub fn new(label: impl Into<String>) -> Self {
    Self { label: label.into(), color: Color::Green }
  }

  pub fn with_color(mut self, color: Color) -> Self {
    self.color = color;
    self
  }

  pub fn render(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
    let style = if is_focused {
      Style::default().bg(self.color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(self.color)
    };

    let button = Paragraph::new(format!("[ {} ]", self.label))
      .alignment(Alignment::Center)
      .style(style)
      .block(Block::default().borders(Borders::empty()));

    frame.render_widget(button, area);
  }
}

/// Title bar widget
pub struct TitleBar {
  pub title: String,
}

impl TitleBar {
  pub fn new(title: impl Into<String>) -> Self {
    Self { title: title.into() }
  }

  pub fn render(&self, frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(self.title.as_str())
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }
}
