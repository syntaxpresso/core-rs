use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::{Path, PathBuf};

use crate::commands::get_all_jpa_entities_command;
use crate::ui::form_trait::{FormBehavior, FormState, InputMode, button_helpers, helpers};
use crate::ui::forms::create_many_to_one_relationship::CreateManyToOneRelationshipForm;
use crate::ui::forms::create_one_to_one_relationship::CreateOneToOneRelationshipForm;

/// Enum to hold different child form types
enum ChildFormType {
  OneToOne(Box<CreateOneToOneRelationshipForm>),
  ManyToOne(Box<CreateManyToOneRelationshipForm>),
  // OneToMany(Box<CreateOneToManyRelationshipForm>), // Future
  // ManyToMany(Box<CreateManyToManyRelationshipForm>), // Future
}

/// Relationship category options
#[derive(Debug, Clone, Copy, PartialEq)]
enum RelationshipCategory {
  OneToOne,
  ManyToOne,
  OneToMany,
  ManyToMany,
}

impl RelationshipCategory {
  fn all() -> Vec<RelationshipCategory> {
    vec![
      RelationshipCategory::OneToOne,
      RelationshipCategory::ManyToOne,
      RelationshipCategory::OneToMany,
      RelationshipCategory::ManyToMany,
    ]
  }

  fn as_str(&self) -> &'static str {
    match self {
      RelationshipCategory::OneToOne => "One-to-One",
      RelationshipCategory::ManyToOne => "Many-to-One",
      RelationshipCategory::OneToMany => "One-to-Many (Coming Soon)",
      RelationshipCategory::ManyToMany => "Many-to-Many (Coming Soon)",
    }
  }

  fn is_implemented(&self) -> bool {
    matches!(self, RelationshipCategory::OneToOne | RelationshipCategory::ManyToOne)
  }
}

/// Represents which form state we're in
#[derive(Debug, Clone, Copy, PartialEq)]
enum FormPhase {
  CategorySelection,
  ChildForm,
}

/// Focused field in category selection
#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedField {
  CategoryList,
  NextButton,
}

/// Main form state for creating entity relationships
pub struct CreateEntityRelationshipForm {
  // Common form state (embedded)
  state: FormState,

  // Current phase
  phase: FormPhase,

  // Category selection
  selected_category: RelationshipCategory,
  category_state: ListState,
  focused_field: FocusedField,

  // Integration with syntaxpresso-core
  cwd: PathBuf,
  entity_file_b64_src: String,
  entity_file_path: PathBuf,

  // Entity files for relationship selection
  entity_files_json: String,

  // Child form (if navigated to)
  child_form: Option<ChildFormType>,
}

impl CreateEntityRelationshipForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    // Fetch entity files for relationships
    let entity_files_json = Self::fetch_entity_files(&cwd).unwrap_or_else(|_| "[]".to_string());

    let mut category_state = ListState::default();
    category_state.select(Some(0));

    Self {
      state: FormState::new(),
      phase: FormPhase::CategorySelection,
      selected_category: RelationshipCategory::OneToOne,
      category_state,
      focused_field: FocusedField::CategoryList,
      cwd,
      entity_file_b64_src,
      entity_file_path,
      entity_files_json,
      child_form: None,
    }
  }

  fn fetch_entity_files(cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let response = get_all_jpa_entities_command::execute(cwd);
    let json = response.to_json_pretty()?;
    Ok(json)
  }

  /// Update selected category from list state
  fn update_selected_category(&mut self) {
    if let Some(idx) = self.category_state.selected() {
      let categories = RelationshipCategory::all();
      if let Some(category) = categories.get(idx) {
        self.selected_category = *category;
      }
    }
  }

  /// Navigate to child form based on selected category
  fn navigate_to_child_form(&mut self) {
    // Check if the selected category is implemented
    if !self.selected_category.is_implemented() {
      self.state.error_message =
        Some(format!("{} is not yet implemented", self.selected_category.as_str()));
      return;
    }

    match self.selected_category {
      RelationshipCategory::OneToOne => {
        let one_to_one_form = CreateOneToOneRelationshipForm::new(
          self.cwd.clone(),
          self.entity_file_b64_src.clone(),
          self.entity_file_path.clone(),
          self.entity_files_json.clone(),
        );
        self.child_form = Some(ChildFormType::OneToOne(Box::new(one_to_one_form)));
        self.phase = FormPhase::ChildForm;

        // Clear any previous error messages
        self.state.error_message = None;
      }
      RelationshipCategory::ManyToOne => {
        let many_to_one_form = CreateManyToOneRelationshipForm::new(
          self.cwd.clone(),
          self.entity_file_b64_src.clone(),
          self.entity_file_path.clone(),
          self.entity_files_json.clone(),
        );
        self.child_form = Some(ChildFormType::ManyToOne(Box::new(many_to_one_form)));
        self.phase = FormPhase::ChildForm;
        self.state.error_message = None;
      }
      _ => {
        self.state.error_message =
          Some(format!("{} is not yet implemented", self.selected_category.as_str()));
      }
    }
  }

  fn handle_category_selection_input(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::CategoryList => match key {
        KeyCode::Char('j') | KeyCode::Down => {
          let len = RelationshipCategory::all().len();
          helpers::navigate_list_static(&KeyCode::Down, &mut self.category_state, len);
          self.update_selected_category();
        }
        KeyCode::Char('k') | KeyCode::Up => {
          let len = RelationshipCategory::all().len();
          helpers::navigate_list_static(&KeyCode::Up, &mut self.category_state, len);
          self.update_selected_category();
        }
        KeyCode::Enter => {
          self.state.input_mode = InputMode::Normal;
        }
        _ => {}
      },
      FocusedField::NextButton => {
        if let KeyCode::Enter = key {
          self.navigate_to_child_form();
        }
      }
    }
  }

  fn render_category_selection(&mut self, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(2), // Title bar
        Constraint::Length(6), // Category selector
        Constraint::Min(0),    // Flexible space for errors
        Constraint::Length(1), // Next button
      ])
      .split(area);

    // Render title bar
    let title = Paragraph::new("New Entity Relationship")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // Render category selector
    let is_focused = self.focused_field == FocusedField::CategoryList;
    let categories = RelationshipCategory::all();
    let items: Vec<ListItem> = categories
      .iter()
      .enumerate()
      .map(|(i, category)| {
        let is_selected = self.category_state.selected() == Some(i);
        let prefix = if is_selected { "●" } else { "○" };
        ListItem::new(format!(" {} {}", prefix, category.as_str()))
      })
      .collect();

    let border_style =
      if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };

    let title = self.generate_title("Relationship Type", is_focused);
    let list = List::new(items)
      .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style))
      .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
      .highlight_symbol(">> ");

    frame.render_stateful_widget(list, chunks[1], &mut self.category_state);

    // Render error message if present
    if let Some(ref error_msg) = self.state.error_message {
      let error_paragraph =
        Paragraph::new(error_msg.as_str()).style(Style::default().fg(Color::Red)).block(
          Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
        );
      frame.render_widget(error_paragraph, chunks[2]);
    }

    // Render next button
    self.render_next_button(frame, chunks[3]);
  }

  fn render_next_button(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
    button_helpers::render_single_button(
      frame,
      area,
      self.focused_field == FocusedField::NextButton,
      self.state.escape_handler.pressed_once,
      button_helpers::ButtonType::Next,
    );
  }

  fn render_child_form(&mut self, frame: &mut Frame) {
    if let Some(ref mut child) = self.child_form {
      match child {
        ChildFormType::OneToOne(form) => form.render(frame),
        ChildFormType::ManyToOne(form) => form.render(frame),
      }
    }
  }
}

// Implement the FormBehavior trait
impl FormBehavior for CreateEntityRelationshipForm {
  fn form_state(&self) -> &FormState {
    &self.state
  }

  fn form_state_mut(&mut self) -> &mut FormState {
    &mut self.state
  }

  fn focus_next(&mut self) {
    if self.phase == FormPhase::CategorySelection {
      self.focused_field = match self.focused_field {
        FocusedField::CategoryList => FocusedField::NextButton,
        FocusedField::NextButton => FocusedField::CategoryList,
      };
    }
  }

  fn focus_prev(&mut self) {
    if self.phase == FormPhase::CategorySelection {
      self.focused_field = match self.focused_field {
        FocusedField::CategoryList => FocusedField::NextButton,
        FocusedField::NextButton => FocusedField::CategoryList,
      };
    }
  }

  fn on_enter_insert_mode(&mut self, _key: KeyCode) {
    // No special behavior needed for category selection
  }

  fn on_enter_pressed(&mut self) {
    match self.phase {
      FormPhase::CategorySelection => {
        if self.focused_field == FocusedField::NextButton {
          self.navigate_to_child_form();
        }
      }
      FormPhase::ChildForm => {
        if let Some(ref mut child) = self.child_form {
          match child {
            ChildFormType::OneToOne(form) => form.on_enter_pressed(),
            ChildFormType::ManyToOne(form) => form.on_enter_pressed(),
          }
        }
      }
    }
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.phase {
      FormPhase::CategorySelection => {
        if self.focused_field == FocusedField::CategoryList {
          self.handle_category_selection_input(key);
        }
      }
      FormPhase::ChildForm => {
        if let Some(ref mut child) = self.child_form {
          match child {
            ChildFormType::OneToOne(form) => form.handle_field_insert(key),
            ChildFormType::ManyToOne(form) => form.handle_field_insert(key),
          }
        }
      }
    }
  }

  fn render(&mut self, frame: &mut Frame) {
    match self.phase {
      FormPhase::CategorySelection => self.render_category_selection(frame),
      FormPhase::ChildForm => self.render_child_form(frame),
    }
  }

  // Override handle_input to support proper escape handling and child forms
  fn handle_input(&mut self, key: KeyCode) -> bool {
    // Handle Ctrl+Enter (F1 signal)
    if key == KeyCode::F(1) {
      match self.phase {
        FormPhase::CategorySelection => {
          self.navigate_to_child_form();
          return false;
        }
        FormPhase::ChildForm => {
          if let Some(ref mut child) = self.child_form {
            return match child {
              ChildFormType::OneToOne(form) => form.handle_input(key),
              ChildFormType::ManyToOne(form) => form.handle_input(key),
            };
          }
        }
      }
    }

    // Handle Ctrl+Backspace (F2 signal)
    if key == KeyCode::F(2)
      && self.phase == FormPhase::ChildForm
      && let Some(ref mut child) = self.child_form
    {
      return match child {
        ChildFormType::OneToOne(form) => form.handle_input(key),
        ChildFormType::ManyToOne(form) => form.handle_input(key),
      };
    }

    // Handle Esc key
    if let KeyCode::Esc = key {
      match self.phase {
        FormPhase::ChildForm => {
          if let Some(ref mut child) = self.child_form {
            return match child {
              ChildFormType::OneToOne(form) => form.handle_input(key),
              ChildFormType::ManyToOne(form) => form.handle_input(key),
            };
          }
        }
        FormPhase::CategorySelection => {
          let mode = self.input_mode();
          let (should_quit, new_mode) = self.escape_handler_mut().handle_escape(mode);
          self.set_input_mode(new_mode);
          if should_quit {
            return true;
          }
          return false;
        }
      }
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

    // Dispatch to appropriate handler based on phase
    if self.phase == FormPhase::ChildForm {
      if let Some(ref mut child) = self.child_form {
        let quit = match child {
          ChildFormType::OneToOne(form) => form.handle_input(key),
          ChildFormType::ManyToOne(form) => form.handle_input(key),
        };

        if quit {
          return true;
        }

        // Check if child wants to go back
        let should_go_back = match child {
          ChildFormType::OneToOne(form) => form.should_go_back(),
          ChildFormType::ManyToOne(form) => form.should_go_back(),
        };

        if should_go_back {
          self.child_form = None;
          self.phase = FormPhase::CategorySelection;
          self.state.error_message = None;
          self.escape_handler_mut().reset();
          return false;
        }

        // Check if child form finished successfully
        let child_should_quit = match child {
          ChildFormType::OneToOne(form) => form.form_state().should_quit,
          ChildFormType::ManyToOne(form) => form.form_state().should_quit,
        };

        if child_should_quit {
          self.state.should_quit = true;
          return true;
        }
      }
    } else {
      match self.input_mode() {
        InputMode::Normal => self.handle_normal_mode(key),
        InputMode::Insert => self.handle_field_insert(key),
      }
    }

    false
  }
}
