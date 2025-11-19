#![allow(dead_code)]

use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::{create_jpa_entity_enum_field_command, get_java_files_command};
use crate::common::types::enum_field_config::EnumFieldConfig;
use crate::common::types::java_enum_type::JavaEnumType;
use crate::common::types::java_file_type::JavaFileType;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, helpers};

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  EnumType,
  EnumTypeStorage,
  FieldName,
  FieldLength,
  OtherOptions,
  BackButton,
  ConfirmButton,
}

/// Represents an enum file response
#[derive(Debug, Clone, serde::Deserialize)]
struct EnumFileResponse {
  #[serde(rename = "fileType")]
  file_type: String,
  #[serde(rename = "filePackageName")]
  file_package_name: String,
  #[serde(rename = "filePath")]
  file_path: String,
}

/// Main form state for creating an enum field
pub struct CreateEnumFieldForm {
  // Common form state (embedded)
  state: FormState,

  // Field values
  enum_type_index: usize,
  enum_package_name: String,
  enum_type: String,
  enum_type_path: String,
  field_name: String,
  enum_type_storage_index: usize,
  field_length: String,

  // Other options (checkboxes)
  mandatory: bool,
  unique: bool,

  // Enum lists
  all_enum_types: Vec<EnumFileResponse>,

  // List states
  enum_type_state: ListState,
  enum_type_storage_state: ListState,
  other_options_state: ListState,

  // Text input states
  field_name_cursor: usize,
  field_length_cursor: usize,

  // Visibility flags
  field_length_hidden: bool,

  // Focus management
  focused_field: FocusedField,

  // Integration with syntaxpresso-core
  cwd: PathBuf,
  entity_file_b64_src: String,
  entity_file_path: PathBuf,

  // Navigation
  should_go_back: bool,
  back_pressed_once: bool,
}

impl CreateEnumFieldForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    // Fetch all enum types from syntaxpresso-core
    let all_enum_types = Self::fetch_enum_types(&cwd).unwrap_or_default();

    let mut enum_type_state = ListState::default();
    enum_type_state.select(Some(0));

    let mut enum_type_storage_state = ListState::default();
    enum_type_storage_state.select(Some(1)); // Default to STRING

    let mut other_options_state = ListState::default();
    other_options_state.select(Some(0));

    // Default values
    let (default_enum_package, default_enum_type, default_enum_path) = if !all_enum_types.is_empty()
    {
      (
        all_enum_types[0].file_package_name.clone(),
        all_enum_types[0].file_type.clone(),
        all_enum_types[0].file_path.clone(),
      )
    } else {
      (String::new(), String::new(), String::new())
    };

    let default_field_name = Self::auto_field_name(&default_enum_type);

    Self {
      state: FormState::new(),
      enum_type_index: 0,
      enum_package_name: default_enum_package,
      enum_type: default_enum_type,
      enum_type_path: default_enum_path,
      field_name: default_field_name,
      enum_type_storage_index: 1, // STRING (index 1)
      field_length: "255".to_string(),
      mandatory: false,
      unique: false,
      all_enum_types,
      enum_type_state,
      enum_type_storage_state,
      other_options_state,
      field_name_cursor: 0,
      field_length_cursor: 3,
      field_length_hidden: false, // STRING type shows length by default
      focused_field: FocusedField::EnumType,
      cwd,
      entity_file_b64_src,
      entity_file_path,
      should_go_back: false,
      back_pressed_once: false,
    }
  }

  /// Fetch enum types from syntaxpresso-core
  fn fetch_enum_types(cwd: &Path) -> Result<Vec<EnumFileResponse>, Box<dyn std::error::Error>> {
    let response = get_java_files_command::execute(cwd, &JavaFileType::Enum);

    let mut enum_types = Vec::new();
    if let Some(data) = response.data {
      for file in data.files {
        enum_types.push(EnumFileResponse {
          file_type: file.file_type,
          file_package_name: file.file_package_name,
          file_path: file.file_path,
        });
      }
    }

    Ok(enum_types)
  }

  /// Auto-generate field name from enum type (CamelCase -> camelCase)
  fn auto_field_name(type_name: &str) -> String {
    if type_name.is_empty() {
      return String::new();
    }
    let first_char = type_name.chars().next().unwrap();
    format!("{}{}", first_char.to_lowercase(), &type_name[1..])
  }

  /// Update enum type and related values
  fn update_enum_type(&mut self) {
    if let Some(idx) = self.enum_type_state.selected()
      && let Some(enum_info) = self.all_enum_types.get(idx)
    {
      self.enum_type_index = idx;
      self.enum_type = enum_info.file_type.clone();
      self.enum_package_name = enum_info.file_package_name.clone();
      self.enum_type_path = enum_info.file_path.clone();
      self.field_name = Self::auto_field_name(&self.enum_type);
      self.field_name_cursor = self.field_name.len();
    }
  }

  /// Update enum type storage and related visibility
  fn update_enum_type_storage(&mut self) {
    if let Some(idx) = self.enum_type_storage_state.selected() {
      self.enum_type_storage_index = idx;
      // STRING (index 1) shows length field, ORDINAL (index 0) hides it
      self.field_length_hidden = idx == 0;
    }
  }

  /// Move focus to the next visible field
  fn focus_next(&mut self) {
    // Reset back button confirmation when focus changes
    self.back_pressed_once = false;

    loop {
      self.focused_field = match self.focused_field {
        FocusedField::EnumType => FocusedField::EnumTypeStorage,
        FocusedField::EnumTypeStorage => FocusedField::FieldName,
        FocusedField::FieldName => FocusedField::FieldLength,
        FocusedField::FieldLength => FocusedField::OtherOptions,
        FocusedField::OtherOptions => FocusedField::BackButton,
        FocusedField::BackButton => FocusedField::ConfirmButton,
        FocusedField::ConfirmButton => FocusedField::EnumType,
      };

      // Skip hidden fields
      if !self.is_field_hidden(self.focused_field) {
        break;
      }
    }
  }

  /// Move focus to the previous visible field
  fn focus_prev(&mut self) {
    // Reset back button confirmation when focus changes
    self.back_pressed_once = false;

    loop {
      self.focused_field = match self.focused_field {
        FocusedField::EnumType => FocusedField::ConfirmButton,
        FocusedField::EnumTypeStorage => FocusedField::EnumType,
        FocusedField::FieldName => FocusedField::EnumTypeStorage,
        FocusedField::FieldLength => FocusedField::FieldName,
        FocusedField::OtherOptions => FocusedField::FieldLength,
        FocusedField::BackButton => FocusedField::OtherOptions,
        FocusedField::ConfirmButton => FocusedField::BackButton,
      };

      // Skip hidden fields
      if !self.is_field_hidden(self.focused_field) {
        break;
      }
    }
  }

  /// Check if a field is hidden
  fn is_field_hidden(&self, field: FocusedField) -> bool {
    match field {
      FocusedField::FieldLength => self.field_length_hidden,
      _ => false,
    }
  }

  /// Called when entering insert mode
  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    if key == KeyCode::Char('a') {
      match self.focused_field {
        FocusedField::FieldName => {
          self.field_name_cursor = self.field_name.len();
        }
        FocusedField::FieldLength => {
          self.field_length_cursor = self.field_length.len();
        }
        _ => {}
      }
    }
  }

  /// Called when Enter is pressed in Normal mode
  fn on_enter_pressed(&mut self) {
    match self.focused_field {
      FocusedField::ConfirmButton => {
        self.execute_create_enum_field();
      }
      FocusedField::BackButton => {
        if self.back_pressed_once {
          self.should_go_back = true;
          self.back_pressed_once = false; // Reset after going back
        } else {
          self.back_pressed_once = true;
        }
      }
      _ => {
        self.back_pressed_once = false; // Reset if Enter pressed on other fields
      }
    }
  }

  /// Handle field-specific input in Insert mode
  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::EnumType => self.handle_enum_type_insert(key),
      FocusedField::EnumTypeStorage => self.handle_enum_type_storage_insert(key),
      FocusedField::FieldName => self.handle_field_name_input(key),
      FocusedField::FieldLength => self.handle_field_length_input(key),
      FocusedField::OtherOptions => self.handle_other_options_insert(key),
      FocusedField::BackButton => {
        if key == KeyCode::Enter {
          if self.back_pressed_once {
            self.should_go_back = true;
          } else {
            self.back_pressed_once = true;
          }
        }
      }
      FocusedField::ConfirmButton => {
        if key == KeyCode::Enter {
          self.execute_create_enum_field();
        }
      }
    }
  }

  fn handle_enum_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.all_enum_types.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.enum_type_state, len);
        self.update_enum_type();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.all_enum_types.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.enum_type_state, len);
        self.update_enum_type();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_enum_type_storage_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        // JavaEnumType has 2 variants: Ordinal (0) and String (1)
        helpers::navigate_list_static(&KeyCode::Down, &mut self.enum_type_storage_state, 2);
        self.update_enum_type_storage();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        // JavaEnumType has 2 variants: Ordinal (0) and String (1)
        helpers::navigate_list_static(&KeyCode::Up, &mut self.enum_type_storage_state, 2);
        self.update_enum_type_storage();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_field_name_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) => {
        self.field_name.insert(self.field_name_cursor, c);
        self.field_name_cursor += 1;
      }
      KeyCode::Backspace => {
        if self.field_name_cursor > 0 {
          self.field_name.remove(self.field_name_cursor - 1);
          self.field_name_cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if self.field_name_cursor < self.field_name.len() {
          self.field_name.remove(self.field_name_cursor);
        }
      }
      KeyCode::Left => {
        if self.field_name_cursor > 0 {
          self.field_name_cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.field_name_cursor < self.field_name.len() {
          self.field_name_cursor += 1;
        }
      }
      KeyCode::Home => {
        self.field_name_cursor = 0;
      }
      KeyCode::End => {
        self.field_name_cursor = self.field_name.len();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_field_length_input(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(c) if c.is_ascii_digit() => {
        self.field_length.insert(self.field_length_cursor, c);
        self.field_length_cursor += 1;
      }
      KeyCode::Backspace => {
        if self.field_length_cursor > 0 {
          self.field_length.remove(self.field_length_cursor - 1);
          self.field_length_cursor -= 1;
        }
      }
      KeyCode::Delete => {
        if self.field_length_cursor < self.field_length.len() {
          self.field_length.remove(self.field_length_cursor);
        }
      }
      KeyCode::Left => {
        if self.field_length_cursor > 0 {
          self.field_length_cursor -= 1;
        }
      }
      KeyCode::Right => {
        if self.field_length_cursor < self.field_length.len() {
          self.field_length_cursor += 1;
        }
      }
      KeyCode::Home => {
        self.field_length_cursor = 0;
      }
      KeyCode::End => {
        self.field_length_cursor = self.field_length.len();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_other_options_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        helpers::navigate_list_static(&KeyCode::Down, &mut self.other_options_state, 2);
      }
      KeyCode::Char('k') | KeyCode::Up => {
        helpers::navigate_list_static(&KeyCode::Up, &mut self.other_options_state, 2);
      }
      KeyCode::Char(' ') | KeyCode::Enter => {
        if let Some(idx) = self.other_options_state.selected() {
          match idx {
            0 => self.mandatory = !self.mandatory,
            1 => self.unique = !self.unique,
            _ => {}
          }
        }
        if key == KeyCode::Enter {
          self.state.input_mode = InputMode::Normal;
        }
      }
      _ => {}
    }
  }

  fn execute_create_enum_field(&mut self) {
    // Validate field name
    if self.field_name.is_empty() {
      self.state.error_message = Some("Field name cannot be empty".to_string());
      return;
    }

    // Validate enum type selected
    if self.enum_type.is_empty() {
      self.state.error_message = Some("Please select an enum type".to_string());
      return;
    }

    // Parse field length (only for STRING type)
    let field_length =
      if !self.field_length_hidden { self.field_length.parse::<u16>().ok() } else { None };

    // Get enum type storage
    let enum_type_storage =
      if self.enum_type_storage_index == 0 { JavaEnumType::Ordinal } else { JavaEnumType::String };

    // Build field config
    let field_config = EnumFieldConfig {
      field_name: self.field_name.clone(),
      enum_type: self.enum_type.clone(),
      enum_package_name: self.enum_package_name.clone(),
      enum_type_storage,
      field_length,
      field_nullable: !self.mandatory,
      field_unique: self.unique,
    };

    // Call command layer instead of service directly
    let response = create_jpa_entity_enum_field_command::execute(
      &self.cwd,
      &self.entity_file_b64_src,
      &self.entity_file_path,
      field_config,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  fn render_enum_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::EnumType;

    let items: Vec<ListItem> = if self.all_enum_types.is_empty() {
      vec![ListItem::new(" No enum types available")]
    } else {
      self
        .all_enum_types
        .iter()
        .enumerate()
        .map(|(i, enum_info)| {
          let is_selected = self.enum_type_state.selected() == Some(i);
          let prefix = if is_selected { "●" } else { "○" };
          let display = format!("{} ({})", enum_info.file_type, enum_info.file_package_name);
          ListItem::new(format!(" {} {}", prefix, display))
        })
        .collect()
    };

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Enum type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.enum_type_state);
  }

  fn render_enum_type_storage_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::EnumTypeStorage;

    // JavaEnumType has 2 variants: Ordinal and String
    let items: Vec<ListItem> = vec![
      {
        let is_selected = self.enum_type_storage_state.selected() == Some(0);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} ORDINAL", prefix))
      },
      {
        let is_selected = self.enum_type_storage_state.selected() == Some(1);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} STRING", prefix))
      },
    ];

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Enum type storage", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.enum_type_storage_state);
  }

  fn render_field_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FieldName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title("Field name", is_focused);
    let input = Paragraph::new(self.field_name.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);
    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.field_name_cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_field_length_input(&mut self, frame: &mut Frame, area: Rect) {
    if self.field_length_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::FieldLength;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title("Field length", is_focused);
    let input = Paragraph::new(self.field_length.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);
    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.field_length_cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_other_options_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::OtherOptions;

    let items = vec![
      ListItem::new(format!(" [{}] Mandatory", if self.mandatory { "x" } else { " " })),
      ListItem::new(format!(" [{}] Unique", if self.unique { "x" } else { " " })),
    ];

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Other options (Space to toggle)", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.other_options_state);
  }

  fn render_buttons(&self, frame: &mut Frame, area: Rect) {
    use crate::ui::form_trait::button_helpers::{ButtonType, render_two_button_layout};

    render_two_button_layout(
      frame,
      area,
      self.focused_field == FocusedField::BackButton,
      self.focused_field == FocusedField::ConfirmButton,
      self.back_pressed_once,
      self.state.escape_handler.pressed_once,
      ButtonType::Confirm,
    );
  }

  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title_text = "Create new JPA Entity enum field";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let enum_type_height = (self.all_enum_types.len() as u16 + 2).clamp(7, 15);
    let length_height = if self.field_length_hidden { 0 } else { 3 };

    let mut constraints = vec![
      Constraint::Length(2),                // Title bar
      Constraint::Length(enum_type_height), // Enum type selector
      Constraint::Length(4),                // Enum type storage selector
      Constraint::Length(3),                // Field name input
    ];

    if length_height > 0 {
      constraints.push(Constraint::Length(length_height));
    }

    constraints.push(Constraint::Length(4)); // Other options
    constraints.push(Constraint::Min(0)); // Flexible space for errors
    constraints.push(Constraint::Length(1)); // Buttons

    let chunks =
      Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let mut chunk_idx = 0;

    self.render_title_bar(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_enum_type_selector(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_enum_type_storage_selector(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_field_name_input(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    if length_height > 0 {
      self.render_field_length_input(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    self.render_other_options_selector(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    // Render error message if present
    if let Some(ref error_msg) = self.state.error_message {
      let error_paragraph =
        Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
          Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        );
      frame.render_widget(error_paragraph, chunks[chunk_idx]);
    }
    chunk_idx += 1;

    self.render_buttons(frame, chunks[chunk_idx]);
  }

  /// Check if user wants to go back to category selection
  pub fn should_go_back(&self) -> bool {
    self.should_go_back
  }
}

// Implement the FormBehavior trait to get inherited methods
impl FormBehavior for CreateEnumFieldForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateEnumFieldForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateEnumFieldForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateEnumFieldForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateEnumFieldForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateEnumFieldForm::handle_field_insert(self, key)
  }

  fn handle_normal_mode(&mut self, key: KeyCode) {
    // Handle Ctrl+Enter (F1 signal) - always confirm
    if key == KeyCode::F(1) {
      self.execute_create_enum_field();
      return;
    }

    // Handle Ctrl+Backspace (F2 signal) - same as pressing back button
    if key == KeyCode::F(2) {
      if self.back_pressed_once {
        self.should_go_back = true;
      } else {
        self.back_pressed_once = true;
      }
      return;
    }

    // Default behavior for other keys
    match key {
      KeyCode::Char('j') | KeyCode::Tab => {
        self.back_pressed_once = false; // Reset when changing focus
        self.focus_next();
      }
      KeyCode::Char('k') | KeyCode::BackTab => {
        self.back_pressed_once = false; // Reset when changing focus
        self.focus_prev();
      }
      KeyCode::Char('i') | KeyCode::Char('a') => {
        self.back_pressed_once = false; // Reset when entering insert mode
        self.on_enter_insert_mode(key);
        self.set_input_mode(InputMode::Insert);
      }
      KeyCode::Enter => {
        // Don't reset back_pressed_once here - let on_enter_pressed handle it
        self.on_enter_pressed();
      }
      _ => {
        self.back_pressed_once = false; // Reset on any other key
      }
    }
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateEnumFieldForm::render(self, frame)
  }
}
