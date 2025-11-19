use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::{
  create_jpa_entity_command, get_all_jpa_mapped_superclasses, get_all_packages_command,
};
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, button_helpers, helpers};

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  EntityName,
  Superclass,
  PackageName,
  ConfirmButton,
}

/// Main form state for creating a JPA entity
pub struct CreateJpaEntityForm {
  // Common form state (embedded)
  state: FormState,

  // Field values
  entity_name: String,
  package_name: String,
  selected_superclass_index: Option<usize>, // None means no superclass (first item)

  // Package autocomplete
  package_list: Vec<String>,
  filtered_packages: Vec<String>,
  package_autocomplete_selected_index: Option<usize>,
  package_autocomplete_scroll_offset: usize,
  show_package_autocomplete: bool,

  // Superclass selection (list)
  superclass_list: Vec<String>, // All available mapped superclasses
  superclass_state: ListState,

  // Text input states
  entity_name_cursor: usize,
  package_name_cursor: usize,

  // Focus management
  focused_field: FocusedField,

  // Integration with syntaxpresso-core
  cwd: PathBuf,
}

impl CreateJpaEntityForm {
  pub fn new(cwd: PathBuf) -> Self {
    // Fetch packages from syntaxpresso-core
    let package_list = Self::fetch_packages(&cwd).unwrap_or_else(|_| {
      vec![
        "com.example".to_string(),
        "com.example.model".to_string(),
        "com.example.entity".to_string(),
      ]
    });

    // Fetch mapped superclasses from syntaxpresso-core
    let mut superclass_list = Self::fetch_superclasses(&cwd).unwrap_or_else(|_| vec![]);

    // Add "None" as the first option
    superclass_list.insert(0, "None".to_string());

    // Find the package with the smallest string length (shortest name)
    let default_package = package_list
      .iter()
      .min_by_key(|pkg| pkg.len())
      .cloned()
      .unwrap_or_else(|| "com.example".to_string());

    let mut superclass_state = ListState::default();
    superclass_state.select(Some(0)); // Default to "None"

    Self {
      state: FormState::new(),
      entity_name: "NewEntity".to_string(),
      package_name: default_package.clone(),
      selected_superclass_index: Some(0), // Default to "None"
      package_list,
      filtered_packages: Vec::new(),
      package_autocomplete_selected_index: None,
      package_autocomplete_scroll_offset: 0,
      show_package_autocomplete: false,
      superclass_list,
      superclass_state,
      entity_name_cursor: 9, // Position at end of "NewEntity"
      package_name_cursor: default_package.len(),
      focused_field: FocusedField::EntityName,
      cwd,
    }
  }

  /// Fetch packages from syntaxpresso-core using the get-all-packages command
  fn fetch_packages(cwd: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = get_all_packages_command::execute(cwd, &JavaSourceDirectoryType::Main);

    let mut packages = Vec::new();
    if let Some(data) = response.data {
      for pkg in data.packages {
        packages.push(pkg.package_name);
      }
    }

    Ok(packages)
  }

  /// Fetch mapped superclasses from syntaxpresso-core
  fn fetch_superclasses(cwd: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = get_all_jpa_mapped_superclasses::execute(cwd);

    let mut superclasses = Vec::new();
    if let Some(data) = response.data {
      for file in data.files {
        superclasses.push(file.file_type);
      }
    }

    Ok(superclasses)
  }

  /// Move focus to the next field
  fn focus_next(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::EntityName => FocusedField::Superclass,
      FocusedField::Superclass => FocusedField::PackageName,
      FocusedField::PackageName => FocusedField::ConfirmButton,
      FocusedField::ConfirmButton => FocusedField::EntityName,
    };
  }

  /// Move focus to the previous field
  fn focus_prev(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::EntityName => FocusedField::ConfirmButton,
      FocusedField::Superclass => FocusedField::EntityName,
      FocusedField::PackageName => FocusedField::Superclass,
      FocusedField::ConfirmButton => FocusedField::PackageName,
    };
  }

  /// Filter packages based on current input (for autocomplete)
  fn filter_packages(&mut self) {
    if self.package_name.is_empty() {
      self.filtered_packages.clear();
      self.show_package_autocomplete = false;
      self.package_autocomplete_scroll_offset = 0;
      return;
    }

    let input_lower = self.package_name.to_lowercase();

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

    prefix_matches.sort();
    contains_matches.sort();
    prefix_matches.extend(contains_matches);

    self.filtered_packages = prefix_matches.into_iter().take(7).collect();
    self.show_package_autocomplete = !self.filtered_packages.is_empty();

    if let Some(idx) = self.package_autocomplete_selected_index
      && idx >= self.filtered_packages.len()
    {
      self.package_autocomplete_selected_index = None;
      self.package_autocomplete_scroll_offset = 0;
    }
  }

  /// Update the selected superclass index from the list state
  fn update_superclass_value(&mut self) {
    self.selected_superclass_index = self.superclass_state.selected();
  }

  /// Called when entering insert mode - 'a' moves cursor to end for text inputs
  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    if key == KeyCode::Char('a') {
      match self.focused_field {
        FocusedField::EntityName => {
          self.entity_name_cursor = self.entity_name.len();
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
      self.execute_create_jpa_entity();
    }
  }

  /// Handle field-specific input in Insert mode
  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::EntityName => self.handle_entity_name_input(key),
      FocusedField::Superclass => self.handle_superclass_insert(key),
      FocusedField::PackageName => self.handle_package_name_input(key),
      FocusedField::ConfirmButton => {
        self.state.input_mode = InputMode::Normal;
      }
    }
  }

  fn handle_superclass_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.superclass_list.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.superclass_state, len);
        self.update_superclass_value();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.superclass_list.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.superclass_state, len);
        self.update_superclass_value();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_entity_name_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.entity_name.insert(self.entity_name_cursor, c);
        self.entity_name_cursor += 1;
      }
      KeyCode::Backspace => {
        if self.entity_name_cursor > 0 {
          self.entity_name.remove(self.entity_name_cursor - 1);
          self.entity_name_cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if self.entity_name_cursor < self.entity_name.len() {
          self.entity_name.remove(self.entity_name_cursor);
        }
      }
      KeyCode::Left => {
        if self.entity_name_cursor > 0 {
          self.entity_name_cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.entity_name_cursor < self.entity_name.len() {
          self.entity_name_cursor += 1;
        }
      }
      KeyCode::Home => {
        self.entity_name_cursor = 0;
      }
      KeyCode::End => {
        self.entity_name_cursor = self.entity_name.len();
      }
      _ => {}
    }
  }

  fn handle_package_name_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.package_name.insert(self.package_name_cursor, c);
        self.package_name_cursor += 1;
        self.filter_packages();
        self.package_autocomplete_selected_index = None;
        self.package_autocomplete_scroll_offset = 0;
      }
      KeyCode::Backspace => {
        if self.package_name_cursor > 0 {
          self.package_name.remove(self.package_name_cursor - 1);
          self.package_name_cursor -= 1;
          self.filter_packages();
          self.package_autocomplete_selected_index = None;
          self.package_autocomplete_scroll_offset = 0;
        }
      }
      KeyCode::Delete => {
        if self.package_name_cursor < self.package_name.len() {
          self.package_name.remove(self.package_name_cursor);
          self.filter_packages();
          self.package_autocomplete_selected_index = None;
          self.package_autocomplete_scroll_offset = 0;
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
        if self.show_package_autocomplete && !self.filtered_packages.is_empty() {
          let new_idx = match self.package_autocomplete_selected_index {
            None => 0,
            Some(idx) => {
              if idx + 1 < self.filtered_packages.len() {
                idx + 1
              } else {
                0
              }
            }
          };

          self.package_autocomplete_selected_index = Some(new_idx);

          const MAX_VISIBLE: usize = 3;
          if new_idx >= self.package_autocomplete_scroll_offset + MAX_VISIBLE {
            self.package_autocomplete_scroll_offset = new_idx - MAX_VISIBLE + 1;
          } else if new_idx < self.package_autocomplete_scroll_offset {
            self.package_autocomplete_scroll_offset = new_idx;
          }
        }
      }
      KeyCode::Up => {
        if self.show_package_autocomplete && !self.filtered_packages.is_empty() {
          let new_idx = match self.package_autocomplete_selected_index {
            None => self.filtered_packages.len() - 1,
            Some(idx) => {
              if idx > 0 {
                idx - 1
              } else {
                self.filtered_packages.len() - 1
              }
            }
          };

          self.package_autocomplete_selected_index = Some(new_idx);

          const MAX_VISIBLE: usize = 3;
          if new_idx >= self.package_autocomplete_scroll_offset + MAX_VISIBLE {
            self.package_autocomplete_scroll_offset = new_idx - MAX_VISIBLE + 1;
          } else if new_idx < self.package_autocomplete_scroll_offset {
            self.package_autocomplete_scroll_offset = new_idx;
          }
        }
      }
      KeyCode::Tab | KeyCode::Enter => {
        if self.show_package_autocomplete && !self.filtered_packages.is_empty() {
          if let Some(idx) = self.package_autocomplete_selected_index {
            if let Some(selected) = self.filtered_packages.get(idx) {
              self.package_name = selected.clone();
              self.package_name_cursor = self.package_name.len();
              self.show_package_autocomplete = false;
              self.package_autocomplete_selected_index = None;
              self.package_autocomplete_scroll_offset = 0;
              self.filtered_packages.clear();
            }
          } else if !self.filtered_packages.is_empty() {
            self.package_name = self.filtered_packages[0].clone();
            self.package_name_cursor = self.package_name.len();
            self.show_package_autocomplete = false;
            self.package_autocomplete_selected_index = None;
            self.package_autocomplete_scroll_offset = 0;
            self.filtered_packages.clear();
          }
        } else if key == KeyCode::Enter {
          self.state.input_mode = InputMode::Normal;
        }
      }
      KeyCode::Esc => {
        self.show_package_autocomplete = false;
        self.package_autocomplete_selected_index = None;
        self.package_autocomplete_scroll_offset = 0;
      }
      _ => {}
    }
  }

  fn execute_create_jpa_entity(&mut self) {
    // Get superclass if one is selected (index 0 is "None")
    let (superclass_type, superclass_package_name) =
      if let Some(idx) = self.selected_superclass_index {
        if idx > 0 && idx < self.superclass_list.len() {
          // Selected a real superclass (not "None")
          let superclass = &self.superclass_list[idx];
          // For now, we'll assume the superclass is in the same package
          // In a more sophisticated implementation, you'd fetch the actual package
          (Some(superclass.as_str()), Some(self.package_name.as_str()))
        } else {
          // Selected "None"
          (None, None)
        }
      } else {
        (None, None)
      };

    // Call command layer instead of service directly
    let response = create_jpa_entity_command::execute(
      &self.cwd,
      &self.package_name,
      &self.entity_name,
      superclass_type,
      superclass_package_name,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  fn render_entity_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::EntityName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title("Entity name", is_focused);
    let input = Paragraph::new(self.entity_name.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);
    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.entity_name_cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_package_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::PackageName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title =
      if is_focused && self.show_package_autocomplete && !self.filtered_packages.is_empty() {
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

  fn render_superclass_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::Superclass;

    let items: Vec<ListItem> = self
      .superclass_list
      .iter()
      .enumerate()
      .map(|(i, superclass)| {
        let is_selected = self.superclass_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, superclass))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Superclass (optional)", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.superclass_state);
  }

  fn render_autocomplete_dropdown(&self, frame: &mut Frame, area: Rect) {
    if !self.show_package_autocomplete || self.filtered_packages.is_empty() {
      return;
    }

    const MAX_VISIBLE: usize = 3;
    let total_items = self.filtered_packages.len();

    let visible_start = self.package_autocomplete_scroll_offset;
    let visible_end = (visible_start + MAX_VISIBLE).min(total_items);

    let list_items: Vec<ListItem> = self
      .filtered_packages
      .iter()
      .enumerate()
      .skip(visible_start)
      .take(visible_end - visible_start)
      .map(|(i, item)| {
        let is_selected = self.package_autocomplete_selected_index == Some(i);
        let prefix = if is_selected { "▶" } else { " " };
        ListItem::new(format!("{} {}", prefix, item)).style(if is_selected {
          Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
          Style::default()
        })
      })
      .collect();

    let title = if total_items > MAX_VISIBLE {
      format!("Suggestions ({}-{} of {})", visible_start + 1, visible_end, total_items)
    } else {
      format!("Suggestions ({})", total_items)
    };

    let list = List::new(list_items).block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title),
    );

    frame.render_widget(list, area);
  }

  fn render_confirm_button(&self, frame: &mut Frame, area: Rect) {
    button_helpers::render_single_button(
      frame,
      area,
      self.focused_field == FocusedField::ConfirmButton,
      self.state.escape_handler.pressed_once,
      button_helpers::ButtonType::Confirm,
    );
  }

  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Create new JPA Entity")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    // Calculate superclass list height dynamically
    // Max 7 lines total (2 borders + 5 items visible)
    // This allows scrolling if there are more than 5 items
    let superclass_height = (self.superclass_list.len() as u16 + 2).min(7);

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2),                 // Title bar
        Constraint::Length(3),                 // Entity name input
        Constraint::Length(superclass_height), // Superclass selector (dynamic, max 6)
        Constraint::Length(3),                 // Package name input
        Constraint::Min(3),                    // Flexible space for autocomplete + errors
        Constraint::Length(1),                 // Confirm button
      ])
      .split(area);

    self.render_title_bar(frame, chunks[0]);
    self.render_entity_name_input(frame, chunks[1]);
    self.render_superclass_selector(frame, chunks[2]);
    self.render_package_name_input(frame, chunks[3]);

    // Render autocomplete dropdown and error message in flexible space
    let flexible_area = chunks[4];

    // Only show autocomplete for package name
    let show_autocomplete = self.focused_field == FocusedField::PackageName
      && self.show_package_autocomplete
      && !self.filtered_packages.is_empty();

    let autocomplete_height = if show_autocomplete { 5 } else { 0 };

    if autocomplete_height > 0 {
      let autocomplete_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(autocomplete_height), Constraint::Min(0)])
        .split(flexible_area);

      self.render_autocomplete_dropdown(frame, autocomplete_chunks[0]);

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
    } else if let Some(ref error_msg) = self.state.error_message {
      let error_paragraph =
        Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
          Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        );
      frame.render_widget(error_paragraph, flexible_area);
    }

    self.render_confirm_button(frame, chunks[5]);
  }
}

// Implement the FormBehavior trait to get inherited methods
impl FormBehavior for CreateJpaEntityForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateJpaEntityForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateJpaEntityForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateJpaEntityForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateJpaEntityForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateJpaEntityForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateJpaEntityForm::render(self, frame)
  }
}
