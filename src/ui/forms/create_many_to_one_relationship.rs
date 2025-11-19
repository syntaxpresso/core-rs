#![allow(dead_code)]

use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::create_jpa_many_to_one_relationship_command;
use crate::commands::services::{get_all_jpa_entities_service, get_jpa_entity_info_service};
use crate::common::types::cascade_type::CascadeType;
use crate::common::types::collection_type::CollectionType;
use crate::common::types::fetch_type::FetchType;
use crate::common::types::many_to_one_field_config::ManyToOneFieldConfig;
use crate::common::types::mapping_type::MappingType;
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
  // Phase 1: Owning Configuration (Many side)
  MappingType,
  TargetEntityType,
  OwningFieldName,
  InverseFieldName,
  FetchType,
  OwningCascades,
  OwningOther,

  // Phase 2: Inverse Configuration (One side -> becomes OneToMany)
  CollectionType,
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

/// Main form state for creating many-to-one relationships
pub struct CreateManyToOneRelationshipForm {
  // Common form state (embedded)
  state: FormState,

  // Current phase
  phase: FormPhase,

  // Field values
  mapping_type_index: usize,
  target_entity_index: Option<usize>,
  owning_field_name: String,
  inverse_field_name: String,
  fetch_type_index: usize,
  collection_type_index: usize,

  // Current entity information (owning side - Many side)
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
  fetch_type_state: ListState,
  collection_type_state: ListState,
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

impl CreateManyToOneRelationshipForm {
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

    let mut fetch_type_state = ListState::default();
    fetch_type_state.select(Some(0));

    let mut collection_type_state = ListState::default();
    collection_type_state.select(Some(0));

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
      fetch_type_index: 0,
      collection_type_index: 0,
      current_entity_name,
      current_entity_package,
      entity_types,
      owning_cascades: Vec::new(),
      inverse_cascades: Vec::new(),
      owning_other: Vec::new(),
      inverse_other: Vec::new(),
      mapping_type_state,
      entity_type_state,
      fetch_type_state,
      collection_type_state,
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
      Ok(entities) => entities
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
        .collect(),
      Err(_) => {
        // Return empty list if service fails
        Vec::new()
      }
    }
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

  /// Auto-generate inverse field name from entity type and collection type
  fn auto_inverse_field_name(entity_name: &str, collection_type_index: usize) -> String {
    if entity_name.is_empty() {
      return String::new();
    }
    let collection_suffix = match collection_type_index {
      0 => "List",       // List
      1 => "Set",        // Set
      _ => "Collection", // Collection
    };
    // Convert to camelCase: entityNameList, entityNameSet, entityNameCollection
    let base_name = Self::auto_field_name(entity_name);
    format!("{}{}", base_name, collection_suffix)
  }

  /// Update target entity and auto-fill owning field name
  fn update_target_entity(&mut self) {
    if let Some(idx) = self.entity_type_state.selected() {
      self.target_entity_index = Some(idx);
      if let Some(entity) = self.entity_types.get(idx) {
        // Owning side (Many side): single reference to target entity
        self.owning_field_name = Self::auto_field_name(&entity.name);
        self.owning_field_name_cursor = self.owning_field_name.len();
      }
    }
  }

  /// Update inverse field name when collection type or entering inverse phase
  fn update_inverse_field_name(&mut self) {
    self.inverse_field_name =
      Self::auto_inverse_field_name(&self.current_entity_name, self.collection_type_index);
    self.inverse_field_name_cursor = self.inverse_field_name.len();
  }

  /// Update mapping type
  fn update_mapping_type(&mut self) {
    if let Some(idx) = self.mapping_type_state.selected() {
      self.mapping_type_index = idx;
    }
  }

  /// Update fetch type
  fn update_fetch_type(&mut self) {
    if let Some(idx) = self.fetch_type_state.selected() {
      self.fetch_type_index = idx;
    }
  }

  /// Update collection type
  fn update_collection_type(&mut self) {
    if let Some(idx) = self.collection_type_state.selected() {
      self.collection_type_index = idx;
      // Auto-update inverse field name when collection type changes
      self.update_inverse_field_name();
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

  /// Get fetch type from index
  fn get_fetch_type(&self) -> FetchType {
    match self.fetch_type_index {
      0 => FetchType::Lazy,
      _ => FetchType::Eager,
    }
  }

  /// Get collection type from index
  fn get_collection_type(&self) -> CollectionType {
    match self.collection_type_index {
      0 => CollectionType::List,
      1 => CollectionType::Set,
      _ => CollectionType::Collection,
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

  /// Get owning side other options (Many side)
  fn get_owning_other_options() -> Vec<OtherType> {
    vec![OtherType::Mandatory, OtherType::Unique]
  }

  /// Get inverse side other options (One side)
  fn get_inverse_other_options() -> Vec<OtherType> {
    vec![OtherType::OrphanRemoval]
  }

  /// Toggle item in a list
  fn toggle_in_list(list: &mut Vec<usize>, index: usize) {
    if let Some(pos) = list.iter().position(|&i| i == index) {
      list.remove(pos);
    } else {
      list.push(index);
    }
  }

  /// Check if user wants to go back
  pub fn should_go_back(&self) -> bool {
    self.should_go_back
  }

  /// Generate title with insert mode indicator
  fn generate_title(&self, base: &str, is_focused: bool) -> String {
    if is_focused && self.state.input_mode == InputMode::Insert {
      format!("{} [INSERT]", base)
    } else {
      base.to_string()
    }
  }

  /// Move focus to the next visible field
  fn focus_next(&mut self) {
    self.back_pressed_once = false;

    self.focused_field = match self.phase {
      FormPhase::OwningConfiguration => match self.focused_field {
        FocusedField::MappingType => FocusedField::TargetEntityType,
        FocusedField::TargetEntityType => FocusedField::OwningFieldName,
        FocusedField::OwningFieldName => FocusedField::FetchType,
        FocusedField::FetchType => FocusedField::OwningCascades,
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
        FocusedField::CollectionType => FocusedField::InverseFieldName,
        FocusedField::InverseFieldName => FocusedField::InverseCascades,
        FocusedField::InverseCascades => FocusedField::InverseOther,
        FocusedField::InverseOther => FocusedField::BackButton,
        FocusedField::BackButton => FocusedField::ConfirmButton,
        FocusedField::ConfirmButton => FocusedField::CollectionType,
        _ => FocusedField::CollectionType,
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
        FocusedField::FetchType => FocusedField::OwningFieldName,
        FocusedField::OwningCascades => FocusedField::FetchType,
        FocusedField::OwningOther => FocusedField::OwningCascades,
        FocusedField::BackButton => FocusedField::OwningOther,
        FocusedField::NextButton => FocusedField::BackButton,
        FocusedField::ConfirmButton => FocusedField::BackButton,
        _ => FocusedField::MappingType,
      },
      FormPhase::InverseConfiguration => match self.focused_field {
        FocusedField::CollectionType => FocusedField::ConfirmButton,
        FocusedField::InverseFieldName => FocusedField::CollectionType,
        FocusedField::InverseCascades => FocusedField::InverseFieldName,
        FocusedField::InverseOther => FocusedField::InverseCascades,
        FocusedField::BackButton => FocusedField::InverseOther,
        FocusedField::ConfirmButton => FocusedField::BackButton,
        _ => FocusedField::CollectionType,
      },
    };
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
        // Move to inverse configuration phase
        self.phase = FormPhase::InverseConfiguration;
        self.focused_field = FocusedField::CollectionType;
        self.back_pressed_once = false;
        // Auto-generate inverse field name based on current collection type
        self.update_inverse_field_name();
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
    let field_config = ManyToOneFieldConfig {
      inverse_field_type: target_entity_name,
      fetch_type: self.get_fetch_type(),
      collection_type: self.get_collection_type(),
      mapping_type: Some(self.get_mapping_type()),
      owning_side_cascades: Self::get_cascade_types(&self.owning_cascades),
      inverse_side_cascades: Self::get_cascade_types(&self.inverse_cascades),
      owning_side_other: Self::get_other_types(&self.owning_other, true),
      inverse_side_other: Self::get_other_types(&self.inverse_other, false),
    };

    // Call command layer instead of service directly
    let response = create_jpa_many_to_one_relationship_command::execute(
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

  /// Handle field-specific input in Insert mode
  fn handle_field_insert_impl(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::MappingType => match key {
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
      },
      FocusedField::TargetEntityType => match key {
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
      },
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
      FocusedField::FetchType => match key {
        KeyCode::Char('j') | KeyCode::Down => {
          helpers::navigate_list_static(&KeyCode::Down, &mut self.fetch_type_state, 2);
          self.update_fetch_type();
        }
        KeyCode::Char('k') | KeyCode::Up => {
          helpers::navigate_list_static(&KeyCode::Up, &mut self.fetch_type_state, 2);
          self.update_fetch_type();
        }
        KeyCode::Enter => {
          self.state.input_mode = InputMode::Normal;
        }
        _ => {}
      },
      FocusedField::CollectionType => match key {
        KeyCode::Char('j') | KeyCode::Down => {
          helpers::navigate_list_static(&KeyCode::Down, &mut self.collection_type_state, 3);
          self.update_collection_type();
        }
        KeyCode::Char('k') | KeyCode::Up => {
          helpers::navigate_list_static(&KeyCode::Up, &mut self.collection_type_state, 3);
          self.update_collection_type();
        }
        KeyCode::Enter => {
          self.state.input_mode = InputMode::Normal;
        }
        _ => {}
      },
      FocusedField::OwningCascades | FocusedField::InverseCascades => {
        let state = if matches!(self.focused_field, FocusedField::OwningCascades) {
          &mut self.owning_cascades_state
        } else {
          &mut self.inverse_cascades_state
        };
        let list = if matches!(self.focused_field, FocusedField::OwningCascades) {
          &mut self.owning_cascades
        } else {
          &mut self.inverse_cascades
        };

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
      FocusedField::OwningOther | FocusedField::InverseOther => {
        let state = if matches!(self.focused_field, FocusedField::OwningOther) {
          &mut self.owning_other_state
        } else {
          &mut self.inverse_other_state
        };
        let list = if matches!(self.focused_field, FocusedField::OwningOther) {
          &mut self.owning_other
        } else {
          &mut self.inverse_other
        };
        let len = if matches!(self.focused_field, FocusedField::OwningOther) {
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
      FocusedField::NextButton => {
        if key == KeyCode::Enter {
          self.phase = FormPhase::InverseConfiguration;
          self.focused_field = FocusedField::CollectionType;
          self.back_pressed_once = false;
          self.state.input_mode = InputMode::Normal;
          // Auto-generate inverse field name based on current collection type
          self.update_inverse_field_name();
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

  pub fn render_impl(&mut self, frame: &mut Frame) {
    match self.phase {
      FormPhase::OwningConfiguration => self.render_owning_phase(frame),
      FormPhase::InverseConfiguration => self.render_inverse_phase(frame),
    }
  }

  fn render_owning_phase(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2), // Title
        Constraint::Length(4), // Mapping type
        Constraint::Length(9), // Entity type
        Constraint::Length(3), // Owning field name
        Constraint::Length(4), // Fetch type
        Constraint::Length(7), // Owning cascades
        Constraint::Length(4), // Owning other
        Constraint::Min(0),    // Errors
        Constraint::Length(1), // Buttons
      ])
      .split(area);

    let mut idx = 0;

    // Title
    let title = Paragraph::new("Many-to-One: Owning Side Configuration")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[idx]);
    idx += 1;

    // Mapping type
    self.render_mapping_type_selector(frame, chunks[idx]);
    idx += 1;

    // Entity type
    self.render_entity_type_selector(frame, chunks[idx]);
    idx += 1;

    // Owning field name
    self.render_text_input(
      frame,
      chunks[idx],
      FocusedField::OwningFieldName,
      "Owning Side Field Name (Many side)",
      &self.owning_field_name.clone(),
      self.owning_field_name_cursor,
    );
    idx += 1;

    // Fetch type
    self.render_fetch_type_selector(frame, chunks[idx]);
    idx += 1;

    // Cascades
    let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
    let params = SelectorParams {
      field: FocusedField::OwningCascades,
      title: "Owning Side Cascade Types (Space to toggle)",
      selected_indices: &self.owning_cascades,
    };
    Self::render_cascade_selector_static(
      frame,
      chunks[idx],
      &mut self.owning_cascades_state,
      &params,
      &ctx,
    );
    idx += 1;

    // Other options
    let params = SelectorParams {
      field: FocusedField::OwningOther,
      title: "Owning Side Options (Space to toggle)",
      selected_indices: &self.owning_other,
    };
    Self::render_other_selector_static(
      frame,
      chunks[idx],
      &mut self.owning_other_state,
      &params,
      true,
      &ctx,
    );
    idx += 1;

    // Error
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

    // Buttons
    self.render_owning_phase_buttons(frame, chunks[idx]);
  }

  fn render_inverse_phase(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2), // Title
        Constraint::Length(5), // Collection type
        Constraint::Length(3), // Inverse field name
        Constraint::Length(7), // Inverse cascades
        Constraint::Length(3), // Inverse other
        Constraint::Min(0),    // Errors
        Constraint::Length(1), // Buttons
      ])
      .split(area);

    let mut idx = 0;

    // Title
    let title = Paragraph::new("Many-to-One: Inverse Side (One-to-Many) Configuration")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[idx]);
    idx += 1;

    // Collection type
    self.render_collection_type_selector(frame, chunks[idx]);
    idx += 1;

    // Inverse field name
    self.render_text_input(
      frame,
      chunks[idx],
      FocusedField::InverseFieldName,
      "Inverse Side Field Name (One side)",
      &self.inverse_field_name.clone(),
      self.inverse_field_name_cursor,
    );
    idx += 1;

    // Cascades
    let ctx = RenderContext { focused_field: self.focused_field, form_state: &self.state };
    let params = SelectorParams {
      field: FocusedField::InverseCascades,
      title: "Inverse Side Cascade Types (Space to toggle)",
      selected_indices: &self.inverse_cascades,
    };
    Self::render_cascade_selector_static(
      frame,
      chunks[idx],
      &mut self.inverse_cascades_state,
      &params,
      &ctx,
    );
    idx += 1;

    // Other options
    let params = SelectorParams {
      field: FocusedField::InverseOther,
      title: "Inverse Side Options (Space to toggle)",
      selected_indices: &self.inverse_other,
    };
    Self::render_other_selector_static(
      frame,
      chunks[idx],
      &mut self.inverse_other_state,
      &params,
      false,
      &ctx,
    );
    idx += 1;

    // Error
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

    // Buttons
    self.render_buttons(frame, chunks[idx]);
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

    let title = self.generate_title("Target Entity Type (the 'One' side)", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.entity_type_state);
  }

  fn render_fetch_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::FetchType;

    let fetch_types = ["Lazy (Default)", "Eager"];
    let items: Vec<ListItem> = fetch_types
      .iter()
      .enumerate()
      .map(|(i, name)| {
        let is_selected = self.fetch_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, name))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Fetch Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.fetch_type_state);
  }

  fn render_collection_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::CollectionType;

    let collection_types = ["List", "Set", "Collection"];
    let items: Vec<ListItem> = collection_types
      .iter()
      .enumerate()
      .map(|(i, name)| {
        let is_selected = self.collection_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, name))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Collection Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.collection_type_state);
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
}

// Implement the FormBehavior trait
impl FormBehavior for CreateManyToOneRelationshipForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateManyToOneRelationshipForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateManyToOneRelationshipForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateManyToOneRelationshipForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateManyToOneRelationshipForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateManyToOneRelationshipForm::handle_field_insert_impl(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateManyToOneRelationshipForm::render_impl(self, frame)
  }
}
