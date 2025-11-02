#[cfg(test)]
mod annotation_service_tests {
  use syntaxpresso_core::common::services::annotation_service::*;
  use syntaxpresso_core::common::ts_file::TSFile;
  use syntaxpresso_core::common::types::annotation_types::AnnotationInsertionPosition;

  fn create_ts_file(content: &str) -> TSFile {
    TSFile::from_source_code(content)
  }

  const SIMPLE_JAVA_CLASS: &str = "@Entity\npublic class User {\n    @Id\n    private Long id;\n}";

  #[test]
  fn test_get_all_annotation_nodes_basic() {
    let ts_file = create_ts_file(SIMPLE_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let annotations = get_all_annotation_nodes(&ts_file, root);
      assert!(annotations.len() >= 2, "Should find @Entity and @Id annotations");
    }
  }

  #[test]
  fn test_find_annotation_node_by_name_basic() {
    let ts_file = create_ts_file(SIMPLE_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let entity_annotation = find_annotation_node_by_name(&ts_file, root, "Entity");
      assert!(entity_annotation.is_some(), "Should find @Entity annotation");

      let id_annotation = find_annotation_node_by_name(&ts_file, root, "Id");
      assert!(id_annotation.is_some(), "Should find @Id annotation");
    }
  }

  #[test]
  fn test_find_annotation_node_by_name_not_found() {
    let ts_file = create_ts_file(SIMPLE_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let result = find_annotation_node_by_name(&ts_file, root, "NonExistent");
      assert!(result.is_none(), "Should not find non-existent annotation");
    }
  }

  #[test]
  fn test_get_annotation_name_node_basic() {
    let ts_file = create_ts_file(SIMPLE_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let annotations = get_all_annotation_nodes(&ts_file, root);

      for annotation in annotations {
        if let Some(name_node) = get_annotation_name_node(&ts_file, annotation)
          && let Some(name_text) = ts_file.get_text_from_node(&name_node)
        {
          assert!(!name_text.is_empty(), "Annotation name should not be empty");
          assert!(
            name_text.chars().all(|c| c.is_alphanumeric() || c == '_'),
            "Name should be valid identifier"
          );
        }
      }
    }
  }

  #[test]
  fn test_add_annotation_basic() {
    let mut ts_file = create_ts_file("public class User {}");

    let result = add_annotation(
      &mut ts_file,
      0,
      &AnnotationInsertionPosition::BeforeFirstAnnotation,
      "@Entity",
    );

    assert!(result.is_some(), "Should successfully add annotation");
    assert!(ts_file.source_code.contains("@Entity"), "Source should contain added annotation");
  }

  #[test]
  fn test_add_annotation_argument_basic() {
    let mut ts_file = create_ts_file("@Entity\npublic class User {}");

    let annotation_pos = ts_file.source_code.find("@Entity").unwrap_or(0);

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "name", "\"users\"");

    assert!(result.is_some(), "Should successfully add argument to annotation");
    assert!(ts_file.source_code.contains("name = \"users\""), "Should contain new argument");
  }

  #[test]
  fn test_add_annotation_single_value_basic() {
    let mut ts_file = create_ts_file("@Test\npublic void testMethod() {}");

    let annotation_pos = ts_file.source_code.find("@Test").unwrap_or(0);

    let result = add_annotation_single_value(&mut ts_file, annotation_pos, "\"test value\"");

    assert!(result.is_some(), "Should successfully add single value to annotation");
    assert!(ts_file.source_code.contains("test value"), "Should contain single value");
  }

  // Tests with more complex Java code
  const COMPLEX_JAVA_CLASS: &str = r#"
@Entity
@Table(name = "users", schema = "public")
@JsonIgnoreProperties(ignoreUnknown = true)
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    @Column(name = "id", nullable = false)
    private Long id;

    @Column(name = "username", nullable = false, unique = true)
    @NotNull
    @Size(min = 3, max = 50)
    private String username;

    @OneToMany(mappedBy = "user", cascade = CascadeType.ALL, fetch = FetchType.LAZY)
    private List<Order> orders;
}
"#;

  #[test]
  fn test_get_all_annotation_nodes_complex() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let annotations = get_all_annotation_nodes(&ts_file, root);

      // Should find all annotations: @Entity, @Table, @JsonIgnoreProperties, @Id, @GeneratedValue, @Column (x2), @NotNull, @Size, @OneToMany
      assert!(
        annotations.len() >= 10,
        "Should find multiple annotations, found: {}",
        annotations.len()
      );

      let annotation_texts: Vec<String> = annotations
        .iter()
        .filter_map(|node| ts_file.get_text_from_node(node))
        .map(|text| text.to_string())
        .collect();

      // Verify specific annotations exist
      assert!(annotation_texts.iter().any(|text| text.contains("@Entity")));
      assert!(annotation_texts.iter().any(|text| text.contains("@Table")));
      assert!(annotation_texts.iter().any(|text| text.contains("@JsonIgnoreProperties")));
      assert!(annotation_texts.iter().any(|text| text.contains("@Column")));
      assert!(annotation_texts.iter().any(|text| text.contains("@OneToMany")));
    }
  }

  #[test]
  fn test_get_annotation_argument_pair_nodes_complex() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let pairs = get_annotation_argument_pair_nodes(&ts_file, root);

      assert!(pairs.len() > 5, "Should find multiple argument pairs");

      // Check that we can get text from the pairs and they contain '='
      for pair in pairs {
        if let Some(pair_text) = ts_file.get_text_from_node(&pair) {
          assert!(pair_text.contains("="), "Argument pair should contain '=': {}", pair_text);
        }
      }
    }
  }

  #[test]
  fn test_get_annotation_argument_key_nodes_complex() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let keys = get_annotation_argument_key_nodes(&ts_file, root);

      assert!(!keys.is_empty(), "Should find annotation argument keys");

      let key_texts: Vec<String> = keys
        .iter()
        .filter_map(|node| ts_file.get_text_from_node(node))
        .map(|text| text.to_string())
        .collect();

      // Should find common keys like "name", "nullable", "strategy", etc.
      assert!(key_texts.iter().any(|text| text == "name"), "Should find 'name' key");
      assert!(
        key_texts.iter().any(|text| text == "nullable" || text == "strategy" || text == "min"),
        "Should find common annotation keys"
      );
    }
  }

  #[test]
  fn test_get_annotation_argument_value_nodes_complex() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let values = get_annotation_argument_value_nodes(&ts_file, root);

      assert!(!values.is_empty(), "Should find annotation argument values");

      let value_texts: Vec<String> = values
        .iter()
        .filter_map(|node| ts_file.get_text_from_node(node))
        .map(|text| text.to_string())
        .collect();

      // Should find string values, boolean values, enum values, etc.
      assert!(
        value_texts.iter().any(|text| text.contains("\"users\"") || text.contains("\"username\"")),
        "Should find string values"
      );
      assert!(
        value_texts
          .iter()
          .any(|text| text == "false" || text == "true" || text.contains("GenerationType")),
        "Should find boolean or enum values"
      );
    }
  }

  #[test]
  fn test_find_annotation_value_node_by_key_complex() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      // Look for "name" key values
      let name_value = find_annotation_value_node_by_key(&ts_file, root, "name");
      assert!(name_value.is_some(), "Should find value for 'name' key");

      if let Some(value_node) = name_value
        && let Some(value_text) = ts_file.get_text_from_node(&value_node)
      {
        assert!(value_text.contains("\""), "Name value should be a quoted string");
      }

      // Look for "strategy" key value
      let strategy_value = find_annotation_value_node_by_key(&ts_file, root, "strategy");
      assert!(strategy_value.is_some(), "Should find value for 'strategy' key");
    }
  }

  #[test]
  fn test_find_annotation_node_by_name_case_sensitive() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      // Test exact case match
      let entity_annotation = find_annotation_node_by_name(&ts_file, root, "Entity");
      assert!(entity_annotation.is_some(), "Should find @Entity with exact case");

      // Test case sensitivity
      let result = find_annotation_node_by_name(&ts_file, root, "entity");
      assert!(result.is_none(), "Should not find annotation with wrong case");

      let result = find_annotation_node_by_name(&ts_file, root, "ENTITY");
      assert!(result.is_none(), "Should not find annotation with wrong case");
    }
  }

  #[test]
  fn test_find_annotation_node_by_name_empty_and_invalid() {
    let ts_file = create_ts_file(COMPLEX_JAVA_CLASS);
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let result = find_annotation_node_by_name(&ts_file, root, "");
      assert!(result.is_none(), "Should return None for empty annotation name");

      let result = find_annotation_node_by_name(&ts_file, root, "   ");
      assert!(result.is_none(), "Should return None for whitespace-only annotation name");

      let result = find_annotation_node_by_name(&ts_file, root, "NonExistentAnnotation");
      assert!(result.is_none(), "Should return None for non-existent annotation");
    }
  }

  #[test]
  fn test_get_all_annotation_nodes_empty_class() {
    let ts_file = create_ts_file("public class EmptyClass {}");
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let annotations = get_all_annotation_nodes(&ts_file, root);
      assert_eq!(annotations.len(), 0, "Should find no annotations in empty class");
    }
  }

  #[test]
  fn test_get_all_annotation_nodes_no_tree() {
    let mut ts_file_no_tree = TSFile::from_source_code("");
    ts_file_no_tree.tree = None;

    let ts_file = create_ts_file("public class Test {}");
    if let Some(root) = ts_file.tree.as_ref().map(|tree| tree.root_node()) {
      let annotations = get_all_annotation_nodes(&ts_file_no_tree, root);
      assert_eq!(annotations.len(), 0, "Should return empty vec when tree is None");
    }
  }

  #[test]
  fn test_annotation_functions_with_no_tree() {
    let mut ts_file = create_ts_file("@Test public void test() {}");
    ts_file.tree = None;

    let ts_file_with_tree = create_ts_file("@Test public void test() {}");
    if let Some(root) = ts_file_with_tree.tree.as_ref().map(|tree| tree.root_node()) {
      let result = get_annotation_name_node(&ts_file, root);
      assert!(result.is_none(), "Should return None when tree is None");

      let result = find_annotation_node_by_name(&ts_file, root, "Test");
      assert!(result.is_none(), "Should return None when tree is None");

      let pairs = get_annotation_argument_pair_nodes(&ts_file, root);
      assert_eq!(pairs.len(), 0, "Should return empty vec when tree is None");

      let keys = get_annotation_argument_key_nodes(&ts_file, root);
      assert_eq!(keys.len(), 0, "Should return empty vec when tree is None");

      let values = get_annotation_argument_value_nodes(&ts_file, root);
      assert_eq!(values.len(), 0, "Should return empty vec when tree is None");

      let result = find_annotation_value_node_by_key(&ts_file, root, "key");
      assert!(result.is_none(), "Should return None when tree is None");
    }
  }

  // Tests for mutable functions with edge cases
  #[test]
  fn test_add_annotation_with_empty_text() {
    let mut ts_file = create_ts_file("public class User {}");

    let result =
      add_annotation(&mut ts_file, 0, &AnnotationInsertionPosition::BeforeFirstAnnotation, "");
    assert!(result.is_none(), "Should return None for empty annotation text");

    let result =
      add_annotation(&mut ts_file, 0, &AnnotationInsertionPosition::BeforeFirstAnnotation, "   ");
    assert!(result.is_none(), "Should return None for whitespace-only annotation text");
  }

  #[test]
  fn test_add_annotation_argument_with_empty_values() {
    let mut ts_file = create_ts_file("@Entity\npublic class User {}");
    let annotation_pos = ts_file.source_code.find("@Entity").unwrap_or(0);

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "", "value");
    assert!(result.is_none(), "Should return None for empty key");

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "key", "");
    assert!(result.is_none(), "Should return None for empty value");

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "   ", "value");
    assert!(result.is_none(), "Should return None for whitespace-only key");

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "key", "   ");
    assert!(result.is_none(), "Should return None for whitespace-only value");
  }

  #[test]
  fn test_add_annotation_single_value_with_empty_value() {
    let mut ts_file = create_ts_file("@Test\npublic void testMethod() {}");
    let annotation_pos = ts_file.source_code.find("@Test").unwrap_or(0);

    let result = add_annotation_single_value(&mut ts_file, annotation_pos, "");
    assert!(result.is_none(), "Should return None for empty value");

    let result = add_annotation_single_value(&mut ts_file, annotation_pos, "   ");
    assert!(result.is_none(), "Should return None for whitespace-only value");
  }

  #[test]
  fn test_add_annotation_to_existing_annotations() {
    let mut ts_file =
      create_ts_file("@Entity\npublic class User {\n    @Id\n    private Long id;\n}");

    // Add annotation before existing @Entity
    let class_pos = ts_file.source_code.find("@Entity").unwrap_or(0);

    let result = add_annotation(
      &mut ts_file,
      class_pos,
      &AnnotationInsertionPosition::BeforeFirstAnnotation,
      "@Table(name = \"users\")",
    );

    assert!(result.is_some(), "Should successfully add annotation before existing ones");
    assert!(ts_file.source_code.contains("@Table"), "Should contain new annotation");
    assert!(ts_file.source_code.contains("@Entity"), "Should preserve existing annotation");
  }

  #[test]
  fn test_add_annotation_argument_to_existing_arguments() {
    let mut ts_file = create_ts_file("@Column(name = \"username\")\nprivate String username;");
    let annotation_pos = ts_file.source_code.find("@Column").unwrap_or(0);

    let result = add_annotation_argument(&mut ts_file, annotation_pos, "nullable", "false");

    assert!(result.is_some(), "Should successfully add argument to existing annotation");
    assert!(ts_file.source_code.contains("nullable = false"), "Should contain new argument");
    assert!(
      ts_file.source_code.contains("name = \"username\""),
      "Should preserve existing argument"
    );
  }
}
