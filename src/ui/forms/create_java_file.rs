use clap::ValueEnum;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::get_all_packages_command;
use crate::commands::services::create_java_file_service;
use crate::common::types::{
  java_file_type::JavaFileType, java_source_directory_type::JavaSourceDirectoryType,
};
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, helpers};

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  FileType,
  FileName,
  PackageName,
  ConfirmButton,
}

/// Main form state for creating a Java file
pub struct CreateJavaFileForm {
  // Common form state (embedded)
  state: FormState,

  // Field values (using enums for type safety!)
  file_type: JavaFileType,
  file_name: String,
  package_name: String,

  // File type state
  file_type_state: ListState,

  // Package autocomplete
  package_list: Vec<String>,                  // All available packages
  filtered_packages: Vec<String>,             // Filtered packages based on input
  autocomplete_selected_index: Option<usize>, // Selected index in filtered list
  autocomplete_scroll_offset: usize, // Scroll offset for autocomplete (which item is at top)
  show_autocomplete: bool,           // Whether to show autocomplete dropdown

  // Text input states
  file_name_cursor: usize,
  package_name_cursor: usize,

  // Focus management
  focused_field: FocusedField,

  // Integration with syntaxpresso-core
  cwd: PathBuf,
}

impl CreateJavaFileForm {
  pub fn new(cwd: PathBuf) -> Self {
    // Fetch packages directly from service
    let package_list = Self::fetch_packages(&cwd).unwrap_or_else(|_| {
      vec![
        "com.example".to_string(),
        "com.example.model".to_string(),
        "com.example.service".to_string(),
        "com.example.controller".to_string(),
      ]
    });

    let mut file_type_state = ListState::default();
    file_type_state.select(Some(0));

    let default_package =
      package_list.first().cloned().unwrap_or_else(|| "com.example".to_string());

    Self {
      state: FormState::new(),
      // Use enum variants directly!
      file_type: JavaFileType::Class,
      file_name: "NewFile".to_string(),
      package_name: default_package.clone(),
      file_type_state,
      package_list,
      filtered_packages: Vec::new(),
      autocomplete_selected_index: None,
      autocomplete_scroll_offset: 0,
      show_autocomplete: false,
      file_name_cursor: 7, // Position at end of "NewFile"
      package_name_cursor: default_package.len(),
      focused_field: FocusedField::FileType,
      cwd,
    }
  }

  /// Fetch packages directly from service
  fn fetch_packages(cwd: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = get_all_packages_command::execute(cwd, &JavaSourceDirectoryType::Main);

    if let Some(data) = response.data {
      Ok(data.packages.iter().map(|p| p.package_name.clone()).collect())
    } else {
      Err("Failed to fetch packages".into())
    }
  }

  /// Move focus to the next field
  fn focus_next(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::FileType => FocusedField::FileName,
      FocusedField::FileName => FocusedField::PackageName,
      FocusedField::PackageName => FocusedField::ConfirmButton,
      FocusedField::ConfirmButton => FocusedField::FileType,
    };
  }

  /// Move focus to the previous field
  fn focus_prev(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::FileType => FocusedField::ConfirmButton,
      FocusedField::FileName => FocusedField::FileType,
      FocusedField::PackageName => FocusedField::FileName,
      FocusedField::ConfirmButton => FocusedField::PackageName,
    };
  }

  /// Filter packages based on current input (for autocomplete)
  fn filter_packages(&mut self) {
    if self.package_name.is_empty() {
      self.filtered_packages.clear();
      self.show_autocomplete = false;
      self.autocomplete_scroll_offset = 0;
      return;
    }

    let input_lower = self.package_name.to_lowercase();

    // Separate exact prefix matches and contains matches
    let mut prefix_matches = Vec::new();
    let mut contains_matches = Vec::new();

    for package in &self.package_list {
      let package_lower = package.to_lowercase();
      if package_lower.starts_with(&input_lower) {
        prefix_matches.push(package.clone());
      } else if package_lower.contains(&input_lower) {
        contains_matches.push(package.clone());
      }
    }

    // Combine: prefix matches first, then contains matches
    prefix_matches.sort();
    contains_matches.sort();
    prefix_matches.extend(contains_matches);

    // Limit to top 7 suggestions
    self.filtered_packages = prefix_matches.into_iter().take(7).collect();
    self.show_autocomplete = !self.filtered_packages.is_empty();

    // Reset selection and scroll if out of bounds
    if let Some(idx) = self.autocomplete_selected_index && idx >= self.filtered_packages.len() {
      self.autocomplete_selected_index = None;
      self.autocomplete_scroll_offset = 0;
    }
  }

  /// Called when entering insert mode - 'a' moves cursor to end for text inputs
  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    if key == KeyCode::Char('a') {
      match self.focused_field {
        FocusedField::FileName => {
          self.file_name_cursor = self.file_name.len();
        }
        FocusedField::PackageName => {
          self.package_name_cursor = self.package_name.len();
        }
        _ => {}
      }
    }
  }

  /// Called when Enter is pressed in Normal mode
  fn on_enter_pressed(&mut self) {
    if self.focused_field == FocusedField::ConfirmButton {
      self.execute_create_java_file();
    }
  }

  /// Handle field-specific input in Insert mode
  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::FileName => self.handle_file_name_input(key),
      FocusedField::PackageName => self.handle_package_name_input(key),
      FocusedField::FileType => self.handle_file_type_insert(key),
      FocusedField::ConfirmButton => {
        // Can't insert on button, exit insert mode
        self.state.input_mode = InputMode::Normal;
      }
    }
  }

  fn handle_file_type_insert(&mut self, key: KeyCode) {
    match key {
      // Use j/k for navigation in insert mode
      KeyCode::Char('j') | KeyCode::Down => {
        let len = JavaFileType::value_variants().len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.file_type_state, len);
        self.update_file_type_value();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = JavaFileType::value_variants().len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.file_type_state, len);
        self.update_file_type_value();
      }
      KeyCode::Enter => {
        // Accept selection and exit insert mode
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_file_name_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.file_name.insert(self.file_name_cursor, c);
        self.file_name_cursor += 1;
      }
      KeyCode::Backspace => {
        if self.file_name_cursor > 0 {
          self.file_name.remove(self.file_name_cursor - 1);
          self.file_name_cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if self.file_name_cursor < self.file_name.len() {
          self.file_name.remove(self.file_name_cursor);
        }
      }
      KeyCode::Left => {
        if self.file_name_cursor > 0 {
          self.file_name_cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.file_name_cursor < self.file_name.len() {
          self.file_name_cursor += 1;
        }
      }
      KeyCode::Home => {
        self.file_name_cursor = 0;
      }
      KeyCode::End => {
        self.file_name_cursor = self.file_name.len();
      }
      _ => {}
    }
  }

  fn handle_package_name_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.package_name.insert(self.package_name_cursor, c);
        self.package_name_cursor += 1;
        // Update autocomplete suggestions
        self.filter_packages();
        self.autocomplete_selected_index = None;
        self.autocomplete_scroll_offset = 0;
      }
      KeyCode::Backspace => {
        if self.package_name_cursor > 0 {
          self.package_name.remove(self.package_name_cursor - 1);
          self.package_name_cursor -= 1;
          // Update autocomplete suggestions
          self.filter_packages();
          self.autocomplete_selected_index = None;
          self.autocomplete_scroll_offset = 0;
        }
      }
      KeyCode::Delete => {
        if self.package_name_cursor < self.package_name.len() {
          self.package_name.remove(self.package_name_cursor);
          // Update autocomplete suggestions
          self.filter_packages();
          self.autocomplete_selected_index = None;
          self.autocomplete_scroll_offset = 0;
        }
      }
      KeyCode::Left => {
        if self.package_name_cursor > 0 {
          self.package_name_cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.package_name_cursor < self.package_name.len() {
          self.package_name_cursor += 1;
        }
      }
      KeyCode::Home => {
        self.package_name_cursor = 0;
      }
      KeyCode::End => {
        self.package_name_cursor = self.package_name.len();
      }
      KeyCode::Down => {
        // Navigate down in autocomplete suggestions
        if self.show_autocomplete && !self.filtered_packages.is_empty() {
          let new_idx = match self.autocomplete_selected_index {
            None => 0,
            Some(idx) => {
              if idx + 1 < self.filtered_packages.len() {
                idx + 1
              } else {
                0 // Wrap to top
              }
            }
          };

          self.autocomplete_selected_index = Some(new_idx);

          // Update scroll offset to keep selection visible (max 3 visible items)
          const MAX_VISIBLE: usize = 3;
          if new_idx >= self.autocomplete_scroll_offset + MAX_VISIBLE {
            self.autocomplete_scroll_offset = new_idx - MAX_VISIBLE + 1;
          } else if new_idx < self.autocomplete_scroll_offset {
            self.autocomplete_scroll_offset = new_idx;
          }
        }
      }
      KeyCode::Up => {
        // Navigate up in autocomplete suggestions
        if self.show_autocomplete && !self.filtered_packages.is_empty() {
          let new_idx = match self.autocomplete_selected_index {
            None => self.filtered_packages.len() - 1,
            Some(idx) => {
              if idx > 0 {
                idx - 1
              } else {
                self.filtered_packages.len() - 1 // Wrap to bottom
              }
            }
          };

          self.autocomplete_selected_index = Some(new_idx);

          // Update scroll offset to keep selection visible (max 3 visible items)
          const MAX_VISIBLE: usize = 3;
          if new_idx >= self.autocomplete_scroll_offset + MAX_VISIBLE {
            self.autocomplete_scroll_offset = new_idx - MAX_VISIBLE + 1;
          } else if new_idx < self.autocomplete_scroll_offset {
            self.autocomplete_scroll_offset = new_idx;
          }
        }
      }
      KeyCode::Tab | KeyCode::Enter => {
        // Accept autocomplete suggestion
        if self.show_autocomplete && !self.filtered_packages.is_empty() {
          if let Some(idx) = self.autocomplete_selected_index {
            if let Some(selected) = self.filtered_packages.get(idx) {
              self.package_name = selected.clone();
              self.package_name_cursor = self.package_name.len();
              self.show_autocomplete = false;
              self.autocomplete_selected_index = None;
              self.autocomplete_scroll_offset = 0;
              self.filtered_packages.clear();
            }
          } else if !self.filtered_packages.is_empty() {
            // If nothing selected, select first suggestion
            self.package_name = self.filtered_packages[0].clone();
            self.package_name_cursor = self.package_name.len();
            self.show_autocomplete = false;
            self.autocomplete_selected_index = None;
            self.autocomplete_scroll_offset = 0;
            self.filtered_packages.clear();
          }
        } else if key == KeyCode::Enter {
          // If no autocomplete, Enter exits insert mode
          self.state.input_mode = InputMode::Normal;
        }
      }
      KeyCode::Esc => {
        // Hide autocomplete without selecting
        self.show_autocomplete = false;
        self.autocomplete_selected_index = None;
        self.autocomplete_scroll_offset = 0;
      }
      _ => {}
    }
  }

  fn update_file_type_value(&mut self) {
    if let Some(i) = self.file_type_state.selected() {
      let variants = JavaFileType::value_variants();
      if let Some(variant) = variants.get(i) {
        self.file_type = variant.clone();
      }
    }
  }

  fn execute_create_java_file(&mut self) {
    // Call service directly with enum types - always use Main source directory
    match create_java_file_service::run(
      &self.cwd,
      &self.package_name,
      &self.file_name,
      &self.file_type,
      &JavaSourceDirectoryType::Main,
    ) {
      Ok(response) => {
        // Success! Print result to stderr (stdout reserved for TUI)
        eprintln!("Successfully created {} at {}", response.file_type, response.file_path);
        // Signal quit and exit with success code
        self.state.should_quit = true;
        std::process::exit(0);
      }
      Err(e) => {
        // Show error in UI instead of quitting
        self.state.error_message = Some(format!("Error: {}", e));
      }
    }
  }

  fn render_file_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FileType;
    let variants = JavaFileType::value_variants();

    let items: Vec<ListItem> = variants
      .iter()
      .enumerate()
      .map(|(i, variant)| {
        let is_selected = self.file_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };

        // Get display name from enum using to_possible_value()
        let display_name = variant
          .to_possible_value()
          .map(|pv| pv.get_name().to_string())
          .unwrap_or_else(|| "unknown".to_string());

        // Capitalize first letter for display: "class" -> "Class"
        let label = if !display_name.is_empty() {
          format!("{}{}", display_name[0..1].to_uppercase(), &display_name[1..])
        } else {
          display_name
        };

        ListItem::new(format!(" {} {}", prefix, label))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title("File type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.file_type_state);
  }

  fn render_file_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FileName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title("File name", is_focused);
    let input = Paragraph::new(self.file_name.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);
    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.file_name_cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_package_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::PackageName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    // Build title with autocomplete hint
    let title = if is_focused && self.show_autocomplete && !self.filtered_packages.is_empty() {
      format!(
        "Package name [↑↓ navigate, Tab/Enter select, Esc cancel] ({} matches)",
        self.filtered_packages.len()
      )
    } else {
      self.generate_title("Package name", is_focused)
    };

    let input = Paragraph::new(self.package_name.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);

    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.package_name_cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_autocomplete_dropdown(&self, frame: &mut Frame, area: Rect) {
    if !self.show_autocomplete || self.filtered_packages.is_empty() {
      return;
    }

    const MAX_VISIBLE: usize = 3;
    let total_items = self.filtered_packages.len();

    // Calculate visible range
    let visible_start = self.autocomplete_scroll_offset;
    let visible_end = (visible_start + MAX_VISIBLE).min(total_items);

    // Create list items for visible autocomplete suggestions only
    let items: Vec<ListItem> = self
      .filtered_packages
      .iter()
      .enumerate()
      .skip(visible_start)
      .take(visible_end - visible_start)
      .map(|(i, pkg)| {
        let is_selected = self.autocomplete_selected_index == Some(i);
        let prefix = if is_selected { "▶" } else { " " };
        ListItem::new(format!("{} {}", prefix, pkg)).style(if is_selected {
          Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
          Style::default()
        })
      })
      .collect();

    // Build title with scroll indicator
    let title = if total_items > MAX_VISIBLE {
      format!("Suggestions ({}-{} of {})", visible_start + 1, visible_end, total_items)
    } else {
      format!("Suggestions ({})", total_items)
    };

    let list = List::new(items).block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title),
    );

    frame.render_widget(list, area);
  }

  fn render_confirm_button(&self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::ConfirmButton;
    let color = match self.state.escape_handler.pressed_once {
      true => Color::Red,
      false => Color::Green,
    };
    let text = match self.state.escape_handler.pressed_once {
      true => "Press esc again to close or any key to return",
      false => "Confirm",
    };
    let style = if is_focused {
      Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(color)
    };
    let button = Paragraph::new(format!("[ {} ]", text).to_string())
      .alignment(Alignment::Center)
      .style(style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(button, area);
  }

  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Create new Java file")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    // Split form into sections
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2), // Title bar
        Constraint::Length(6), // File type selector (4 types + 2 borders)
        Constraint::Length(3), // File name input
        Constraint::Length(3), // Package name input
        Constraint::Min(3),    // Flexible space for autocomplete + errors
        Constraint::Length(1), // Confirm button
      ])
      .split(area);

    // Render combined title/status bar
    self.render_title_bar(frame, chunks[0]);

    // Render file type selector
    self.render_file_type_selector(frame, chunks[1]);

    // Render file name input
    self.render_file_name_input(frame, chunks[2]);

    // Render package name input
    self.render_package_name_input(frame, chunks[3]);

    // Render autocomplete dropdown and error message in flexible space
    let flexible_area = chunks[4];

    // Calculate autocomplete dropdown size (always 5 lines: 2 border + 3 items max)
    let autocomplete_height = if self.show_autocomplete && !self.filtered_packages.is_empty() {
      5 // Fixed height: 2 borders + 3 visible items
    } else {
      0
    };

    if autocomplete_height > 0 {
      let autocomplete_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(autocomplete_height), Constraint::Min(0)])
        .split(flexible_area);

      self.render_autocomplete_dropdown(frame, autocomplete_chunks[0]);

      // Render error in remaining space
      if let Some(ref error_msg) = self.state.error_message {
        let error_paragraph =
          Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
            Block::default()
              .title("Error")
              .borders(Borders::ALL)
              .border_style(Style::default().fg(Color::Red)),
          );
        frame.render_widget(error_paragraph, autocomplete_chunks[1]);
      }
    } else {
      // No autocomplete, just render error if present
      if let Some(ref error_msg) = self.state.error_message {
        let error_paragraph =
          Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
            Block::default()
              .title("Error")
              .borders(Borders::ALL)
              .border_style(Style::default().fg(Color::Red)),
          );
        frame.render_widget(error_paragraph, flexible_area);
      }
    }

    // Render confirm button
    self.render_confirm_button(frame, chunks[5]);
  }
}

// Implement the FormBehavior trait to get inherited methods
impl FormBehavior for CreateJavaFileForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateJavaFileForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateJavaFileForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateJavaFileForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateJavaFileForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateJavaFileForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateJavaFileForm::render(self, frame)
  }
}
