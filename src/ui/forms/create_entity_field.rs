use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::PathBuf;

use crate::ui::form_trait::{FormBehavior, FormState, InputMode, button_helpers, helpers};
use crate::ui::forms::create_basic_field::CreateBasicFieldForm;
use crate::ui::forms::create_enum_field::CreateEnumFieldForm;
use crate::ui::forms::create_id_field::CreateIdFieldForm;

/// Enum to hold different child form types
enum ChildFormType {
  Basic(Box<CreateBasicFieldForm>),
  Enum(Box<CreateEnumFieldForm>),
  Id(Box<CreateIdFieldForm>),
}

/// Field category options
#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldCategory {
  Basic,
  Enum,
  Id,
}

impl FieldCategory {
  fn all() -> Vec<FieldCategory> {
    vec![FieldCategory::Basic, FieldCategory::Enum, FieldCategory::Id]
  }

  fn as_str(&self) -> &'static str {
    match self {
      FieldCategory::Basic => "Basic Field",
      FieldCategory::Enum => "Enum Field",
      FieldCategory::Id => "ID Field",
    }
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

/// Main form state for creating entity fields
pub struct CreateEntityFieldForm {
  // Common form state (embedded)
  state: FormState,

  // Current phase
  phase: FormPhase,

  // Category selection
  selected_category: FieldCategory,
  category_state: ListState,
  focused_field: FocusedField,

  // Integration with syntaxpresso-core
  cwd: PathBuf,
  entity_file_b64_src: String,
  entity_file_path: PathBuf,

  // Child form (if navigated to)
  child_form: Option<ChildFormType>,
}

impl CreateEntityFieldForm {
  pub fn new(cwd: PathBuf, entity_file_b64_src: String, entity_file_path: PathBuf) -> Self {
    let mut category_state = ListState::default();
    category_state.select(Some(0));

    Self {
      state: FormState::new(),
      phase: FormPhase::CategorySelection,
      selected_category: FieldCategory::Basic,
      category_state,
      focused_field: FocusedField::CategoryList,
      cwd,
      entity_file_b64_src,
      entity_file_path,
      child_form: None,
    }
  }

  /// Update selected category from list state
  fn update_selected_category(&mut self) {
    if let Some(idx) = self.category_state.selected() {
      let categories = FieldCategory::all();
      if let Some(category) = categories.get(idx) {
        self.selected_category = *category;
      }
    }
  }

  /// Navigate to child form based on selected category
  fn navigate_to_child_form(&mut self) {
    match self.selected_category {
      FieldCategory::Basic => {
        // Create basic field form
        let basic_form = CreateBasicFieldForm::new(
          self.cwd.clone(),
          self.entity_file_b64_src.clone(),
          self.entity_file_path.clone(),
        );
        self.child_form = Some(ChildFormType::Basic(Box::new(basic_form)));
        self.phase = FormPhase::ChildForm;
      }
      FieldCategory::Enum => {
        // Create enum field form
        let enum_form = CreateEnumFieldForm::new(
          self.cwd.clone(),
          self.entity_file_b64_src.clone(),
          self.entity_file_path.clone(),
        );
        self.child_form = Some(ChildFormType::Enum(Box::new(enum_form)));
        self.phase = FormPhase::ChildForm;
      }
      FieldCategory::Id => {
        // Create ID field form
        let id_form = CreateIdFieldForm::new(
          self.cwd.clone(),
          self.entity_file_b64_src.clone(),
          self.entity_file_path.clone(),
        );
        self.child_form = Some(ChildFormType::Id(Box::new(id_form)));
        self.phase = FormPhase::ChildForm;
      }
    }
  }

  fn handle_category_selection_input(&mut self, key: KeyCode) {
    match self.focused_field {
      FocusedField::CategoryList => match key {
        KeyCode::Char('j') | KeyCode::Down => {
          let len = FieldCategory::all().len();
          helpers::navigate_list_static(&KeyCode::Down, &mut self.category_state, len);
          self.update_selected_category();
        }
        KeyCode::Char('k') | KeyCode::Up => {
          let len = FieldCategory::all().len();
          helpers::navigate_list_static(&KeyCode::Up, &mut self.category_state, len);
          self.update_selected_category();
        }
        KeyCode::Enter => {
          self.navigate_to_child_form();
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
        Constraint::Length(5), // Category selector (3 items + 2 borders + padding)
        Constraint::Min(0),    // Flexible space for errors
        Constraint::Length(1), // Next button
      ])
      .split(area);

    // Render title bar
    let title = Paragraph::new("New Entity Field")
      .alignment(Alignment::Center)
      .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
      .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // Render category selector
    let is_focused = self.focused_field == FocusedField::CategoryList;
    let categories = FieldCategory::all();
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

    let title = if is_focused && self.state.input_mode == InputMode::Insert {
      "Category -- INSERT --"
    } else if is_focused {
      "Category -- NORMAL --"
    } else {
      "Category"
    };

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
        ChildFormType::Basic(form) => form.render(frame),
        ChildFormType::Enum(form) => form.render(frame),
        ChildFormType::Id(form) => form.render(frame),
      }
    }
  }
}

// Implement the FormBehavior trait
impl FormBehavior for CreateEntityFieldForm {
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
    // Child forms handle their own focus
  }

  fn focus_prev(&mut self) {
    if self.phase == FormPhase::CategorySelection {
      self.focused_field = match self.focused_field {
        FocusedField::CategoryList => FocusedField::NextButton,
        FocusedField::NextButton => FocusedField::CategoryList,
      };
    }
    // Child forms handle their own focus
  }

  fn on_enter_insert_mode(&mut self, _key: KeyCode) {
    // Category selection doesn't use insert mode
    // Child forms handle their own insert mode
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
            ChildFormType::Basic(form) => form.on_enter_pressed(),
            ChildFormType::Enum(form) => form.on_enter_pressed(),
            ChildFormType::Id(form) => form.on_enter_pressed(),
          }
        }
      }
    }
  }

  fn handle_field_insert(&mut self, key: KeyCode) {
    match self.phase {
      FormPhase::CategorySelection => {
        // Only category list can be in insert mode
        if self.focused_field == FocusedField::CategoryList {
          self.handle_category_selection_input(key);
        }
      }
      FormPhase::ChildForm => {
        if let Some(ref mut child) = self.child_form {
          match child {
            ChildFormType::Basic(form) => form.handle_field_insert(key),
            ChildFormType::Enum(form) => form.handle_field_insert(key),
            ChildFormType::Id(form) => form.handle_field_insert(key),
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

  // Override handle_input to support proper escape handling
  fn handle_input(&mut self, key: KeyCode) -> bool {
    // Handle Ctrl+Enter (F1 signal) - navigate to child form from category selection
    if key == KeyCode::F(1) {
      match self.phase {
        FormPhase::CategorySelection => {
          self.navigate_to_child_form();
          return false;
        }
        FormPhase::ChildForm => {
          // Delegate to child
          if let Some(ref mut child) = self.child_form {
            let result = match child {
              ChildFormType::Basic(form) => form.handle_input(key),
              ChildFormType::Enum(form) => form.handle_input(key),
              ChildFormType::Id(form) => form.handle_input(key),
            };
            return result;
          }
        }
      }
    }

    // Handle Ctrl+Backspace (F2 signal) - not applicable for category selection (no back), delegate to child
    if key == KeyCode::F(2)
      && self.phase == FormPhase::ChildForm
      && let Some(ref mut child) = self.child_form
    {
      let result = match child {
        ChildFormType::Basic(form) => form.handle_input(key),
        ChildFormType::Enum(form) => form.handle_input(key),
        ChildFormType::Id(form) => form.handle_input(key),
      };
      return result;
    }

    // Handle Esc key
    if let KeyCode::Esc = key {
      match self.phase {
        FormPhase::ChildForm => {
          // Delegate to child form for proper escape handling
          if let Some(ref mut child) = self.child_form {
            let result = match child {
              ChildFormType::Basic(form) => form.handle_input(key),
              ChildFormType::Enum(form) => form.handle_input(key),
              ChildFormType::Id(form) => form.handle_input(key),
            };
            return result;
          }
        }
        FormPhase::CategorySelection => {
          // Normal Esc handling for category selection
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
        // Handle input first
        let quit = match child {
          ChildFormType::Basic(form) => form.handle_input(key),
          ChildFormType::Enum(form) => form.handle_input(key),
          ChildFormType::Id(form) => form.handle_input(key),
        };

        if quit {
          // Child signaled quit, propagate it
          return true;
        }

        // Check if child wants to go back (after handling input)
        let should_go_back = match child {
          ChildFormType::Basic(form) => form.should_go_back(),
          ChildFormType::Enum(form) => form.should_go_back(),
          ChildFormType::Id(form) => form.should_go_back(),
        };

        if should_go_back {
          // Go back to category selection
          self.child_form = None;
          self.phase = FormPhase::CategorySelection;
          self.state.error_message = None;
          self.escape_handler_mut().reset();
          return false;
        }

        // Check if child form finished successfully
        let child_should_quit = match child {
          ChildFormType::Basic(form) => form.form_state().should_quit,
          ChildFormType::Enum(form) => form.form_state().should_quit,
          ChildFormType::Id(form) => form.form_state().should_quit,
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

    false // Don't quit
  }
}
