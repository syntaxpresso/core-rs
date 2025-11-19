#![allow(dead_code)]

use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::create_jpa_one_to_one_relationship_command;
use crate::commands::services::{get_all_jpa_entities_service, get_jpa_entity_info_service};
use crate::common::types::cascade_type::CascadeType;
use crate::common::types::mapping_type::MappingType;
use crate::common::types::one_to_one_field_config::OneToOneFieldConfig;
use crate::common::types::other_type::OtherType;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, button_helpers, helpers};

/// Entity type information
#[derive(Debug, Clone)]
struct EntityTypeInfo {
  name: String,
  package_name: String,
}

/// Represents which phase of the form we're in
#[derive(Debug, Clone, Copy, PartialEq)]
enum FormPhase {
  OwningConfiguration,
  InverseConfiguration,
}

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  // Phase 1: Owning Configuration
  MappingType,
  TargetEntityType,
  OwningFieldName,
  OwningCascades,
  OwningOther,

  // Phase 2: Inverse Configuration (only for bidirectional)
  InverseFieldName,
  InverseCascades,
  InverseOther,

  // Navigation
  BackButton,
  NextButton,
  ConfirmButton,
}

/// Parameters for static render functions
struct RenderContext<'a> {
  focused_field: FocusedField,
  form_state: &'a FormState,
}

/// Parameters for selector rendering
struct SelectorParams<'a> {
  field: FocusedField,
  title: &'a str,
  selected_indices: &'a [usize],
}

/// Main form state for creating one-to-one relationships
pub struct CreateOneToOneRelationshipForm {
  // Common form state (embedded)
  state: FormState,

  // Current phase
  phase: FormPhase,

  // Field values
  mapping_type_index: usize,
  target_entity_index: Option<usize>,
  owning_field_name: String,
  inverse_field_name: String,

  // Current entity information (owning side)
  current_entity_name: String,
  current_entity_package: String,

  // Entity types available for selection (target entities - excludes current entity)
  entity_types: Vec<EntityTypeInfo>,

  // Cascade selections (indices of selected items)
  owning_cascades: Vec<usize>,
  inverse_cascades: Vec<usize>,

  // Other options (indices of selected items)
  owning_other: Vec<usize>,
  inverse_other: Vec<usize>,

  // List states
  mapping_type_state: ListState,
  entity_type_state: ListState,
  owning_cascades_state: ListState,
  inverse_cascades_state: ListState,
  owning_other_state: ListState,
  inverse_other_state: ListState,

  // Text input cursors
  owning_field_name_cursor: usize,
  inverse_field_name_cursor: usize,

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

impl CreateOneToOneRelationshipForm {
  pub fn new(
    cwd: PathBuf,
    entity_file_b64_src: String,
    entity_file_path: PathBuf,
    _entity_files_json: String, // Not used anymore, we fetch directly
  ) -> Self {
    // Fetch current entity info
    let (current_entity_name, current_entity_package) =
      Self::fetch_current_entity_info(&entity_file_path, &entity_file_b64_src);

    // Fetch all JPA entities and filter out the current one
    let entity_types =
      Self::fetch_target_entities(&cwd, &current_entity_name, &current_entity_package);

    let mut mapping_type_state = ListState::default();
    mapping_type_state.select(Some(0));

    let mut entity_type_state = ListState::default();
    entity_type_state.select(Some(0));

    let mut owning_cascades_state = ListState::default();
    owning_cascades_state.select(Some(0));

    let mut inverse_cascades_state = ListState::default();
    inverse_cascades_state.select(Some(0));

    let mut owning_other_state = ListState::default();
    owning_other_state.select(Some(0));

    let mut inverse_other_state = ListState::default();
    inverse_other_state.select(Some(0));

    Self {
      state: FormState::new(),
      phase: FormPhase::OwningConfiguration,
      mapping_type_index: 0,
      target_entity_index: None,
      owning_field_name: String::new(),
      inverse_field_name: String::new(),
      current_entity_name,
      current_entity_package,
      entity_types,
      owning_cascades: Vec::new(),
      inverse_cascades: Vec::new(),
      owning_other: Vec::new(),
      inverse_other: Vec::new(),
      mapping_type_state,
      entity_type_state,
      owning_cascades_state,
      inverse_cascades_state,
      owning_other_state,
      inverse_other_state,
      owning_field_name_cursor: 0,
      inverse_field_name_cursor: 0,
      focused_field: FocusedField::MappingType,
      cwd,
      entity_file_b64_src,
      entity_file_path,
      should_go_back: false,
      back_pressed_once: false,
    }
  }

  /// Fetch current entity information
  fn fetch_current_entity_info(
    entity_file_path: &Path,
    entity_file_b64_src: &str,
  ) -> (String, String) {
    match get_jpa_entity_info_service::run(Some(entity_file_path), Some(entity_file_b64_src)) {
      Ok(entity_info) => (entity_info.entity_type, entity_info.entity_package_name),
      Err(_) => {
        // Fallback to unknown if service fails
        ("Unknown".to_string(), "unknown".to_string())
      }
    }
  }

  /// Fetch all JPA entities and filter out the current entity
  fn fetch_target_entities(
    cwd: &Path,
    current_entity_name: &str,
    current_entity_package: &str,
  ) -> Vec<EntityTypeInfo> {
    match get_all_jpa_entities_service::run(cwd) {
      Ok(entities) => {
        entities
          .into_iter()
          .filter(|entity| {
            // Exclude the current entity
            !(entity.file_type == current_entity_name
              && entity.file_package_name == current_entity_package)
          })
          .map(|entity| EntityTypeInfo {
            name: entity.file_type,
            package_name: entity.file_package_name,
          })
          .collect()
      }
      Err(_) => {
        // Return empty list if service fails
        Vec::new()
      }
    }
  }

  /// Parse entity files from JSON response (deprecated - kept for compatibility)
  #[allow(dead_code)]
  fn parse_entity_files(json_str: &str) -> Vec<EntityTypeInfo> {
    let mut entity_types = Vec::new();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str)
      && let Some(data) = json.get("data")
      && let Some(files) = data.get("files").and_then(|f| f.as_array())
    {
      for file in files {
        if let (Some(file_type), Some(package_name)) = (
          file.get("fileType").and_then(|t| t.as_str()),
          file.get("filePackageName").and_then(|p| p.as_str()),
        ) {
          entity_types.push(EntityTypeInfo {
            name: file_type.to_string(),
            package_name: package_name.to_string(),
          });
        }
      }
    }

    entity_types
  }

  /// Auto-generate field name from entity type
  fn auto_field_name(type_name: &str) -> String {
    if type_name.is_empty() {
      return String::new();
    }
    let mut chars = type_name.chars();
    if let Some(first) = chars.next() {
      format!("{}{}", first.to_lowercase(), chars.as_str())
    } else {
      String::new()
    }
  }

  /// Update target entity and auto-fill field name
  fn update_target_entity(&mut self) {
    if let Some(idx) = self.entity_type_state.selected() {
      self.target_entity_index = Some(idx);
      if let Some(entity) = self.entity_types.get(idx) {
        self.owning_field_name = Self::auto_field_name(&entity.name);
        self.owning_field_name_cursor = self.owning_field_name.len();
      }
    }
  }

  /// Update mapping type
  fn update_mapping_type(&mut self) {
    if let Some(idx) = self.mapping_type_state.selected() {
      self.mapping_type_index = idx;
    }
  }

  /// Check if bidirectional mapping is selected
  fn is_bidirectional(&self) -> bool {
    self.mapping_type_index == 0
  }

  /// Get mapping type from index
  fn get_mapping_type(&self) -> MappingType {
    match self.mapping_type_index {
      0 => MappingType::BidirectionalJoinColumn,
      _ => MappingType::UnidirectionalJoinColumn,
    }
  }

  /// Get cascade types from indices
  fn get_cascade_types(indices: &[usize]) -> Vec<CascadeType> {
    let all_cascades = Self::get_all_cascades();
    indices.iter().filter_map(|&i| all_cascades.get(i).cloned()).collect()
  }

  /// Get other types from indices
  fn get_other_types(indices: &[usize], is_owning: bool) -> Vec<OtherType> {
    let all_others =
      if is_owning { Self::get_owning_other_options() } else { Self::get_inverse_other_options() };
    indices.iter().filter_map(|&i| all_others.get(i).cloned()).collect()
  }

  /// Get all cascade types
  fn get_all_cascades() -> Vec<CascadeType> {
    vec![
      CascadeType::Persist,
      CascadeType::Merge,
      CascadeType::Remove,
      CascadeType::Refresh,
      CascadeType::Detach,
    ]
  }

  /// Get owning side other options
  fn get_owning_other_options() -> Vec<OtherType> {
    vec![OtherType::Mandatory, OtherType::Unique, OtherType::OrphanRemoval]
  }

  /// Get inverse side other options
  fn get_inverse_other_options() -> Vec<OtherType> {
    vec![OtherType::Mandatory, OtherType::OrphanRemoval]
  }

  /// Toggle item in a list
  fn toggle_in_list(list: &mut Vec<usize>, index: usize) {
    if let Some(pos) = list.iter().position(|&i| i == index) {
      list.remove(pos);
    } else {
      list.push(index);
    }
  }

  /// Move focus to the next visible field
  fn focus_next(&mut self) {
    self.back_pressed_once = false;

    self.focused_field = match self.phase {
      FormPhase::OwningConfiguration => match self.focused_field {
        FocusedField::MappingType => FocusedField::TargetEntityType,
        FocusedField::TargetEntityType => FocusedField::OwningFieldName,
        FocusedField::OwningFieldName => FocusedField::OwningCascades,
        FocusedField::OwningCascades => FocusedField::OwningOther,
        FocusedField::OwningOther => FocusedField::BackButton,
        FocusedField::BackButton => {
          if self.is_bidirectional() {
            FocusedField::NextButton
          } else {
            FocusedField::ConfirmButton
          }
        }
        FocusedField::NextButton => FocusedField::MappingType,
        FocusedField::ConfirmButton => FocusedField::MappingType,
        _ => FocusedField::MappingType,
      },
      FormPhase::InverseConfiguration => match self.focused_field {
        FocusedField::InverseFieldName => FocusedField::InverseCascades,
        FocusedField::InverseCascades => FocusedField::InverseOther,
        FocusedField::InverseOther => FocusedField::BackButton,
        FocusedField::BackButton => FocusedField::ConfirmButton,
        FocusedField::ConfirmButton => FocusedField::InverseFieldName,
        _ => FocusedField::InverseFieldName,
      },
    };
  }

  /// Move focus to the previous visible field
  fn focus_prev(&mut self) {
    self.back_pressed_once = false;

    self.focused_field = match self.phase {
      FormPhase::OwningConfiguration => match self.focused_field {
        FocusedField::MappingType => {
          if self.is_bidirectional() {
            FocusedField::NextButton
          } else {
            FocusedField::ConfirmButton
          }
        }
        FocusedField::TargetEntityType => FocusedField::MappingType,
        FocusedField::OwningFieldName => FocusedField::TargetEntityType,
        FocusedField::OwningCascades => FocusedField::OwningFieldName,
        FocusedField::OwningOther => FocusedField::OwningCascades,
        FocusedField::BackButton => FocusedField::OwningOther,
        FocusedField::NextButton => FocusedField::BackButton,
        FocusedField::ConfirmButton => FocusedField::BackButton,
        _ => FocusedField::MappingType,
      },
      FormPhase::InverseConfiguration => match self.focused_field {
        FocusedField::InverseFieldName => FocusedField::ConfirmButton,
        FocusedField::InverseCascades => FocusedField::InverseFieldName,
        FocusedField::InverseOther => FocusedField::InverseCascades,
        FocusedField::BackButton => FocusedField::InverseOther,
        FocusedField::ConfirmButton => FocusedField::BackButton,
        _ => FocusedField::InverseFieldName,
      },
    };
  }

  /// Check if a field is hidden (not used anymore with phases, but kept for compatibility)
  fn is_field_hidden(&self, _field: FocusedField) -> bool {
    false
  }

  /// Called when entering insert mode
  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    if key == KeyCode::Char('a') {
      match self.focused_field {
        FocusedField::OwningFieldName => {
          self.owning_field_name_cursor = self.owning_field_name.len();
        }
        FocusedField::InverseFieldName => {
          self.inverse_field_name_cursor = self.inverse_field_name.len();
        }
        _ => {}
      }
    }
  }

  /// Called when Enter is pressed in Normal mode
  fn on_enter_pressed(&mut self) {
    match self.focused_field {
      FocusedField::NextButton => {
        // Auto-generate inverse field name from current entity name
        self.inverse_field_name = Self::auto_field_name(&self.current_entity_name);
        self.inverse_field_name_cursor = self.inverse_field_name.len();

        // Move to inverse configuration phase
        self.phase = FormPhase::InverseConfiguration;
        self.focused_field = FocusedField::InverseFieldName;
        self.back_pressed_once = false;
      }
      FocusedField::ConfirmButton => {
        self.execute_create_relationship();
      }
      FocusedField::BackButton => {
        if self.back_pressed_once {
          // If in inverse phase, go back to owning phase
          if self.phase == FormPhase::InverseConfiguration {
            self.phase = FormPhase::OwningConfiguration;
            self.focused_field = FocusedField::MappingType;
            self.back_pressed_once = false;
          } else {
            // If in owning phase, go back to parent form
            self.should_go_back = true;
          }
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
      FocusedField::MappingType => self.handle_mapping_type_insert(key),
      FocusedField::TargetEntityType => self.handle_entity_type_insert(key),
      FocusedField::OwningFieldName => {
        helpers::handle_text_input(
          key,
          &mut self.owning_field_name,
          &mut self.owning_field_name_cursor,
          &mut self.state.input_mode,
        );
      }
      FocusedField::InverseFieldName => {
        helpers::handle_text_input(
          key,
          &mut self.inverse_field_name,
          &mut self.inverse_field_name_cursor,
          &mut self.state.input_mode,
        );
      }
      FocusedField::OwningCascades => self.handle_cascades_insert(key, true),
      FocusedField::InverseCascades => self.handle_cascades_insert(key, false),
      FocusedField::OwningOther => self.handle_other_insert(key, true),
      FocusedField::InverseOther => self.handle_other_insert(key, false),
      FocusedField::NextButton => {
        if key == KeyCode::Enter {
          // Auto-generate inverse field name from current entity name
          self.inverse_field_name = Self::auto_field_name(&self.current_entity_name);
          self.inverse_field_name_cursor = self.inverse_field_name.len();

          self.phase = FormPhase::InverseConfiguration;
          self.focused_field = FocusedField::InverseFieldName;
          self.back_pressed_once = false;
          self.state.input_mode = InputMode::Normal;
        }
      }
      FocusedField::BackButton => {
        if key == KeyCode::Enter {
          if self.back_pressed_once {
            if self.phase == FormPhase::InverseConfiguration {
              self.phase = FormPhase::OwningConfiguration;
              self.focused_field = FocusedField::MappingType;
              self.back_pressed_once = false;
              self.state.input_mode = InputMode::Normal;
            } else {
              self.should_go_back = true;
            }
          } else {
            self.back_pressed_once = true;
          }
        }
      }
      FocusedField::ConfirmButton => {
        if key == KeyCode::Enter {
          self.execute_create_relationship();
        }
      }
    }
  }

  fn handle_mapping_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        helpers::navigate_list_static(&KeyCode::Down, &mut self.mapping_type_state, 2);
        self.update_mapping_type();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        helpers::navigate_list_static(&KeyCode::Up, &mut self.mapping_type_state, 2);
        self.update_mapping_type();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_entity_type_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.entity_types.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.entity_type_state, len);
        self.update_target_entity();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.entity_types.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.entity_type_state, len);
        self.update_target_entity();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
      }
      _ => {}
    }
  }

  fn handle_cascades_insert(&mut self, key: KeyCode, is_owning: bool) {
    let state =
      if is_owning { &mut self.owning_cascades_state } else { &mut self.inverse_cascades_state };
    let list = if is_owning { &mut self.owning_cascades } else { &mut self.inverse_cascades };

    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = Self::get_all_cascades().len();
        helpers::navigate_list_static(&KeyCode::Down, state, len);
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = Self::get_all_cascades().len();
        helpers::navigate_list_static(&KeyCode::Up, state, len);
      }
      KeyCode::Char(' ') | KeyCode::Enter => {
        if let Some(idx) = state.selected() {
          Self::toggle_in_list(list, idx);
        }
        if key == KeyCode::Enter {
          self.state.input_mode = InputMode::Normal;
        }
      }
      _ => {}
    }
  }

  fn handle_other_insert(&mut self, key: KeyCode, is_owning: bool) {
    let state =
      if is_owning { &mut self.owning_other_state } else { &mut self.inverse_other_state };
    let list = if is_owning { &mut self.owning_other } else { &mut self.inverse_other };
    let len = if is_owning {
      Self::get_owning_other_options().len()
    } else {
      Self::get_inverse_other_options().len()
    };

    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        helpers::navigate_list_static(&KeyCode::Down, state, len);
      }
      KeyCode::Char('k') | KeyCode::Up => {
        helpers::navigate_list_static(&KeyCode::Up, state, len);
      }
      KeyCode::Char(' ') | KeyCode::Enter => {
        if let Some(idx) = state.selected() {
          Self::toggle_in_list(list, idx);
        }
        if key == KeyCode::Enter {
          self.state.input_mode = InputMode::Normal;
        }
      }
      _ => {}
    }
  }

  fn execute_create_relationship(&mut self) {
    // Validate
    let target_entity_name = match self.target_entity_index {
      Some(idx) => match self.entity_types.get(idx) {
        Some(entity) => entity.name.clone(),
        None => {
          self.state.error_message = Some("No target entity selected".to_string());
          return;
        }
      },
      None => {
        self.state.error_message = Some("Target entity type is required".to_string());
        return;
      }
    };

    if self.owning_field_name.is_empty() {
      self.state.error_message = Some("Owning side field name is required".to_string());
      return;
    }

    if self.is_bidirectional() && self.inverse_field_name.is_empty() {
      self.state.error_message =
        Some("Inverse side field name is required for bidirectional relationships".to_string());
      return;
    }

    // Build field config
    let field_config = OneToOneFieldConfig {
      inverse_field_type: target_entity_name,
      mapping_type: Some(self.get_mapping_type()),
      owning_side_cascades: Self::get_cascade_types(&self.owning_cascades),
      inverse_side_cascades: Self::get_cascade_types(&self.inverse_cascades),
      owning_side_other: Self::get_other_types(&self.owning_other, true),
      inverse_side_other: Self::get_other_types(&self.inverse_other, false),
    };

    // Call command layer instead of service directly
    let response = create_jpa_one_to_one_relationship_command::execute(
      &self.cwd,
      &self.entity_file_b64_src,
      &self.entity_file_path,
      self.owning_field_name.clone(),
      self.inverse_field_name.clone(),
      field_config,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  /// Check if user wants to go back
  pub fn should_go_back(&self) -> bool {
    self.should_go_back
  }

  // Render methods
  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title_text = "Create new JPA Entity One-to-One Relationship";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
  }

  fn render_mapping_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::MappingType;

    let mapping_types = ["Bidirectional (Recommended)", "Unidirectional with Join Column"];
    let items: Vec<ListItem> = mapping_types
      .iter()
      .enumerate()
      .map(|(i, name)| {
        let is_selected = self.mapping_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, name))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Mapping Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.mapping_type_state);
  }

  fn render_entity_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::TargetEntityType;

    let items: Vec<ListItem> = self
      .entity_types
      .iter()
      .enumerate()
      .map(|(i, entity)| {
        let is_selected = self.entity_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {} ({})", prefix, entity.name, entity.package_name))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Target Entity Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.entity_type_state);
  }

  fn render_text_input(
    &self,
    frame: &mut Frame,
    area: Rect,
    field: FocusedField,
    title: &str,
    text: &str,
    cursor: usize,
  ) {
    if self.is_field_hidden(field) {
      return;
    }

    let is_focused = self.focused_field == field;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title(title, is_focused);
    let input = Paragraph::new(text)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);

    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + cursor as u16 + 1, area.y + 1));
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn render_cascade_selector(
    &self,
    frame: &mut Frame,
    area: Rect,
    field: FocusedField,
    title: &str,
    selected_indices: &[usize],
    state: &mut ListState,
  ) {
    let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
    let params = SelectorParams { field, title, selected_indices };
    Self::render_cascade_selector_static(frame, area, state, &params, &ctx)
  }

  fn render_cascade_selector_static(
    frame: &mut Frame,
    area: Rect,
    state: &mut ListState,
    params: &SelectorParams,
    ctx: &RenderContext,
  ) {
    // Check if field is hidden based on inverse_fields_hidden
    let is_inverse = matches!(params.field, FocusedField::InverseCascades);
    if is_inverse
      && (matches!(ctx.focused_field, FocusedField::MappingType)
        && ctx.form_state.input_mode != InputMode::Normal)
    {
      // This is a simplified check - in practice we'd need inverse_fields_hidden
      // For now, we'll just render it
    }

    let is_focused = ctx.focused_field == params.field;
    let cascades = Self::get_all_cascades();

    let items: Vec<ListItem> = cascades
      .iter()
      .enumerate()
      .map(|(i, cascade)| {
        let is_selected = params.selected_indices.contains(&i);
        let checkbox = if is_selected { "[x]" } else { "[ ]" };
        ListItem::new(format!(" {} {}", checkbox, cascade.as_str()))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title_text = if is_focused && ctx.form_state.input_mode == InputMode::Insert {
      format!("{} [INSERT]", params.title)
    } else {
      params.title.to_string()
    };

    let list = List::new(items)
      .block(Block::default().title(title_text).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, state);
  }

  #[allow(clippy::too_many_arguments)]
  fn render_other_selector(
    &self,
    frame: &mut Frame,
    area: Rect,
    field: FocusedField,
    title: &str,
    selected_indices: &[usize],
    state: &mut ListState,
    is_owning: bool,
  ) {
    let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
    let params = SelectorParams { field, title, selected_indices };
    Self::render_other_selector_static(frame, area, state, &params, is_owning, &ctx)
  }

  fn render_other_selector_static(
    frame: &mut Frame,
    area: Rect,
    state: &mut ListState,
    params: &SelectorParams,
    is_owning: bool,
    ctx: &RenderContext,
  ) {
    let is_focused = ctx.focused_field == params.field;
    let options =
      if is_owning { Self::get_owning_other_options() } else { Self::get_inverse_other_options() };

    let items: Vec<ListItem> = options
      .iter()
      .enumerate()
      .map(|(i, other)| {
        let is_selected = params.selected_indices.contains(&i);
        let checkbox = if is_selected { "[x]" } else { "[ ]" };
        let display = match other {
          OtherType::Mandatory => "Mandatory",
          OtherType::Unique => "Unique",
          OtherType::OrphanRemoval => "Orphan Removal",
          OtherType::LargeObject => "Large Object",
          OtherType::EqualsHashcode => "Equals/Hashcode",
          OtherType::Mutable => "Mutable",
        };
        ListItem::new(format!(" {} {}", checkbox, display))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title_text = if is_focused && ctx.form_state.input_mode == InputMode::Insert {
      format!("{} [INSERT]", params.title)
    } else {
      params.title.to_string()
    };

    let list = List::new(items)
      .block(Block::default().title(title_text).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, state);
  }

  fn render_owning_phase_buttons(&self, frame: &mut Frame, area: Rect) {
    if self.is_bidirectional() {
      // Show Back + Next for bidirectional
      button_helpers::render_two_button_layout(
        frame,
        area,
        self.focused_field == FocusedField::BackButton,
        self.focused_field == FocusedField::NextButton,
        self.back_pressed_once,
        self.state.escape_handler.pressed_once,
        button_helpers::ButtonType::Next,
      );
    } else {
      // Show Back + Confirm for unidirectional
      button_helpers::render_two_button_layout(
        frame,
        area,
        self.focused_field == FocusedField::BackButton,
        self.focused_field == FocusedField::ConfirmButton,
        self.back_pressed_once,
        self.state.escape_handler.pressed_once,
        button_helpers::ButtonType::Confirm,
      );
    }
  }

  fn render_buttons(&self, frame: &mut Frame, area: Rect) {
    button_helpers::render_two_button_layout(
      frame,
      area,
      self.focused_field == FocusedField::BackButton,
      self.focused_field == FocusedField::ConfirmButton,
      self.back_pressed_once,
      self.state.escape_handler.pressed_once,
      button_helpers::ButtonType::Confirm,
    );
  }

  pub fn render(&mut self, frame: &mut Frame) {
    match self.phase {
      FormPhase::OwningConfiguration => self.render_owning_phase(frame),
      FormPhase::InverseConfiguration => self.render_inverse_phase(frame),
    }
  }

  fn render_owning_phase(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let constraints = vec![
      Constraint::Length(2), // Title
      Constraint::Length(4), // Mapping type
      Constraint::Length(9), // Entity type
      Constraint::Length(3), // Owning field name
      Constraint::Length(7), // Owning cascades
      Constraint::Length(5), // Owning other
      Constraint::Min(0),    // Errors
      Constraint::Length(1), // Buttons
    ];

    let chunks =
      Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let mut idx = 0;

    // Title
    let title_text = "Create JPA One-to-One Relationship - Step 1: Owning Side";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[idx]);
    idx += 1;

    self.render_mapping_type_selector(frame, chunks[idx]);
    idx += 1;

    self.render_entity_type_selector(frame, chunks[idx]);
    idx += 1;

    self.render_text_input(
      frame,
      chunks[idx],
      FocusedField::OwningFieldName,
      "Owning Side Field Name",
      &self.owning_field_name,
      self.owning_field_name_cursor,
    );
    idx += 1;

    // Render cascade selectors
    {
      let owning_cascades = &self.owning_cascades;
      let owning_cascades_state = &mut self.owning_cascades_state;
      let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
      let params = SelectorParams {
        field: FocusedField::OwningCascades,
        title: "Owning Side Cascade Types (Space to toggle)",
        selected_indices: owning_cascades,
      };
      Self::render_cascade_selector_static(
        frame,
        chunks[idx],
        owning_cascades_state,
        &params,
        &ctx,
      );
    }
    idx += 1;

    {
      let owning_other = &self.owning_other;
      let owning_other_state = &mut self.owning_other_state;
      let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
      let params = SelectorParams {
        field: FocusedField::OwningOther,
        title: "Owning Side Options (Space to toggle)",
        selected_indices: owning_other,
      };
      Self::render_other_selector_static(
        frame,
        chunks[idx],
        owning_other_state,
        &params,
        true,
        &ctx,
      );
    }
    idx += 1;

    // Render error if present
    if let Some(ref error_msg) = self.state.error_message {
      let error_paragraph =
        Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
          Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        );
      frame.render_widget(error_paragraph, chunks[idx]);
    }
    idx += 1;

    self.render_owning_phase_buttons(frame, chunks[idx]);
  }

  fn render_inverse_phase(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let constraints = vec![
      Constraint::Length(2), // Title
      Constraint::Length(3), // Inverse field name
      Constraint::Length(7), // Inverse cascades
      Constraint::Length(4), // Inverse other
      Constraint::Min(0),    // Errors
      Constraint::Length(1), // Buttons
    ];

    let chunks =
      Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let mut idx = 0;

    // Title
    let title_text = "Create JPA One-to-One Relationship - Step 2: Inverse Side";
    let title = Paragraph::new(title_text)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[idx]);
    idx += 1;

    self.render_text_input(
      frame,
      chunks[idx],
      FocusedField::InverseFieldName,
      "Inverse Side Field Name",
      &self.inverse_field_name,
      self.inverse_field_name_cursor,
    );
    idx += 1;

    // Render cascade selectors
    {
      let inverse_cascades = &self.inverse_cascades;
      let inverse_cascades_state = &mut self.inverse_cascades_state;
      let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
      let params = SelectorParams {
        field: FocusedField::InverseCascades,
        title: "Inverse Side Cascade Types (Space to toggle)",
        selected_indices: inverse_cascades,
      };
      Self::render_cascade_selector_static(
        frame,
        chunks[idx],
        inverse_cascades_state,
        &params,
        &ctx,
      );
    }
    idx += 1;

    {
      let inverse_other = &self.inverse_other;
      let inverse_other_state = &mut self.inverse_other_state;
      let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
      let params = SelectorParams {
        field: FocusedField::InverseOther,
        title: "Inverse Side Options (Space to toggle)",
        selected_indices: inverse_other,
      };
      Self::render_other_selector_static(
        frame,
        chunks[idx],
        inverse_other_state,
        &params,
        false,
        &ctx,
      );
    }
    idx += 1;

    // Render error if present
    if let Some(ref error_msg) = self.state.error_message {
      let error_paragraph =
        Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
          Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        );
      frame.render_widget(error_paragraph, chunks[idx]);
    }
    idx += 1;

    self.render_buttons(frame, chunks[idx]);
  }
}

// Implement the FormBehavior trait
impl FormBehavior for CreateOneToOneRelationshipForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateOneToOneRelationshipForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateOneToOneRelationshipForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateOneToOneRelationshipForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateOneToOneRelationshipForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateOneToOneRelationshipForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateOneToOneRelationshipForm::render(self, frame)
  }
}
