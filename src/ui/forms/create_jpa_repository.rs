use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::path::{Path, PathBuf};

use crate::commands::services::get_jpa_entity_info_service;
use crate::commands::{
  create_jpa_repository_command, get_all_packages_command, get_java_basic_types_command,
};
use crate::common::types::java_basic_types::JavaBasicType;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, button_helpers, helpers};

/// Represents which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  IdFieldType,
  PackageName,
  ConfirmButton,
}

/// Stores information about a Java type (for ID field selection)
#[derive(Debug, Clone)]
struct JavaTypeInfo {
  name: String,
  package_path: String,
}

/// Main form state for creating a JPA repository
pub struct CreateJpaRepositoryForm {
  // Common form state (embedded)
  state: FormState,

  // Entity information (provided at initialization)
  cwd: PathBuf,
  entity_file_path: PathBuf,
  entity_file_b64_src: String,

  // Available ID types from get-java-basic-types
  available_id_types: Vec<JavaTypeInfo>,
  id_type_state: ListState,
  selected_id_type_index: Option<usize>,

  // Package autocomplete
  package_list: Vec<String>,
  filtered_packages: Vec<String>,
  package_autocomplete_selected_index: Option<usize>,
  package_autocomplete_scroll_offset: usize,
  show_package_autocomplete: bool,

  // Field values
  id_field_type: String,    // The selected/detected ID type name (e.g., "Long")
  id_field_package: String, // The package path (e.g., "java.lang")
  package_name: String,     // Repository package name

  // Text input states
  package_name_cursor: usize,

  // Focus management
  focused_field: FocusedField,

  // Auto-detection status
  auto_detected_id: bool,
  entity_name: String,
}

impl CreateJpaRepositoryForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    // Fetch available ID types
    let available_id_types = Self::fetch_id_types().unwrap_or_else(|_| {
      vec![
        JavaTypeInfo { name: "Long".to_string(), package_path: "java.lang".to_string() },
        JavaTypeInfo { name: "Integer".to_string(), package_path: "java.lang".to_string() },
        JavaTypeInfo { name: "String".to_string(), package_path: "java.lang".to_string() },
        JavaTypeInfo { name: "UUID".to_string(), package_path: "java.util".to_string() },
      ]
    });

    // Fetch packages for autocomplete
    let package_list = Self::fetch_packages(&cwd).unwrap_or_else(|_| {
      vec![
        "com.example".to_string(),
        "com.example.repository".to_string(),
        "com.example.model.repository".to_string(),
      ]
    });

    // Extract entity name from file path
    let entity_name =
      entity_file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Entity").to_string();

    // Try to detect ID field from entity
    let (id_field_type, id_field_package, auto_detected) =
      Self::try_detect_id_field(&entity_file_path, &entity_file_b64_src, &available_id_types);

    // Set default repository package (same as entity package)
    let entity_package = Self::extract_entity_package(&entity_file_path);
    let default_repo_package =
      if entity_package.is_empty() { "com.example".to_string() } else { entity_package.clone() };

    // Find the index of the detected/default ID type in the list
    let mut id_type_state = ListState::default();
    let selected_id_type_index = available_id_types
      .iter()
      .position(|t| t.name == id_field_type && t.package_path == id_field_package)
      .or(Some(0)); // Default to first if not found

    if let Some(idx) = selected_id_type_index {
      id_type_state.select(Some(idx));
    }

    Self {
      state: FormState::new(),
      cwd,
      entity_file_path,
      entity_file_b64_src,
      available_id_types,
      id_type_state,
      selected_id_type_index,
      package_list,
      filtered_packages: Vec::new(),
      package_autocomplete_selected_index: None,
      package_autocomplete_scroll_offset: 0,
      show_package_autocomplete: false,
      id_field_type,
      id_field_package,
      package_name: default_repo_package.clone(),
      package_name_cursor: default_repo_package.len(),
      focused_field: FocusedField::IdFieldType,
      auto_detected_id: auto_detected,
      entity_name,
    }
  }

  /// Fetch available ID types from get-java-basic-types command
  fn fetch_id_types() -> Result<Vec<JavaTypeInfo>, Box<dyn std::error::Error>> {
    let response = get_java_basic_types_command::execute(&JavaBasicType::IdTypes);

    let mut types = Vec::new();
    if let Some(data) = response.data {
      for type_info in data {
        types.push(JavaTypeInfo {
          name: type_info.name,
          package_path: type_info.package_path.unwrap_or_else(|| "java.lang".to_string()),
        });
      }
    }

    Ok(types)
  }

  /// Fetch packages for autocomplete from get-all-packages command
  fn fetch_packages(cwd: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = get_all_packages_command::execute(cwd, &JavaSourceDirectoryType::Main);

    if let Some(data) = response.data {
      Ok(data.packages.iter().map(|p| p.package_name.clone()).collect())
    } else {
      Err("Failed to fetch packages".into())
    }
  }

  /// Try to detect ID field from entity using get_jpa_entity_info_service
  fn try_detect_id_field(
    entity_file_path: &Path,
    entity_file_b64_src: &str,
    available_types: &[JavaTypeInfo],
  ) -> (String, String, bool) {
    // Try to get entity info
    let entity_info_result =
      get_jpa_entity_info_service::run(Some(entity_file_path), Some(entity_file_b64_src));

    if let Ok(entity_info) = entity_info_result
      && let (Some(id_type), Some(id_package)) =
        (entity_info.id_field_type, entity_info.id_field_package_name)
    {
      return (id_type, id_package, true);
    }

    // Fallback: default to Long
    (
      available_types.first().map(|t| t.name.clone()).unwrap_or_else(|| "Long".to_string()),
      available_types
        .first()
        .map(|t| t.package_path.clone())
        .unwrap_or_else(|| "java.lang".to_string()),
      false,
    )
  }

  /// Extract entity package from file path
  fn extract_entity_package(entity_file_path: &Path) -> String {
    // Try to extract package from path like: .../src/main/java/com/example/demo/Entity.java
    let path_str = entity_file_path.to_string_lossy();

    if let Some(java_idx) = path_str.find("/src/main/java/") {
      let after_java = &path_str[java_idx + 15..]; // Skip "/src/main/java/"
      if let Some(last_slash) = after_java.rfind('/') {
        let package_path = &after_java[..last_slash];
        return package_path.replace('/', ".");
      }
    }

    String::new()
  }

  /// Update selected ID type from list state
  fn update_id_type_from_selection(&mut self) {
    if let Some(idx) = self.id_type_state.selected()
      && let Some(type_info) = self.available_id_types.get(idx)
    {
      self.id_field_type = type_info.name.clone();
      self.id_field_package = type_info.package_path.clone();
      self.selected_id_type_index = Some(idx);
    }
  }

  /// Move focus to the next field
  fn focus_next(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::IdFieldType => FocusedField::PackageName,
      FocusedField::PackageName => FocusedField::ConfirmButton,
      FocusedField::ConfirmButton => FocusedField::IdFieldType,
    };
  }

  /// Move focus to the previous field
  fn focus_prev(&mut self) {
    self.focused_field = match self.focused_field {
      FocusedField::IdFieldType => FocusedField::ConfirmButton,
      FocusedField::PackageName => FocusedField::IdFieldType,
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

  /// Called when entering insert mode - 'a' moves cursor to end for text inputs
  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    if key == KeyCode::Char('a') && self.focused_field == FocusedField::PackageName {
      self.package_name_cursor = self.package_name.len();
    }
  }

  /// Called when Enter is pressed in Normal mode
  fn on_enter_pressed(&mut self) {
    if self.focused_field == FocusedField::ConfirmButton {
      self.execute_create_jpa_repository();
    }
  }

  /// Handle field-specific input in Insert mode
  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::IdFieldType => self.handle_id_type_selector_insert(key),
      FocusedField::PackageName => self.handle_package_name_input(key),
      FocusedField::ConfirmButton => {
        self.state.input_mode = InputMode::Normal;
      }
    }
  }

  fn handle_id_type_selector_insert(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char('j') | KeyCode::Down => {
        let len = self.available_id_types.len();
        helpers::navigate_list_static(&KeyCode::Down, &mut self.id_type_state, len);
        self.update_id_type_from_selection();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        let len = self.available_id_types.len();
        helpers::navigate_list_static(&KeyCode::Up, &mut self.id_type_state, len);
        self.update_id_type_from_selection();
      }
      KeyCode::Enter => {
        self.state.input_mode = InputMode::Normal;
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

  fn execute_create_jpa_repository(&mut self) {
    // Validate package name
    if self.package_name.trim().is_empty() {
      self.state.error_message = Some("Repository package name is required".to_string());
      return;
    }

    // Call command layer to create repository with manual ID
    let response = create_jpa_repository_command::execute_with_manual_id(
      &self.cwd,
      &self.entity_file_b64_src,
      self.entity_file_path.as_path(),
      &self.id_field_type,
      &self.id_field_package,
    );

    // Use helper function to output response and exit
    helpers::output_response_and_exit(response, &mut self.state);
  }

  fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let title = format!("Create JPA Repository for {}", self.entity_name);
    let title_widget = Paragraph::new(title)
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title_widget, area);
  }

  fn render_id_type_selector(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::IdFieldType;

    let items: Vec<ListItem> = self
      .available_id_types
      .iter()
      .enumerate()
      .map(|(i, type_info)| {
        let is_selected = self.id_type_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        let label = format!(" {} {} ({})", prefix, type_info.name, type_info.package_path);
        ListItem::new(label)
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title_text = if self.auto_detected_id {
      format!(
        "ID Field Type (auto-detected: {} from {})",
        self.id_field_type, self.id_field_package
      )
    } else {
      "ID Field Type (not detected - please select)".to_string()
    };

    let title = self.generate_title(&title_text, is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut self.id_type_state);
  }

  fn render_package_name_input(&mut self, frame: &mut Frame, area: Rect) {
    let is_focused = self.focused_field == FocusedField::PackageName;
    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title =
      if is_focused && self.show_package_autocomplete && !self.filtered_packages.is_empty() {
        format!(
          "Repository Package [↑↓ navigate, Tab/Enter select, Esc cancel] ({} matches)",
          self.filtered_packages.len()
        )
      } else {
        self.generate_title("Repository Package", is_focused)
      };

    let input = Paragraph::new(self.package_name.as_str())
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    frame.render_widget(input, area);

    if is_focused && self.state.input_mode == InputMode::Insert {
      frame.set_cursor_position((area.x + self.package_name_cursor as u16 + 1, area.y + 1));
    }
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

  fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
    let info_text = format!(
      "Repository: {}Repository\nEntity: {} ({})\nID Type: {} ({})",
      self.entity_name,
      self.entity_name,
      Self::extract_entity_package(&self.entity_file_path),
      self.id_field_type,
      self.id_field_package
    );

    let info = Paragraph::new(info_text)
      .block(
        Block::default()
          .title("Repository Details")
          .borders(Borders::ALL)
          .border_style(Style::default().fg(Color::Blue)),
      )
      .wrap(Wrap { trim: true });
    frame.render_widget(info, area);
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

  pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    // Calculate ID type list height dynamically (max 6 lines)
    let id_type_height = (self.available_id_types.len() as u16 + 2).min(6);

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2),              // Title bar
        Constraint::Length(5),              // Info panel
        Constraint::Length(id_type_height), // ID type selector (dynamic, max 6)
        Constraint::Length(3),              // Package name input
        Constraint::Min(3),                 // Flexible space for autocomplete + errors
        Constraint::Length(1),              // Confirm button
      ])
      .split(area);

    self.render_title_bar(frame, chunks[0]);
    self.render_info_panel(frame, chunks[1]);
    self.render_id_type_selector(frame, chunks[2]);
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
impl FormBehavior for CreateJpaRepositoryForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    CreateJpaRepositoryForm::focus_next(self)
  }

  fn focus_prev(&mut self) {
    CreateJpaRepositoryForm::focus_prev(self)
  }

  fn on_enter_insert_mode(&mut self, key: KeyCode) {
    CreateJpaRepositoryForm::on_enter_insert_mode(self, key)
  }

  fn on_enter_pressed(&mut self) {
    CreateJpaRepositoryForm::on_enter_pressed(self)
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    CreateJpaRepositoryForm::handle_field_insert(self, key)
  }

  fn render(&mut self, frame: &mut Frame) {
    CreateJpaRepositoryForm::render(self, frame)
  }
}
