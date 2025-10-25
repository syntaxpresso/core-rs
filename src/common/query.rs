#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Query compilation failed: {0}")]
    CompilationError(#[from] tree_sitter::QueryError),

    #[error("No tree available for querying")]
    NoTree,

    #[error("Expected exactly one result, found {found}")]
    NotSingleResult { found: usize },

    #[error("Capture '{name}' not found in query")]
    CaptureNotFound { name: String },
}

#[derive(Debug, Clone)]
pub enum ReturnMode {
    AllNodes,                      // Default: all captured nodes
    SingleCapture(String),         // returning("name")
    AllCaptures,                   // returning_all_captures()
    FilteredCaptures(Vec<String>), // returning_captures(&["name", "params"])
}

#[derive(Debug, Clone, Default)]
pub struct QueryMatch<'a> {
    pub captures: HashMap<String, Node<'a>>,
}

impl<'a> QueryMatch<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, capture_name: &str) -> Option<&Node<'a>> {
        self.captures.get(capture_name)
    }

    pub fn contains(&self, capture_name: &str) -> bool {
        self.captures.contains_key(capture_name)
    }
}

pub struct TSQueryResult<'a> {
    matches: Vec<QueryMatch<'a>>,
    return_mode: ReturnMode,
}

impl<'a> TSQueryResult<'a> {
    pub fn new(matches: Vec<QueryMatch<'a>>, return_mode: ReturnMode) -> Self {
        Self {
            matches,
            return_mode,
        }
    }

    pub fn nodes(&self) -> Vec<Node<'a>> {
        match &self.return_mode {
            ReturnMode::SingleCapture(capture_name) => self
                .matches
                .iter()
                .filter_map(|m| m.get(capture_name))
                .copied()
                .collect(),
            ReturnMode::FilteredCaptures(capture_names) => {
                let mut nodes = Vec::new();
                for match_data in &self.matches {
                    for name in capture_names {
                        if let Some(node) = match_data.get(name) {
                            nodes.push(*node);
                        }
                    }
                }
                nodes.sort_by_key(|n| n.start_byte());
                nodes.dedup_by_key(|n| n.id());
                nodes
            }
            ReturnMode::AllCaptures | ReturnMode::AllNodes => {
                let mut nodes = Vec::new();
                for match_data in &self.matches {
                    for node in match_data.captures.values() {
                        nodes.push(*node);
                    }
                }
                nodes.sort_by_key(|n| n.start_byte());
                nodes.dedup_by_key(|n| n.id());
                nodes
            }
        }
    }

    pub fn first_node(&self) -> Option<Node<'a>> {
        self.nodes().into_iter().next()
    }

    pub fn single_node(&self) -> Result<Node<'a>, QueryError> {
        let nodes = self.nodes();
        match nodes.len() {
            0 => Err(QueryError::NotSingleResult { found: 0 }),
            1 => Ok(nodes[0]),
            n => Err(QueryError::NotSingleResult { found: n }),
        }
    }

    pub fn captures(&self) -> &[QueryMatch<'a>] {
        &self.matches
    }

    pub fn first_capture(&self) -> Option<&QueryMatch<'a>> {
        self.matches.first()
    }

    pub fn single_capture(&self) -> Result<&QueryMatch<'a>, QueryError> {
        match self.matches.len() {
            0 => Err(QueryError::NotSingleResult { found: 0 }),
            1 => Ok(&self.matches[0]),
            n => Err(QueryError::NotSingleResult { found: n }),
        }
    }

    pub fn nodes_from(&self, capture_name: &str) -> Vec<Node<'a>> {
        self.matches
            .iter()
            .filter_map(|m| m.get(capture_name))
            .copied()
            .collect()
    }

    pub fn first_node_from(&self, capture_name: &str) -> Option<Node<'a>> {
        self.nodes_from(capture_name).into_iter().next()
    }

    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn filter<F>(&self, predicate: F) -> TSQueryResult<'a>
    where
        F: Fn(&QueryMatch<'a>) -> bool,
    {
        let filtered_matches: Vec<QueryMatch<'a>> = self
            .matches
            .iter()
            .filter(|m| predicate(m))
            .cloned()
            .collect();

        TSQueryResult::new(filtered_matches, self.return_mode.clone())
    }

    pub fn map<T, F>(&self, mapper: F) -> Vec<T>
    where
        F: Fn(&QueryMatch<'a>) -> T,
    {
        self.matches.iter().map(mapper).collect()
    }
}

pub struct TSQueryBuilder<'a> {
    file: &'a TSFile,
    query_string: String,
    scope_node: Option<Node<'a>>,
    return_mode: ReturnMode,
}

impl<'a> TSQueryBuilder<'a> {
    pub fn new(file: &'a TSFile, query_string: String) -> Self {
        Self {
            file,
            query_string,
            scope_node: None,
            return_mode: ReturnMode::AllNodes,
        }
    }

    pub fn within(mut self, node: Node<'a>) -> Self {
        self.scope_node = Some(node);
        self
    }

    pub fn returning(mut self, capture_name: &str) -> Self {
        self.return_mode = ReturnMode::SingleCapture(capture_name.to_string());
        self
    }

    pub fn returning_all_captures(mut self) -> Self {
        self.return_mode = ReturnMode::AllCaptures;
        self
    }

    pub fn returning_captures(mut self, names: &[&str]) -> Self {
        self.return_mode =
            ReturnMode::FilteredCaptures(names.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn execute(self) -> Result<TSQueryResult<'a>, QueryError> {
        let tree = self.file.tree.as_ref().ok_or(QueryError::NoTree)?;
        let query = Query::new(&self.file.language, &self.query_string)?;
        let mut cursor = QueryCursor::new();
        let root_node = self.scope_node.unwrap_or_else(|| tree.root_node());
        let mut query_matches = cursor.matches(&query, root_node, self.file.source_code.as_bytes());
        let mut matches = Vec::new();
        while let Some(query_match) = query_matches.next() {
            let mut match_data = QueryMatch::new();
            for capture in query_match.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match_data
                    .captures
                    .insert(capture_name.to_string(), capture.node);
            }
            matches.push(match_data);
        }

        Ok(TSQueryResult::new(matches, self.return_mode))
    }

    // Convenience methods that execute immediately
    pub fn nodes(self) -> Result<Vec<Node<'a>>, QueryError> {
        Ok(self.execute()?.nodes())
    }

    pub fn first_node(self) -> Result<Option<Node<'a>>, QueryError> {
        Ok(self.execute()?.first_node())
    }

    pub fn single_node(self) -> Result<Node<'a>, QueryError> {
        self.execute()?.single_node()
    }
}
