#![allow(dead_code)]

use crate::common::query::TSQueryBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{
  InputEdit, Language, Node, Parser, Point, Query, QueryCursor, StreamingIterator, Tree,
};

pub struct TSFile {
  pub language: Language,
  parser: Parser,
  new_path: Option<PathBuf>,
  modified: bool,
  pub file: Option<PathBuf>,
  pub tree: Option<Tree>,
  pub source_code: String,
}

impl TSFile {
  fn set_data(&mut self, source_code: &str) {
    self.tree = self.parser.parse(source_code, None);
    self.source_code = source_code.to_string();
  }

  /// Calculate new position after text replacement
  fn calculate_new_position(&self, start_byte: usize, new_text: &str) -> Point {
    let mut row = 0;
    let mut col = 0;
    // Count to start position
    for (i, ch) in self.source_code.char_indices() {
      if i >= start_byte {
        break;
      }
      if ch == '\n' {
        row += 1;
        col = 0;
      } else {
        col += 1;
      }
    }
    // Add new text position
    for ch in new_text.chars() {
      if ch == '\n' {
        row += 1;
        col = 0;
      } else {
        col += 1;
      }
    }
    Point::new(row, col)
  }

  /// Get node from line/column (1-based)
  fn get_node_from_position(&self, line: usize, column: usize) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let root = tree.root_node();
      let point = Point::new(line.saturating_sub(1), column.saturating_sub(1));
      root.named_descendant_for_point_range(point, point)
    } else {
      None
    }
  }

  /// Get node from line/column with specific kind (more robust)
  fn get_node_from_position_with_kind(
    &self,
    line: usize,
    column: usize,
    expected_kind: &str,
  ) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let root = tree.root_node();
      let point = Point::new(line.saturating_sub(1), column.saturating_sub(1));
      // Try to find the exact node at this position
      if let Some(node) = root.named_descendant_for_point_range(point, point) {
        if node.kind() == expected_kind {
          return Some(node);
        }
        // If not exact match, try parent nodes
        let mut current = Some(node);
        while let Some(node) = current {
          if node.kind() == expected_kind {
            return Some(node);
          }
          current = node.parent();
        }
      }
    }
    None
  }

  /// Replace content at specific positions using incremental parsing
  fn replace_text_incremental_by_pos(
    &mut self,
    start_byte: usize,
    old_end_byte: usize,
    start_position: Point,
    old_end_position: Point,
    new_text: &str,
  ) -> bool {
    let new_end_byte = start_byte + new_text.len();
    // Create edit descriptor for tree-sitter
    let edit = InputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position,
      old_end_position,
      new_end_position: self.calculate_new_position(start_byte, new_text),
    };
    // Tell tree about the edit BEFORE changing source
    if let Some(tree) = &mut self.tree {
      tree.edit(&edit);
      // Apply the text change
      self.source_code.replace_range(start_byte..old_end_byte, new_text);
      // Incremental re-parse (much faster than full reparse!)
      self.tree = self.parser.parse(&self.source_code, Some(tree));
      self.modified = true;
      true
    } else {
      false
    }
  }

  pub fn from_source_code(source_code: &str) -> Self {
    let mut parser = Parser::new();
    let language = tree_sitter_java::LANGUAGE;
    parser.set_language(&language.into()).expect("Error loading Java parser");
    let tree = parser.parse(source_code, None);
    TSFile {
      language: language.into(),
      parser,
      file: None,
      tree,
      source_code: source_code.to_string(),
      new_path: None,
      modified: false,
    }
  }

  pub fn from_file(path: &Path) -> std::io::Result<Self> {
    let source_code = fs::read_to_string(path)?;
    let mut parser = Parser::new();
    let language = tree_sitter_java::LANGUAGE;
    parser.set_language(&language.into()).expect("Error loading Java parser");
    let tree = parser.parse(&source_code, None);
    Ok(TSFile {
      language: language.into(),
      parser,
      file: Some(path.to_path_buf()),
      tree,
      source_code,
      new_path: None,
      modified: false,
    })
  }

  /// Update all source code
  pub fn update_source_code(&mut self, new_source_code: &str) {
    self.set_data(new_source_code);
    self.modified = true;
  }

  /// Update a range in the source code
  pub fn replace_text_by_range(&mut self, start: usize, end: usize, new_text: &str) {
    let mut content = self.source_code.clone();
    content.replace_range(start..end, new_text);
    self.set_data(&content);
    self.modified = true;
  }

  /// Replace node content using incremental parsing (FAST!)
  /// This keeps all other nodes valid by using tree-sitter's incremental updates
  /// Returns the fresh node at the same position after replacement
  pub fn replace_text_by_node(&mut self, node: &Node, new_text: &str) -> Option<Node<'_>> {
    let start_pos = node.start_position();
    let start_line = start_pos.row + 1;
    let start_col = start_pos.column;
    let node_kind = node.kind().to_string();
    let success = self.replace_text_incremental_by_pos(
      node.start_byte(),
      node.end_byte(),
      node.start_position(),
      node.end_position(),
      new_text,
    );
    if success {
      // Try to find the same kind of node at the same position
      self
        .get_node_from_position_with_kind(start_line, start_col, &node_kind)
        .or_else(|| self.get_node_from_position(start_line, start_col))
    } else {
      None
    }
  }

  /// Insert text at a position
  pub fn insert_text(&mut self, position: usize, text: &str) {
    let mut content = self.source_code.clone();
    content.insert_str(position, text);
    self.set_data(&content);
    self.modified = true;
  }

  /// Save to original file path
  pub fn save(&mut self) -> std::io::Result<()> {
    let file = self
      .file
      .as_ref()
      .ok_or_else(|| std::io::Error::other("File path is not set. Use save_as(path) instead."))?;
    if let Some(new_path) = &self.new_path {
      fs::rename(file, new_path)?;
      self.file = Some(new_path.clone());
      self.new_path = None;
    }
    fs::write(self.file.as_ref().unwrap(), &self.source_code)?;
    self.modified = false;
    Ok(())
  }

  /// Save to a new file path
  pub fn save_as(&mut self, path: &Path) -> std::io::Result<()> {
    fs::write(path, &self.source_code)?;
    self.file = Some(path.to_path_buf());
    self.modified = false;
    Ok(())
  }

  /// Move file to new destination
  pub fn move_file(&mut self, destination: &Path) {
    self.new_path = Some(destination.to_path_buf());
    self.modified = true;
  }

  /// Rename file in current directory
  pub fn rename(&mut self, new_name: &str) -> std::io::Result<()> {
    let file = self
      .file
      .as_ref()
      .ok_or_else(|| std::io::Error::other("Cannot rename a file that has not been saved yet."))?;
    let parent_dir =
      file.parent().ok_or_else(|| std::io::Error::other("Unable to get parent directory"))?;
    let target_path = parent_dir.join(new_name);
    self.new_path = Some(target_path);
    self.modified = true;
    Ok(())
  }

  /// Get text from byte range
  pub fn get_text_from_range(&self, start: usize, end: usize) -> Option<&str> {
    self.source_code.get(start..end)
  }

  /// Get text from node
  pub fn get_text_from_node(&self, node: &Node) -> Option<&str> {
    self.get_text_from_range(node.start_byte(), node.end_byte())
  }

  pub fn get_file_name_without_ext(&self) -> Option<String> {
    self.file.as_ref()?.file_stem()?.to_str().map(|s| s.to_string())
  }

  /// Is file modified
  pub fn is_modified(&self) -> bool {
    self.modified
  }

  /// Has unsaved changes
  pub fn has_unsaved_changes(&self) -> bool {
    self.modified
  }

  /// Get the file path (if set)
  pub fn file_path(&self) -> Option<&PathBuf> {
    self.file.as_ref()
  }

  /// Create a fluent query builder for advanced queries
  /// This is the new fluent API entry point
  pub fn query_builder(&self, query_string: &str) -> TSQueryBuilder<'_> {
    TSQueryBuilder::new(self, query_string.to_string())
  }

  /// Execute a tree-sitter query on the parsed tree
  /// Returns a vector of nodes from all captures in all matches
  pub fn query(&self, query_string: &str) -> Result<Vec<Node<'_>>, Box<dyn std::error::Error>> {
    let tree = self.tree.as_ref().ok_or("No parsed tree available")?;
    let query = Query::new(&self.language, query_string)?;
    let mut cursor = QueryCursor::new();
    let root_node = tree.root_node();
    let mut nodes = Vec::new();
    let mut query_matches = cursor.matches(&query, root_node, self.source_code.as_bytes());
    while let Some(query_match) = query_matches.next() {
      for capture in query_match.captures {
        nodes.push(capture.node);
      }
    }
    Ok(nodes)
  }

  /// Convert byte position to line/column (1-based)
  fn byte_position_to_point(&self, byte_position: usize) -> Point {
    let mut row = 0;
    let mut col = 0;
    for (i, ch) in self.source_code.char_indices() {
      if i >= byte_position {
        break;
      }
      if ch == '\n' {
        row += 1;
        col = 0;
      } else {
        col += 1;
      }
    }
    Point::new(row, col)
  }

  /// Find a node by byte position
  pub fn get_node_at_byte_position(&self, byte_position: usize) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let root = tree.root_node();
      let point = self.byte_position_to_point(byte_position);
      root.descendant_for_point_range(point, point)
    } else {
      None
    }
  }

  /// Find a named node by byte position
  pub fn get_named_node_at_byte_position(&self, byte_position: usize) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let root = tree.root_node();
      let point = self.byte_position_to_point(byte_position);
      root.named_descendant_for_point_range(point, point)
    } else {
      None
    }
  }

  /// Find a node by byte position with specific kind
  pub fn get_node_at_byte_position_with_kind(
    &self,
    byte_position: usize,
    expected_kind: &str,
  ) -> Option<Node<'_>> {
    if let Some(tree) = &self.tree {
      let root = tree.root_node();
      let point = self.byte_position_to_point(byte_position);
      // Try to find the exact node at this position
      if let Some(node) = root.descendant_for_point_range(point, point) {
        if node.kind() == expected_kind {
          return Some(node);
        }
        // If not exact match, try parent nodes
        let mut current = Some(node);
        while let Some(node) = current {
          if node.kind() == expected_kind {
            return Some(node);
          }
          current = node.parent();
        }
      }
    }
    None
  }

  /// Replace text by byte range using incremental parsing
  /// Returns the fresh node at the same position after replacement
  pub fn replace_text_by_byte_range(
    &mut self,
    start_byte: usize,
    end_byte: usize,
    new_text: &str,
  ) -> Option<Node<'_>> {
    // We need to get node info before doing the replacement since we need immutable access
    let (start_pos, end_pos, node_kind) = {
      if let Some(node) = self.get_named_node_at_byte_position(start_byte) {
        (node.start_position(), node.end_position(), node.kind().to_string())
      } else {
        return None;
      }
    };
    let start_line = start_pos.row + 1;
    let start_col = start_pos.column;
    let success =
      self.replace_text_incremental_by_pos(start_byte, end_byte, start_pos, end_pos, new_text);
    if success {
      // Try to find the same kind of node at the same position
      self
        .get_node_from_position_with_kind(start_line, start_col, &node_kind)
        .or_else(|| self.get_node_from_position(start_line, start_col))
    } else {
      None
    }
  }
}
