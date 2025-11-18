#![allow(dead_code)]

use clap::ValueEnum;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::{
  create_jpa_entity_id_field_command, get_java_basic_types_command, get_jpa_entity_info_command,
};
use crate::common::types::id_field_config::IdFieldConfig;
use crate::common::types::java_basic_types::JavaBasicType;
use crate::common::types::java_id_generation::JavaIdGeneration;
use crate::common::types::java_id_generation_type::JavaIdGenerationType;
use crate::responses::basic_java_type_response::JavaBasicTypeResponse;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, helpers};

/// ID Generation Strategy options
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum IdGeneration {
  None,
  Auto,
  Identity,
  Sequence,
  Uuid,
}

impl IdGeneration {
  fn all() -> Vec<IdGeneration> {
    vec![
      IdGeneration::None,
      IdGeneration::Auto,
      IdGeneration::Identity,
      IdGeneration::Sequence,
      IdGeneration::Uuid,
    ]
  }

  fn as_str(&self) -> &'static str {
    match self {
      IdGeneration::None => "None (Manual)",
      IdGeneration::Auto => "Auto",
      IdGeneration::Identity => "Identity",
      IdGeneration::Sequence => "Sequence",
      IdGeneration::Uuid => "UUID",
    }
  }
}

/// Generation Type options (for sequence generation)
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum GenerationType {
  OrmProvided,
  EntityExclusiveGeneration,
}

impl GenerationType {
  fn all() -> Vec<GenerationType> {
    vec![GenerationType::OrmProvided, GenerationType::EntityExclusiveGeneration]
  }

  fn as_str(&self) -> &'static str {
    match self {
      GenerationType::OrmProvided => "ORM Provided",
      GenerationType::EntityExclusiveGeneration => "Entity Exclusive Generation",
    }
  }
}

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  FieldType,
  FieldName,
  IdGeneration,
  GenerationType,
  GeneratorName,
  SequenceName,
  InitialValue,
  AllocationSize,
  OtherOptions,
  BackButton,
  ConfirmButton,
}

/// Main form state for creating an ID field
pub struct CreateIdFieldForm {
  // Common form state (embedded)
  state: FormState,

  // Field values
  field_type_index: usize,
  field_package_path: Option<String>,
  field_type: String,
  field_name: String,
  id_generation_index: usize,
  id_generation: IdGeneration,
  generation_type_index: usize,
  generation_type: GenerationType,
  generator_name: String,
  sequence_name: String,
  initial_value: String,
  allocation_size: String,

  // Other options (checkboxes)
  mandatory: bool,
  mutable: bool,

  // Type lists
  all_id_types: Vec<JavaBasicTypeResponse>,

  // List states
  field_type_state: ListState,
  id_generation_state: ListState,
  generation_type_state: ListState,
  other_options_state: ListState,

  // Text input states
  field_name_cursor: usize,
  generator_name_cursor: usize,
  sequence_name_cursor: usize,
  initial_value_cursor: usize,
  allocation_size_cursor: usize,

  // Visibility flags
  id_generation_hidden: bool,
  generation_type_hidden: bool,
  generator_name_hidden: bool,
  sequence_name_hidden: bool,
  initial_value_hidden: bool,
  allocation_size_hidden: bool,

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

impl CreateIdFieldForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    // Fetch ID types from syntaxpresso-core
    let all_id_types = match Self::fetch_id_types() {
      Ok(types) => types,
      Err(_e) => Self::get_default_id_types(),
    };

    let all_id_types =
      if all_id_types.is_empty() { Self::get_default_id_types() } else { all_id_types };

    assert!(!all_id_types.is_empty(), "all_id_types must not be empty!");

    // Fetch entity info to generate generator and sequence names
    let (generator_name, sequence_name) = match Self::fetch_entity_info(&cwd, &entity_file_path) {
      Ok((entity_table_name, entity_type)) => {
        // Use table name if available, otherwise derive from entity type
        let base_name = if let Some(table_name) = entity_table_name {
          table_name
        } else {
          // Convert EntityName to entityName (camelCase)
          let mut chars = entity_type.chars();
          if let Some(first) = chars.next() {
            format!("{}{}", first.to_lowercase(), chars.collect::<String>())
          } else {
            "entity".to_string()
          }
        };
        (format!("{}_gen", base_name), format!("{}_seq", base_name))
      }
      Err(_) => (String::new(), String::new()),
    };

    let generator_name_cursor = generator_name.len();
    let sequence_name_cursor = sequence_name.len();

    let mut field_type_state = ListState::default();
    field_type_state.select(Some(0));

    let mut id_generation_state = ListState::default();
    id_generation_state.select(Some(1)); // Default to Auto

    let mut generation_type_state = ListState::default();
    generation_type_state.select(Some(0)); // Default to OrmProvided

    let mut other_options_state = ListState::default();
    other_options_state.select(Some(0));

    // Default to Long type
    let (default_package, default_type) = if !all_id_types.is_empty() {
      (all_id_types[0].package_path.clone(), all_id_types[0].name.clone())
    } else {
      (Some("java.lang".to_string()), "Long".to_string())
    };

    let mut form = Self {
      state: FormState::new(),
      field_type_index: 0,
      field_package_path: default_package,
      field_type: default_type,
      field_name: "id".to_string(),
      id_generation_index: 1, // Auto
      id_generation: IdGeneration::Auto,
      generation_type_index: 0, // OrmProvided
      generation_type: GenerationType::OrmProvided,
      generator_name,
      sequence_name,
      initial_value: "1".to_string(),
      allocation_size: "50".to_string(),
      mandatory: true,
      mutable: false,
      all_id_types,
      field_type_state,
      id_generation_state,
      generation_type_state,
      other_options_state,
      field_name_cursor: 2, // "id".len()
      generator_name_cursor,
      sequence_name_cursor,
      initial_value_cursor: 1,
      allocation_size_cursor: 2,
      id_generation_hidden: false,
      generation_type_hidden: true,
      generator_name_hidden: true,
      sequence_name_hidden: true,
      initial_value_hidden: true,
      allocation_size_hidden: true,
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

  /// Get default fallback ID types when fetching fails
  fn get_default_id_types() -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.lang.Long".to_string(),
        name: "Long".to_string(),
        package_path: Some("java.lang".to_string()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Integer".to_string(),
        name: "Integer".to_string(),
        package_path: Some("java.lang".to_string()),
      },
      JavaBasicTypeResponse {
        id: "java.util.UUID".to_string(),
        name: "UUID".to_string(),
        package_path: Some("java.util".to_string()),
      },
    ]
  }

  /// Fetch ID types from syntaxpresso-core
  fn fetch_id_types() -> Result<Vec<JavaBasicTypeResponse>, Box<dyn std::error::Error>> {
    let response = get_java_basic_types_command::execute(&JavaBasicType::IdTypes);

    if let Some(types) = response.data { Ok(types) } else { Ok(Vec::new()) }
  }

  /// Fetch JPA entity info to generate generator and sequence names
  /// Returns (table_name, entity_type)
  fn fetch_entity_info(
    cwd: &Path,
    entity_file_path: &Path,
  ) -> Result<(Option<String>, String), Box<dyn std::error::Error>> {
    let response = get_jpa_entity_info_command::execute(cwd, Some(entity_file_path), None);

    if let Some(data) = response.data {
      Ok((data.entity_table_name, data.entity_type))
    } else {
      Err("Failed to fetch entity info".into())
    }
  }

  /// Update field type and related visibility flags
  fn update_field_type(&mut self) {
    if let Some(idx) = self.field_type_state.selected()
      && let Some(type_info) = self.all_id_types.get(idx)
    {
      self.field_type_index = idx;
      self.field_type = type_info.name.clone();
      self.field_package_path = type_info.package_path.clone();

      // UUID type can ONLY use uuid generation
      if type_info.name == "UUID" {
        self.id_generation = IdGeneration::Uuid;
        self.id_generation_hidden = true;
        self.generation_type_hidden = true;
        self.generator_name_hidden = true;
        self.sequence_name_hidden = true;
        self.initial_value_hidden = true;
        self.allocation_size_hidden = true;
      } else {
        // Numeric types: show generation options, default to auto
        self.id_generation = IdGeneration::Auto;
        self.id_generation_hidden = false;
        self.generation_type_hidden = true;
        self.generator_name_hidden = true;
        self.sequence_name_hidden = true;
        self.initial_value_hidden = true;
        self.allocation_size_hidden = true;
      }
    }
  }

  /// Update ID generation and related visibility flags
  fn update_id_generation(&mut self) {
    if let Some(idx) = self.id_generation_state.selected() {
      let all_generations = IdGeneration::all();
      if let Some(generation) = all_generations.get(idx) {
        self.id_generation_index = idx;
        self.id_generation = *generation;

        // Only SEQUENCE generation uses generation type and related fields
        if *generation == IdGeneration::Sequence {
          self.generation_type_hidden = false;
          self.generation_type = GenerationType::OrmProvided;
          // Initially hide sequence-specific fields
          self.generator_name_hidden = true;
          self.sequence_name_hidden = true;
          self.initial_value_hidden = true;
          self.allocation_size_hidden = true;
        } else {
          // none, auto, identity, uuid: hide all sequence-related fields
          self.generation_type_hidden = true;
          self.generator_name_hidden = true;
          self.sequence_name_hidden = true;
          self.initial_value_hidden = true;
          self.allocation_size_hidden = true;
        }
      }
    }
  }

  /// Update generation type and related visibility flags
  fn update_generation_type(&mut self) {
    if let Some(idx) = self.generation_type_state.selected() {
      let all_types = GenerationType::all();
      if let Some(gen_type) = all_types.get(idx) {
        self.generation_type_index = idx;
        self.generation_type = *gen_type;

        // Only entity_exclusive_generation requires generator/sequence configuration
        if *gen_type == GenerationType::EntityExclusiveGeneration {
          self.generator_name_hidden = false;
          self.sequence_name_hidden = false;
          self.initial_value_hidden = false;
          self.allocation_size_hidden = false;
        } else {
          // orm_provided: hide all custom generator fields
          self.generator_name_hidden = true;
          self.sequence_name_hidden = true;
          self.initial_value_hidden = true;
          self.allocation_size_hidden = true;
        }
      }
    }
  }

  /// Move focus to the next visible field
  fn focus_next(&mut self) {
    // Reset back button confirmation when focus changes
    self.back_pressed_once = false;

    loop {
      self.focused_field = match self.focused_field {
        FocusedField::FieldType => FocusedField::FieldName,
        FocusedField::FieldName => FocusedField::IdGeneration,
        FocusedField::IdGeneration => FocusedField::GenerationType,
        FocusedField::GenerationType => FocusedField::GeneratorName,
        FocusedField::GeneratorName => FocusedField::SequenceName,
        FocusedField::SequenceName => FocusedField::InitialValue,
        FocusedField::InitialValue => FocusedField::AllocationSize,
        FocusedField::AllocationSize => FocusedField::OtherOptions,
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
        FocusedField::IdGeneration => FocusedField::FieldName,
        FocusedField::GenerationType => FocusedField::IdGeneration,
        FocusedField::GeneratorName => FocusedField::GenerationType,
        FocusedField::SequenceName => FocusedField::GeneratorName,
        FocusedField::InitialValue => FocusedField::SequenceName,
        FocusedField::AllocationSize => FocusedField::InitialValue,
        FocusedField::OtherOptions => FocusedField::AllocationSize,
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
      FocusedField::IdGeneration => self.id_generation_hidden,
      FocusedField::GenerationType => self.generation_type_hidden,
      FocusedField::GeneratorName => self.generator_name_hidden,
      FocusedField::SequenceName => self.sequence_name_hidden,
      FocusedField::InitialValue => self.initial_value_hidden,
      FocusedField::AllocationSize => self.allocation_size_hidden,
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
        FocusedField::GeneratorName => {
          self.generator_name_cursor = self.generator_name.len();
        }
        FocusedField::SequenceName => {
          self.sequence_name_cursor = self.sequence_name.len();
        }
        FocusedField::InitialValue => {
          self.initial_value_cursor = self.initial_value.len();
        }
        FocusedField::AllocationSize => {
          self.allocation_size_cursor = self.allocation_size.len();
        }
        _ => {}
      }
    }
  }

  /// Called when Enter is pressed in Normal mode
  fn on_enter_pressed(&mut self) {
    match self.focused_field {
      FocusedField::ConfirmButton => {
        self.execute_create_id_field();
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
      FocusedField::IdGeneration => self.handle_id_generation_insert(key),
      FocusedField::GenerationType => self.handle_generation_type_insert(key),
      FocusedField::GeneratorName => self.handle_generator_name_input(key),
      FocusedField::SequenceName => self.handle_sequence_name_input(key),
      FocusedField::InitialValue => self.handle_initial_value_input(key),
      FocusedField::AllocationSize => self.handle_allocation_size_input(key),
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
          self.execute_create_id_field();
        }
      }
    }
  }

  fn handle_field_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.all_id_types.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.field_type_state, len);
        self.update_field_type();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.all_id_types.len();
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

  fn handle_id_generation_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = IdGeneration::all().len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.id_generation_state, len);
        self.update_id_generation();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = IdGeneration::all().len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.id_generation_state, len);
        self.update_id_generation();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_generation_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = GenerationType::all().len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.generation_type_state, len);
        self.update_generation_type();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = GenerationType::all().len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.generation_type_state, len);
        self.update_generation_type();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_generator_name_input(&mut self, key: KeyCode) {
    helpers::handle_text_input(
      key,
      &mut self.generator_name,
      &mut self.generator_name_cursor,
      &mut self.state.input_mode,
    );
  }

  fn handle_sequence_name_input(&mut self, key: KeyCode) {
    helpers::handle_text_input(
      key,
      &mut self.sequence_name,
      &mut self.sequence_name_cursor,
      &mut self.state.input_mode,
    );
  }

  fn handle_initial_value_input(&mut self, key: KeyCode) {
    helpers::handle_numeric_input(
      key,
      &mut self.initial_value,
      &mut self.initial_value_cursor,
      &mut self.state.input_mode,
    );
  }

  fn handle_allocation_size_input(&mut self, key: KeyCode) {
    helpers::handle_numeric_input(
      key,
      &mut self.allocation_size,
      &mut self.allocation_size_cursor,
      &mut self.state.input_mode,
    );
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
            1 => self.mutable = !self.mutable,
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

  fn execute_create_id_field(&mut self) {
    // Validate: entity_exclusive_generation requires generator_name
    if self.id_generation == IdGeneration::Sequence
      && self.generation_type == GenerationType::EntityExclusiveGeneration
      && self.generator_name.is_empty()
    {
      self.state.error_message =
        Some("Generator name is required for entity exclusive generation".to_string());
      return;
    }

    // Parse numeric fields
    let initial_value = self.initial_value.parse::<i64>().ok();
    let allocation_size = self.allocation_size.parse::<i64>().ok();

    // Convert generation strategy to enum
    let field_id_generation = match self.id_generation {
      IdGeneration::None => JavaIdGeneration::None,
      IdGeneration::Auto => JavaIdGeneration::Auto,
      IdGeneration::Identity => JavaIdGeneration::Identity,
      IdGeneration::Sequence => JavaIdGeneration::Sequence,
      IdGeneration::Uuid => JavaIdGeneration::Uuid,
    };

    // Convert generation type to enum (only relevant for sequence)
    let field_id_generation_type = if self.id_generation == IdGeneration::Sequence {
      match self.generation_type {
        GenerationType::OrmProvided => JavaIdGenerationType::OrmProvided,
        GenerationType::EntityExclusiveGeneration => {
          JavaIdGenerationType::EntityExclusiveGeneration
        }
      }
    } else {
      JavaIdGenerationType::None
    };

    // Build field config
    let field_config = IdFieldConfig {
      field_name: self.field_name.clone(),
      field_type: self.field_type.clone(),
      field_type_package_name: self.field_package_path.clone(),
      field_id_generation,
      field_id_generation_type,
      field_generator_name: if self.generator_name.is_empty() {
        None
      } else {
        Some(self.generator_name.clone())
      },
      field_sequence_name: if self.sequence_name.is_empty() {
        None
      } else {
        Some(self.sequence_name.clone())
      },
      field_initial_value: initial_value,
      field_allocation_size: allocation_size,
      field_nullable: !self.mandatory,
    };

    // Call command layer instead of service directly
    let response = create_jpa_entity_id_field_command::execute(
      &self.cwd,
      &self.entity_file_b64_src,
      &self.entity_file_path,
      field_config,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title_text = "Create new JPA Entity ID field";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  fn render_field_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FieldType;

    let items: Vec<ListItem> = self
      .all_id_types
      .iter()
      .enumerate()
      .map(|(i, type_info)| {
        let is_selected = self.field_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        let display =
          format!("{} ({})", type_info.name, type_info.package_path.as_deref().unwrap_or(""));
        ListItem::new(format!(" {} {}", prefix, display))
      })
      .collect();

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

  fn render_id_generation_selector(&mut self, frame: &mut Frame, area: Rect) {
    if self.id_generation_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::IdGeneration;
    let all_generations = IdGeneration::all();

    let items: Vec<ListItem> = all_generations
      .iter()
      .enumerate()
      .map(|(i, generation)| {
        let is_selected = self.id_generation_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, generation.as_str()))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("ID Generation Strategy", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.id_generation_state);
  }

  fn render_generation_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    if self.generation_type_hidden {
      return;
    }

    let is_focused = self.focused_field == FocusedField::GenerationType;
    let all_types = GenerationType::all();

    let items: Vec<ListItem> = all_types
      .iter()
      .enumerate()
      .map(|(i, gen_type)| {
        let is_selected = self.generation_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, gen_type.as_str()))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Generation Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.generation_type_state);
  }

  fn render_text_input(
    &self,
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    cursor: usize,
    is_focused: bool,
  ) {
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let title = self.generate_title(label, is_focused);
    let input = Paragraph::new(value)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);
    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + cursor as u16 + 1, area.y + 1));
    }
  }

  fn render_other_options_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::OtherOptions;

    let items = vec![
      ListItem::new(format!(" [{}] Mandatory", if self.mandatory { "x" } else { " " })),
      ListItem::new(format!(" [{}] Mutable", if self.mutable { "x" } else { " " })),
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
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(area);

    // Render Back button
    let is_back_focused = self.focused_field == FocusedField::BackButton;
    let back_color = if self.back_pressed_once { Color::Red } else { Color::Yellow };
    let back_text = if self.back_pressed_once { "Press again to go back" } else { "Back" };
    let back_style = if is_back_focused {
      Style::default().bg(back_color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(back_color)
    };
    let back_button = Paragraph::new(format!("[ {} ]", back_text))
      .alignment(Alignment::Center)
      .style(back_style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(back_button, chunks[0]);

    // Render Confirm button
    let is_confirm_focused = self.focused_field == FocusedField::ConfirmButton;
    let color = match self.state.escape_handler.pressed_once {
      true => Color::Red,
      false => Color::Green,
    };
    let text = match self.state.escape_handler.pressed_once {
      true => "Press esc again to close or any key to return",
      false => "Confirm",
    };
    let confirm_style = if is_confirm_focused {
      Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(color)
    };
    let confirm_button = Paragraph::new(format!("[ {} ]", text))
      .alignment(Alignment::Center)
      .style(confirm_style)
      .block(Block::default().borders(Borders::empty()));
    frame.render_widget(confirm_button, chunks[1]);
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let type_list_height = (self.all_id_types.len() as u16 + 2).clamp(5, 10);
    let id_gen_height = if self.id_generation_hidden { 0 } else { 7 };
    let gen_type_height = if self.generation_type_hidden { 0 } else { 4 };
    let generator_height = if self.generator_name_hidden { 0 } else { 3 };
    let sequence_height = if self.sequence_name_hidden { 0 } else { 3 };
    let initial_height = if self.initial_value_hidden { 0 } else { 3 };
    let allocation_height = if self.allocation_size_hidden { 0 } else { 3 };

    let mut constraints = vec![
      Constraint::Length(2),                // Title bar
      Constraint::Length(type_list_height), // Field type selector
      Constraint::Length(3),                // Field name input
    ];

    if id_gen_height > 0 {
      constraints.push(Constraint::Length(id_gen_height));
    }
    if gen_type_height > 0 {
      constraints.push(Constraint::Length(gen_type_height));
    }
    if generator_height > 0 {
      constraints.push(Constraint::Length(generator_height));
    }
    if sequence_height > 0 {
      constraints.push(Constraint::Length(sequence_height));
    }
    if initial_height > 0 {
      constraints.push(Constraint::Length(initial_height));
    }
    if allocation_height > 0 {
      constraints.push(Constraint::Length(allocation_height));
    }

    constraints.push(Constraint::Length(4)); // Other options
    constraints.push(Constraint::Min(0)); // Flexible space for errors
    constraints.push(Constraint::Length(1)); // Buttons

    let chunks =
      Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let mut chunk_idx = 0;

    self.render_title_bar(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_field_type_selector(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    self.render_field_name_input(frame, chunks[chunk_idx]);
    chunk_idx += 1;

    if id_gen_height > 0 {
      self.render_id_generation_selector(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if gen_type_height > 0 {
      self.render_generation_type_selector(frame, chunks[chunk_idx]);
      chunk_idx += 1;
    }

    if generator_height > 0 {
      self.render_text_input(
        frame,
        chunks[chunk_idx],
        "Generator name (required)",
        &self.generator_name,
        self.generator_name_cursor,
        self.focused_field == FocusedField::GeneratorName,
      );
      chunk_idx += 1;
    }

    if sequence_height > 0 {
      self.render_text_input(
        frame,
        chunks[chunk_idx],
        "Sequence name (optional)",
        &self.sequence_name,
        self.sequence_name_cursor,
        self.focused_field == FocusedField::SequenceName,
      );
      chunk_idx += 1;
    }

    if initial_height > 0 {
      self.render_text_input(
        frame,
        chunks[chunk_idx],
        "Initial value (default: 1)",
        &self.initial_value,
        self.initial_value_cursor,
        self.focused_field == FocusedField::InitialValue,
      );
      chunk_idx += 1;
    }

    if allocation_height > 0 {
      self.render_text_input(
        frame,
        chunks[chunk_idx],
        "Allocation size (default: 50)",
        &self.allocation_size,
        self.allocation_size_cursor,
        self.focused_field == FocusedField::AllocationSize,
      );
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

// Implement the FormBehavior trait
impl FormBehavior for CreateIdFieldForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateIdFieldForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateIdFieldForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateIdFieldForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateIdFieldForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateIdFieldForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateIdFieldForm::render(self, frame)
  }
}
