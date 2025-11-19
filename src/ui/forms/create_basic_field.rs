#![allow(dead_code)]

use clap::ValueEnum;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::PathBuf;

use crate::commands::{create_jpa_entity_basic_field_command, get_java_basic_types_command};
use crate::common::types::basic_field_config::BasicFieldConfig;
use crate::common::types::java_basic_types::JavaBasicType;
use crate::common::types::java_field_temporal::JavaFieldTemporal;
use crate::common::types::java_field_time_zone_storage::JavaFieldTimeZoneStorage;
use crate::responses::basic_java_type_response::JavaBasicTypeResponse;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, helpers};

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  FieldType,
  FieldName,
  FieldLength,
  TimeZoneStorage,
  Temporal,
  PrecisionAndScale,
  OtherOptions,
  BackButton,
  ConfirmButton,
}

/// Main form state for creating a basic field
pub struct CreateBasicFieldForm {
  // Common form state (embedded)
  state: FormState,

  // Field values
  field_type_index: usize,
  field_package_path: Option<String>,
  field_type: String,
  field_name: String,
  field_length: String,
  field_precision: String,
  field_scale: String,
  time_zone_storage_index: Option<usize>,
  temporal_index: Option<usize>,

  // Other options (checkboxes)
  mandatory: bool,
  unique: bool,
  large_object: bool,

  // Type lists and metadata
  all_types: Vec<JavaBasicTypeResponse>,
  types_with_length: Vec<String>,
  types_with_time_zone_storage: Vec<String>,
  types_with_temporal: Vec<String>,
  types_with_extra_other: Vec<String>,
  types_with_precision_and_scale: Vec<String>,

  // List states
  field_type_state: ListState,
  time_zone_storage_state: ListState,
  temporal_state: ListState,
  other_options_state: ListState,

  // Text input states
  field_name_cursor: usize,
  field_length_cursor: usize,
  field_precision_cursor: usize,
  field_scale_cursor: usize,

  // Visibility flags
  field_length_hidden: bool,
  field_temporal_hidden: bool,
  field_time_zone_storage_hidden: bool,
  field_scale_hidden: bool,
  field_precision_hidden: bool,
  other_extra_hidden: bool,
  other_hidden: bool,

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

impl CreateBasicFieldForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    // Fetch all types from syntaxpresso-core
    // Try to fetch types from syntaxpresso-core, fall back to defaults if it fails
    let type_data = match Self::fetch_type_data() {
      Ok(data) => data,
      Err(_e) => Self::get_default_type_data(),
    };

    // If still no types (shouldn't happen with defaults), use defaults as last resort
    let type_data =
      if type_data.all_types.is_empty() { Self::get_default_type_data() } else { type_data };

    // ASSERTION: Verify we have types
    assert!(!type_data.all_types.is_empty(), "type_data.all_types must not be empty!");

    let mut field_type_state = ListState::default();
    field_type_state.select(Some(0));

    let mut time_zone_storage_state = ListState::default();
    time_zone_storage_state.select(Some(0));

    let mut temporal_state = ListState::default();
    temporal_state.select(Some(0));

    let mut other_options_state = ListState::default();
    other_options_state.select(Some(0));

    // Default to String type
    let (default_package, default_type) = if !type_data.all_types.is_empty() {
      (type_data.all_types[0].package_path.clone(), type_data.all_types[0].name.clone())
    } else {
      (Some("java.lang".to_string()), "String".to_string())
    };

    let mut form = Self {
      state: FormState::new(),
      field_type_index: 0,
      field_package_path: default_package,
      field_type: default_type,
      field_name: String::new(),
      field_length: "255".to_string(),
      field_precision: "19".to_string(),
      field_scale: "2".to_string(),
      time_zone_storage_index: Some(0),
      temporal_index: Some(0),
      mandatory: false,
      unique: false,
      large_object: false,
      all_types: type_data.all_types,
      types_with_length: type_data.types_with_length,
      types_with_time_zone_storage: type_data.types_with_time_zone_storage,
      types_with_temporal: type_data.types_with_temporal,
      types_with_extra_other: type_data.types_with_extra_other,
      types_with_precision_and_scale: type_data.types_with_precision_and_scale,
      field_type_state,
      time_zone_storage_state,
      temporal_state,
      other_options_state,
      field_name_cursor: 0,
      field_length_cursor: 3,
      field_precision_cursor: 2,
      field_scale_cursor: 1,
      field_length_hidden: false,
      field_temporal_hidden: true,
      field_time_zone_storage_hidden: true,
      field_scale_hidden: true,
      field_precision_hidden: true,
      other_extra_hidden: false,
      other_hidden: true,
      focused_field: FocusedField::FieldType,
      cwd,
      entity_file_b64_src,
      entity_file_path,
      should_go_back: false,
      back_pressed_once: false,
    };

    // Update field type to set correct visibility flags
    form.update_field_type();

    form
  }

  /// Get default fallback type data when fetching fails
  fn get_default_type_data() -> TypeData {
    TypeData {
      all_types: vec![
        JavaBasicTypeResponse {
          id: "java.lang.String".to_string(),
          name: "String".to_string(),
          package_path: Some("java.lang".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.lang.Integer".to_string(),
          name: "Integer".to_string(),
          package_path: Some("java.lang".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.lang.Long".to_string(),
          name: "Long".to_string(),
          package_path: Some("java.lang".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.lang.Boolean".to_string(),
          name: "Boolean".to_string(),
          package_path: Some("java.lang".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.time.LocalDate".to_string(),
          name: "LocalDate".to_string(),
          package_path: Some("java.time".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.time.LocalDateTime".to_string(),
          name: "LocalDateTime".to_string(),
          package_path: Some("java.time".to_string()),
        },
        JavaBasicTypeResponse {
          id: "java.math.BigDecimal".to_string(),
          name: "BigDecimal".to_string(),
          package_path: Some("java.math".to_string()),
        },
      ],
      types_with_length: vec!["java.lang.String".to_string()],
      types_with_time_zone_storage: vec![],
      types_with_temporal: vec![],
      types_with_extra_other: vec!["java.lang.String".to_string()],
      types_with_precision_and_scale: vec!["java.math.BigDecimal".to_string()],
    }
  }

  /// Fetch type data from syntaxpresso-core
  fn fetch_type_data() -> Result<TypeData, Box<dyn std::error::Error>> {
    let mut type_data = TypeData {
      all_types: vec![],
      types_with_length: vec![],
      types_with_time_zone_storage: vec![],
      types_with_temporal: vec![],
      types_with_extra_other: vec![],
      types_with_precision_and_scale: vec![],
    };

    // Fetch all types
    let response = get_java_basic_types_command::execute(&JavaBasicType::AllTypes);
    if let Some(types) = response.data {
      type_data.all_types = types;
    }

    // Fetch types with length
    let response = get_java_basic_types_command::execute(&JavaBasicType::TypesWithLength);
    if let Some(types) = response.data {
      for type_obj in types {
        type_data.types_with_length.push(type_obj.id);
      }
    }

    // Fetch types with time zone storage
    let response = get_java_basic_types_command::execute(&JavaBasicType::TypesWithTimeZoneStorage);
    if let Some(types) = response.data {
      for type_obj in types {
        type_data.types_with_time_zone_storage.push(type_obj.id);
      }
    }

    // Fetch types with temporal
    let response = get_java_basic_types_command::execute(&JavaBasicType::TypesWithTemporal);
    if let Some(types) = response.data {
      for type_obj in types {
        type_data.types_with_temporal.push(type_obj.id);
      }
    }

    // Fetch types with extra other
    let response = get_java_basic_types_command::execute(&JavaBasicType::TypesWithExtraOther);
    if let Some(types) = response.data {
      for type_obj in types {
        type_data.types_with_extra_other.push(type_obj.id);
      }
    }

    // Fetch types with precision and scale
    let response =
      get_java_basic_types_command::execute(&JavaBasicType::TypesWithPrecisionAndScale);
    if let Some(types) = response.data {
      for type_obj in types {
        type_data.types_with_precision_and_scale.push(type_obj.id);
      }
    }

    Ok(type_data)
  }

  /// Update field type and related visibility flags
  fn update_field_type(&mut self) {
    if let Some(idx) = self.field_type_state.selected()
      && let Some(type_info) = self.all_types.get(idx)
    {
      self.field_type_index = idx;
      self.field_type = type_info.name.clone();
      self.field_package_path = type_info.package_path.clone();

      // Update visibility based on type
      let type_id = &type_info.id;

      self.field_length_hidden = !self.types_with_length.contains(type_id);
      self.field_time_zone_storage_hidden = !self.types_with_time_zone_storage.contains(type_id);
      self.field_temporal_hidden = !self.types_with_temporal.contains(type_id);

      if self.types_with_extra_other.contains(type_id) {
        self.other_hidden = true;
        self.other_extra_hidden = false;
      } else {
        self.other_hidden = false;
        self.other_extra_hidden = true;
      }

      self.field_scale_hidden = !self.types_with_precision_and_scale.contains(type_id);
      self.field_precision_hidden = !self.types_with_precision_and_scale.contains(type_id);
    }
  }

  /// Move focus to the next visible field
  fn focus_next(&mut self) {
    // Reset back button confirmation when focus changes
    self.back_pressed_once = false;

    loop {
      self.focused_field = match self.focused_field {
        FocusedField::FieldType => FocusedField::FieldName,
        FocusedField::FieldName => FocusedField::FieldLength,
        FocusedField::FieldLength => FocusedField::TimeZoneStorage,
        FocusedField::TimeZoneStorage => FocusedField::Temporal,
        FocusedField::Temporal => FocusedField::PrecisionAndScale,
        FocusedField::PrecisionAndScale => FocusedField::OtherOptions,
        FocusedField::OtherOptions => FocusedField::BackButton,
        FocusedField::BackButton => FocusedField::ConfirmButton,
        FocusedField::ConfirmButton => FocusedField::FieldType,
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
        FocusedField::FieldType => FocusedField::ConfirmButton,
        FocusedField::FieldName => FocusedField::FieldType,
        FocusedField::FieldLength => FocusedField::FieldName,
        FocusedField::TimeZoneStorage => FocusedField::FieldLength,
        FocusedField::Temporal => FocusedField::TimeZoneStorage,
        FocusedField::PrecisionAndScale => FocusedField::Temporal,
        FocusedField::OtherOptions => FocusedField::PrecisionAndScale,
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
      FocusedField::TimeZoneStorage => self.field_time_zone_storage_hidden,
      FocusedField::Temporal => self.field_temporal_hidden,
      FocusedField::PrecisionAndScale => self.field_scale_hidden && self.field_precision_hidden,
      FocusedField::OtherOptions => self.other_hidden && self.other_extra_hidden,
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
    // In Normal mode, buttons should also work immediately
    match self.focused_field {
      FocusedField::ConfirmButton => {
        self.execute_create_basic_field();
      }
      FocusedField::BackButton => {
        if self.back_pressed_once {
          self.should_go_back = true;
        } else {
          self.back_pressed_once = true;
        }
      }
      _ => {}
    }
  }

  /// Handle field-specific input in Insert mode
  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::FieldType => self.handle_field_type_insert(key),
      FocusedField::FieldName => self.handle_field_name_input(key),
      FocusedField::FieldLength => self.handle_field_length_input(key),
      FocusedField::TimeZoneStorage => self.handle_time_zone_storage_insert(key),
      FocusedField::Temporal => self.handle_temporal_insert(key),
      FocusedField::PrecisionAndScale => self.handle_precision_scale_input(key),
      FocusedField::OtherOptions => self.handle_other_options_insert(key),
      FocusedField::BackButton => {
        // Back button requires double press for confirmation
        if key == KeyCode::Enter {
          if self.back_pressed_once {
            self.should_go_back = true;
          } else {
            self.back_pressed_once = true;
          }
        }
      }
      FocusedField::ConfirmButton => {
        // Confirm button acts immediately
        if key == KeyCode::Enter {
          self.execute_create_basic_field();
        }
      }
    }
  }

  fn handle_field_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.all_types.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.field_type_state, len);
        self.update_field_type();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.all_types.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.field_type_state, len);
        self.update_field_type();
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

  fn handle_time_zone_storage_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let variants = JavaFieldTimeZoneStorage::value_variants();
        let len = variants.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.time_zone_storage_state, len);
        self.time_zone_storage_index = self.time_zone_storage_state.selected();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let variants = JavaFieldTimeZoneStorage::value_variants();
        let len = variants.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.time_zone_storage_state, len);
        self.time_zone_storage_index = self.time_zone_storage_state.selected();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_temporal_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let variants = JavaFieldTemporal::value_variants();
        let len = variants.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.temporal_state, len);
        self.temporal_index = self.temporal_state.selected();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let variants = JavaFieldTemporal::value_variants();
        let len = variants.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.temporal_state, len);
        self.temporal_index = self.temporal_state.selected();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_precision_scale_input(&mut self, key: KeyCode) {
    // For simplicity, we'll handle both precision and scale together
    // In a more sophisticated implementation, you could track which one is focused
    match key {
      KeyCode::Char(c) if c.is_ascii_digit() => {
        self.field_precision.insert(self.field_precision_cursor, c);
        self.field_precision_cursor += 1;
      }
      KeyCode::Backspace => {
        if self.field_precision_cursor > 0 {
          self.field_precision.remove(self.field_precision_cursor - 1);
          self.field_precision_cursor -= 1;
        }
      }
      KeyCode::Tab => {
        // Switch to scale input
        self.field_scale_cursor = 0;
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
        let len = if !self.other_extra_hidden { 3 } else { 2 };
        helpers::navigate_list_static(&KeyCode::Down, &mut self.other_options_state, len);
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = if !self.other_extra_hidden { 3 } else { 2 };
        helpers::navigate_list_static(&KeyCode::Up, &mut self.other_options_state, len);
      }
      KeyCode::Char(' ') | KeyCode::Enter => {
        // Toggle the selected option
        if let Some(idx) = self.other_options_state.selected() {
          if !self.other_extra_hidden {
            match idx {
              0 => self.large_object = !self.large_object,
              1 => self.mandatory = !self.mandatory,
              2 => self.unique = !self.unique,
              _ => {}
            }
          } else {
            match idx {
              0 => self.mandatory = !self.mandatory,
              1 => self.unique = !self.unique,
              _ => {}
            }
          }
        }
        if key == KeyCode::Enter {
          self.state.input_mode = InputMode::Normal;
        }
      }
      _ => {}
    }
  }

  fn execute_create_basic_field(&mut self) {
    // Parse numeric fields
    let field_length = self.field_length.parse::<u16>().ok();
    let field_precision = self.field_precision.parse::<u16>().ok();
    let field_scale = self.field_scale.parse::<u16>().ok();

    // Get time zone storage
    let field_timezone_storage = if !self.field_time_zone_storage_hidden {
      self.time_zone_storage_index.and_then(|idx| {
        let variants = JavaFieldTimeZoneStorage::value_variants();
        variants.get(idx).cloned()
      })
    } else {
      None
    };

    // Get temporal
    let field_temporal = if !self.field_temporal_hidden {
      self.temporal_index.and_then(|idx| {
        let variants = JavaFieldTemporal::value_variants();
        variants.get(idx).cloned()
      })
    } else {
      None
    };

    // Build field config
    let field_config = BasicFieldConfig {
      field_name: self.field_name.clone(),
      field_type: self.field_type.clone(),
      field_type_package_name: self.field_package_path.clone(),
      field_length,
      field_precision,
      field_scale,
      field_temporal,
      field_timezone_storage,
      field_unique: self.unique,
      field_nullable: !self.mandatory,
      field_large_object: self.large_object,
    };

    // Call command layer instead of service directly
    let response = create_jpa_entity_basic_field_command::execute(
      &self.cwd,
      &self.entity_file_b64_src,
      &self.entity_file_path,
      &field_config,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  fn render_field_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FieldType;

    let items: Vec<ListItem> = if self.all_types.is_empty() {
      // This shouldn't happen due to defaults, but just in case
      vec![ListItem::new(" No types available")]
    } else {
      self
        .all_types
        .iter()
        .enumerate()
        .map(|(i, type_info)| {
          let is_selected = self.field_type_state.selected() == Some(i);
          let prefix = if is_selected { "●" } else { "○" };
          let display =
            format!("{} ({})", type_info.name, type_info.package_path.as_deref().unwrap_or(""));
          ListItem::new(format!(" {} {}", prefix, display))
        })
        .collect()
    };

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Field type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.field_type_state);
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

  fn render_time_zone_storage_selector(&mut self, frame: &mut Frame, area: Rect) {
    if self.field_time_zone_storage_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::TimeZoneStorage;
    let variants = JavaFieldTimeZoneStorage::value_variants();

    let items: Vec<ListItem> = variants
      .iter()
      .enumerate()
      .map(|(i, variant)| {
        let is_selected = self.time_zone_storage_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        let display = variant.as_str();
        ListItem::new(format!(" {} {}", prefix, display))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Time Zone Storage", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.time_zone_storage_state);
  }

  fn render_temporal_selector(&mut self, frame: &mut Frame, area: Rect) {
    if self.field_temporal_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::Temporal;
    let variants = JavaFieldTemporal::value_variants();

    let items: Vec<ListItem> = variants
      .iter()
      .enumerate()
      .map(|(i, variant)| {
        let is_selected = self.temporal_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        let display = variant.as_str();
        ListItem::new(format!(" {} {}", prefix, display))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Temporal", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.temporal_state);
  }

  fn render_precision_scale_inputs(&mut self, frame: &mut Frame, area: Rect) {
    if self.field_precision_hidden && self.field_scale_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::PrecisionAndScale;

    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(area);

    if !self.field_precision_hidden {
      let border_style =
        if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
      let title = self.generate_title("Precision", is_focused);
      let input = Paragraph::new(self.field_precision.as_str())
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
      frame.render_widget(input, chunks[0]);
    }

    if !self.field_scale_hidden {
      let border_style =
        if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
      let input = Paragraph::new(self.field_scale.as_str())
        .block(Block::default().title("Scale").borders(Borders::ALL).border_style(border_style));
      frame.render_widget(input, chunks[1]);
    }
  }

  fn render_other_options_selector(&mut self, frame: &mut Frame, area: Rect) {
    if self.other_hidden && self.other_extra_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::OtherOptions;

    let mut items = Vec::new();

    if !self.other_extra_hidden {
      items.push(ListItem::new(format!(
        " [{}] Large object",
        if self.large_object { "x" } else { " " }
      )));
      items.push(ListItem::new(format!(" [{}] Mandatory", if self.mandatory { "x" } else { " " })));
      items.push(ListItem::new(format!(" [{}] Unique", if self.unique { "x" } else { " " })));
    } else {
      items.push(ListItem::new(format!(" [{}] Mandatory", if self.mandatory { "x" } else { " " })));
      items.push(ListItem::new(format!(" [{}] Unique", if self.unique { "x" } else { " " })));
    }

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
    let title_text = "Create new JPA Entity basic field";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    // Calculate dynamic heights
    // Ensure minimum height of 7 for the type list (5 items + 2 for borders)
    let type_list_height = (self.all_types.len() as u16 + 2).clamp(7, 15);
    let length_height = if self.field_length_hidden { 0 } else { 3 };
    let tz_storage_height = if self.field_time_zone_storage_hidden { 0 } else { 7 };
    let temporal_height = if self.field_temporal_hidden { 0 } else { 5 };
    let precision_scale_height =
      if self.field_precision_hidden && self.field_scale_hidden { 0 } else { 3 };
    let other_height = if self.other_hidden && self.other_extra_hidden {
      0
    } else if !self.other_extra_hidden {
      5
    } else {
      4
    };

    let mut constraints = vec![
      Constraint::Length(2),                // Title bar
      Constraint::Length(type_list_height), // Field type selector
      Constraint::Length(3),                // Field name input
    ];

    if length_height > 0 {
      constraints.push(Constraint::Length(length_height));
    }
    if tz_storage_height > 0 {
      constraints.push(Constraint::Length(tz_storage_height));
    }
    if temporal_height > 0 {
      constraints.push(Constraint::Length(temporal_height));
    }
    if precision_scale_height > 0 {
      constraints.push(Constraint::Length(precision_scale_height));
    }
    if other_height > 0 {
      constraints.push(Constraint::Length(other_height));
    }

    constraints.push(Constraint::Min(0)); // Flexible space for errors
    constraints.push(Constraint::Length(1)); // Confirm button

    let chunks =
      Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let mut chunk_idx = 0;

    self.render_title_bar(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_field_type_selector(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_field_name_input(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    if length_height > 0 {
      self.render_field_length_input(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if tz_storage_height > 0 {
      self.render_time_zone_storage_selector(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if temporal_height > 0 {
      self.render_temporal_selector(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if precision_scale_height > 0 {
      self.render_precision_scale_inputs(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if other_height > 0 {
      self.render_other_options_selector(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

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
impl FormBehavior for CreateBasicFieldForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateBasicFieldForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateBasicFieldForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateBasicFieldForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateBasicFieldForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateBasicFieldForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateBasicFieldForm::render(self, frame)
  }
}

// Helper struct for type data
struct TypeData {
  all_types: Vec<JavaBasicTypeResponse>,
  types_with_length: Vec<String>,
  types_with_time_zone_storage: Vec<String>,
  types_with_temporal: Vec<String>,
  types_with_extra_other: Vec<String>,
  types_with_precision_and_scale: Vec<String>,
}
