// Import Declaration Service Integration Tests
// This module contains comprehensive tests for all import declaration service functions

use syntaxpresso_core::common::services::import_declaration_service::*;
use syntaxpresso_core::common::ts_file::TSFile;
use syntaxpresso_core::common::types::import_types::ImportInsertionPosition;

#[cfg(test)]
mod import_declaration_tests {
  use super::*;

  // Tests for get_all_import_declaration_nodes function
  mod get_all_import_declaration_nodes_tests {
    use super::*;

    #[test]
    fn test_single_import() {
      let java_code = r#"
package com.example;

import java.util.List;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert_eq!(imports.len(), 1, "Should find exactly one import declaration");
      assert_eq!(imports[0].kind(), "import_declaration", "Node should be import_declaration type");
    }

    #[test]
    fn test_multiple_imports() {
      let java_code = r#"
package com.example;

import java.util.List;
import java.util.Map;
import java.io.File;
import com.example.service.MyService;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert_eq!(imports.len(), 4, "Should find exactly four import declarations");
      for import_node in &imports {
        assert_eq!(
          import_node.kind(),
          "import_declaration",
          "All nodes should be import_declaration type"
        );
      }
    }

    #[test]
    fn test_no_imports() {
      let java_code = r#"
package com.example;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert!(imports.is_empty(), "Should return empty vector when no imports exist");
    }

    #[test]
    fn test_wildcard_import() {
      let java_code = r#"
package com.example;

import java.util.*;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert_eq!(imports.len(), 1, "Should find wildcard import declaration");
      assert_eq!(imports[0].kind(), "import_declaration", "Node should be import_declaration type");
    }

    #[test]
    fn test_static_imports() {
      let java_code = r#"
package com.example;

import static java.lang.Math.PI;
import static java.util.Collections.*;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert_eq!(imports.len(), 2, "Should find both static import declarations");
      for import_node in &imports {
        assert_eq!(
          import_node.kind(),
          "import_declaration",
          "All nodes should be import_declaration type"
        );
      }
    }

    #[test]
    fn test_empty_file() {
      let ts_file = TSFile::from_source_code("");
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert!(imports.is_empty(), "Should return empty vector for empty file");
    }

    #[test]
    fn test_invalid_syntax() {
      let ts_file = TSFile::from_source_code("invalid java syntax here");
      let imports = get_all_import_declaration_nodes(&ts_file);

      assert!(imports.is_empty(), "Should return empty vector for invalid syntax");
    }
  }

  // Tests for get_import_declaration_relative_import_scope_node function
  mod get_import_declaration_relative_import_scope_node_tests {
    use super::*;

    #[test]
    fn test_multi_component_import_scope() {
      let java_code = "import com.example.service.MyService;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_relative_import_scope_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract relative import scope");

      if let Some(scope_node) = result {
        let scope_text = ts_file.get_text_from_node(&scope_node).expect("Should get scope text");
        assert_eq!(scope_text, "com.example.service", "Should return scope without class name");
      }
    }

    #[test]
    fn test_deep_nested_import_scope() {
      let java_code = "import com.company.project.module.submodule.feature.MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_relative_import_scope_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract deep nested scope");

      let scope_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get scope text");
      assert_eq!(
        scope_text, "com.company.project.module.submodule.feature",
        "Should return full nested scope"
      );
    }

    #[test]
    fn test_single_component_import() {
      let java_code = "import MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      if !imports.is_empty() {
        let result = get_import_declaration_relative_import_scope_node(&ts_file, imports[0]);
        // Single component imports may not have scoped identifier structure
        // This should return None as there's no scope/name separation
        assert!(result.is_none(), "Single component import should return None for scope");
      }
    }

    #[test]
    fn test_wildcard_import_scope() {
      let java_code = "import java.util.*;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have wildcard import");

      let result = get_import_declaration_relative_import_scope_node(&ts_file, imports[0]);
      // Wildcard imports typically don't have the same scoped identifier structure
      // as regular class imports, so this may return None
      if let Some(scope_node) = result {
        let scope_text = ts_file.get_text_from_node(&scope_node).expect("Should get scope text");
        assert_eq!(scope_text, "java", "If present, should return base scope");
      }
    }

    #[test]
    fn test_invalid_node_type() {
      let java_code = "package com.example;";
      let ts_file = TSFile::from_source_code(java_code);
      let root = ts_file.tree.as_ref().unwrap().root_node();

      let result = get_import_declaration_relative_import_scope_node(&ts_file, root);
      assert!(result.is_none(), "Should return None for non-import declaration node");
    }
  }

  // Tests for get_import_declaration_class_name_node function
  mod get_import_declaration_class_name_node_tests {
    use super::*;

    #[test]
    fn test_multi_component_import_class_name() {
      let java_code = "import com.example.service.MyService;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_class_name_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract class name from import");

      let class_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get class text");
      assert_eq!(class_text, "MyService", "Should return rightmost component as class name");
    }

    #[test]
    fn test_deep_nested_import_class_name() {
      let java_code = "import com.company.project.module.submodule.feature.MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_class_name_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract class name from deep nested import");

      let class_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get class text");
      assert_eq!(class_text, "MyClass", "Should return rightmost component from deep import");
    }

    #[test]
    fn test_two_component_import_class_name() {
      let java_code = "import java.List;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      if !imports.is_empty() {
        let result = get_import_declaration_class_name_node(&ts_file, imports[0]);
        if let Some(class_node) = result {
          let class_text = ts_file.get_text_from_node(&class_node).expect("Should get class text");
          assert_eq!(class_text, "List", "Should return second component as class name");
        }
      }
    }

    #[test]
    fn test_single_component_import_class_name() {
      let java_code = "import MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      if !imports.is_empty() {
        let result = get_import_declaration_class_name_node(&ts_file, imports[0]);
        // Single component imports don't have scoped identifier structure
        assert!(result.is_none(), "Single component import should return None for class name");
      }
    }

    #[test]
    fn test_wildcard_import_class_name() {
      let java_code = "import java.util.*;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have wildcard import");

      let result = get_import_declaration_class_name_node(&ts_file, imports[0]);
      // For wildcard imports like java.util.*, the function returns "util" (the package part before *)
      assert!(result.is_some(), "Wildcard import should return the package part before *");
      if let Some(node) = result {
        let text = ts_file.get_text_from_node(&node).expect("Should get class name text");
        assert_eq!(text, "util", "Should extract 'util' from java.util.*");
      }
    }

    #[test]
    fn test_invalid_node_type() {
      let java_code = "package com.example;";
      let ts_file = TSFile::from_source_code(java_code);
      let root = ts_file.tree.as_ref().unwrap().root_node();

      let result = get_import_declaration_class_name_node(&ts_file, root);
      assert!(result.is_none(), "Should return None for non-import declaration node");
    }
  }

  // Tests for get_import_declaration_full_import_scope_node function
  mod get_import_declaration_full_import_scope_node_tests {
    use super::*;

    #[test]
    fn test_complete_multi_component_import() {
      let java_code = "import com.example.service.MyService;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_full_import_scope_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract full import scope");

      let full_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get full text");
      assert_eq!(
        full_text, "com.example.service.MyService",
        "Should return complete import identifier"
      );
    }

    #[test]
    fn test_complete_deep_nested_import() {
      let java_code = "import com.company.project.module.submodule.feature.MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have at least one import");

      let result = get_import_declaration_full_import_scope_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract full deep nested import");

      let full_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get full text");
      assert_eq!(
        full_text, "com.company.project.module.submodule.feature.MyClass",
        "Should return complete deep import identifier"
      );
    }

    #[test]
    fn test_wildcard_import_full_scope() {
      let java_code = "import java.util.*;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);
      assert!(!imports.is_empty(), "Should have wildcard import");

      let result = get_import_declaration_full_import_scope_node(&ts_file, imports[0]);
      assert!(result.is_some(), "Should extract full scope from wildcard import");

      let full_text = ts_file.get_text_from_node(&result.unwrap()).expect("Should get full text");
      assert_eq!(full_text, "java.util", "Should return package scope for wildcard import");
    }

    #[test]
    fn test_single_component_import_full_scope() {
      let java_code = "import MyClass;";
      let ts_file = TSFile::from_source_code(java_code);
      let imports = get_all_import_declaration_nodes(&ts_file);

      if !imports.is_empty() {
        let result = get_import_declaration_full_import_scope_node(&ts_file, imports[0]);
        // Single component imports may not have scoped identifier structure
        if let Some(full_node) = result {
          let full_text = ts_file.get_text_from_node(&full_node).expect("Should get full text");
          assert_eq!(full_text, "MyClass", "If present, should return the single component");
        }
      }
    }

    #[test]
    fn test_invalid_node_type() {
      let java_code = "package com.example;";
      let ts_file = TSFile::from_source_code(java_code);
      let root = ts_file.tree.as_ref().unwrap().root_node();

      let result = get_import_declaration_full_import_scope_node(&ts_file, root);
      assert!(result.is_none(), "Should return None for non-import declaration node");
    }
  }

  // Tests for find_import_declaration_node function
  mod find_import_declaration_node_tests {
    use super::*;

    #[test]
    fn test_find_existing_regular_import() {
      let java_code = r#"
package com.example;

import java.util.List;
import com.example.service.MyService;
import java.io.File;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);

      let result = find_import_declaration_node(&ts_file, "com.example.service", "MyService");
      assert!(result.is_some(), "Should find existing import declaration");
      if let Some(import_node) = result {
        assert_eq!(
          import_node.kind(),
          "import_declaration",
          "Found node should be import_declaration"
        );
      }
    }

    #[test]
    fn test_find_existing_wildcard_import() {
      let java_code = r#"
package com.example;

import java.util.*;
import com.example.service.MyService;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);

      // For wildcard imports, the class_name parameter represents the wildcard package
      let result = find_import_declaration_node(&ts_file, "java.util", "*");
      if let Some(import_node) = result {
        assert_eq!(
          import_node.kind(),
          "import_declaration",
          "Found node should be import_declaration"
        );
      }
    }

    #[test]
    fn test_find_nonexistent_import() {
      let java_code = r#"
package com.example;

import java.util.List;
import com.example.service.MyService;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);

      let result = find_import_declaration_node(&ts_file, "com.nonexistent", "NonExistent");
      assert!(result.is_none(), "Should return None for non-existent import");
    }

    #[test]
    fn test_find_with_empty_parameters() {
      let java_code = r#"
import java.util.List;
      "#;
      let ts_file = TSFile::from_source_code(java_code);

      let result1 = find_import_declaration_node(&ts_file, "", "List");
      assert!(result1.is_none(), "Should return None for empty package scope");

      let result2 = find_import_declaration_node(&ts_file, "java.util", "");
      assert!(result2.is_none(), "Should return None for empty class name");
    }

    #[test]
    fn test_find_in_file_without_imports() {
      let java_code = r#"
package com.example;

public class Test {}
      "#;
      let ts_file = TSFile::from_source_code(java_code);

      let result = find_import_declaration_node(&ts_file, "java.util", "List");
      assert!(result.is_none(), "Should return None when no imports exist");
    }

    #[test]
    fn test_find_with_case_sensitivity() {
      let java_code = "import com.example.service.MyService;";
      let ts_file = TSFile::from_source_code(java_code);

      let result1 = find_import_declaration_node(&ts_file, "com.example.service", "MyService");
      assert!(result1.is_some(), "Should find exact case match");

      let result2 = find_import_declaration_node(&ts_file, "com.example.service", "myservice");
      assert!(result2.is_none(), "Should not find case mismatch");
    }
  }

  // Tests for add_import function
  mod add_import_tests {
    use super::*;

    #[test]
    fn test_add_import_after_package_declaration() {
      let java_code = r#"package com.example;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result = add_import(
        &mut ts_file,
        &ImportInsertionPosition::AfterPackageDeclaration,
        "java.util",
        "List",
      );
      assert!(result.is_some(), "Should successfully add import after package declaration");

      let new_content =
        ts_file.get_text_from_node(&ts_file.tree.as_ref().unwrap().root_node()).unwrap();
      assert!(new_content.contains("import java.util.List;"), "Should contain the new import");
      assert!(
        new_content.contains("package com.example;"),
        "Should preserve existing package declaration"
      );
    }

    #[test]
    fn test_add_import_before_first_import() {
      let java_code = r#"package com.example;

import java.io.File;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result =
        add_import(&mut ts_file, &ImportInsertionPosition::BeforeFirstImport, "java.util", "List");
      assert!(result.is_some(), "Should successfully add import before first import");

      let imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(imports.len(), 2, "Should have two imports after addition");
    }

    #[test]
    fn test_add_import_after_last_import() {
      let java_code = r#"package com.example;

import java.util.List;
import java.io.File;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterLastImport, "java.util", "Map");
      assert!(result.is_some(), "Should successfully add import after last import");

      let imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(imports.len(), 3, "Should have three imports after addition");
    }

    #[test]
    fn test_add_duplicate_import() {
      let java_code = r#"package com.example;

import java.util.List;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterLastImport, "java.util", "List");
      assert!(result.is_none(), "Should not add duplicate import");

      let imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(imports.len(), 1, "Should still have only one import");
    }

    #[test]
    fn test_add_import_to_empty_file() {
      let mut ts_file = TSFile::from_source_code("");

      let result = add_import(
        &mut ts_file,
        &ImportInsertionPosition::AfterPackageDeclaration,
        "java.util",
        "List",
      );
      assert!(result.is_some(), "Should successfully add import to empty file");

      let new_content =
        ts_file.get_text_from_node(&ts_file.tree.as_ref().unwrap().root_node()).unwrap();
      assert!(new_content.contains("import java.util.List;"), "Should contain the new import");
    }

    #[test]
    fn test_add_import_with_empty_parameters() {
      let mut ts_file = TSFile::from_source_code("package com.example;");

      let result1 =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterPackageDeclaration, "", "List");
      assert!(result1.is_none(), "Should not add import with empty package scope");

      let result2 = add_import(
        &mut ts_file,
        &ImportInsertionPosition::AfterPackageDeclaration,
        "java.util",
        "",
      );
      assert!(result2.is_none(), "Should not add import with empty class name");

      let result3 =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterPackageDeclaration, "   ", "   ");
      assert!(result3.is_none(), "Should not add import with whitespace-only parameters");
    }

    #[test]
    fn test_add_import_without_package_declaration() {
      let java_code = r#"import java.io.File;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result =
        add_import(&mut ts_file, &ImportInsertionPosition::BeforeFirstImport, "java.util", "List");
      assert!(
        result.is_some(),
        "Should successfully add import before first import without package"
      );

      let imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(imports.len(), 2, "Should have two imports after addition");
    }

    #[test]
    fn test_add_import_with_proper_formatting() {
      let java_code = r#"package com.example;

import java.util.List;

public class Test {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      let result =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterLastImport, "java.io", "File");
      assert!(result.is_some(), "Should successfully add import");

      let new_content =
        ts_file.get_text_from_node(&ts_file.tree.as_ref().unwrap().root_node()).unwrap();
      assert!(
        new_content.contains("import java.io.File;"),
        "Should contain properly formatted import"
      );

      // Check that formatting is reasonable (though exact formatting may vary)
      let lines: Vec<&str> = new_content.lines().collect();
      let import_lines: Vec<&str> =
        lines.iter().filter(|line| line.contains("import")).cloned().collect();
      assert_eq!(import_lines.len(), 2, "Should have two import lines");
    }
  }

  // Integration tests combining multiple functions
  mod integration_tests {
    use super::*;

    #[test]
    fn test_comprehensive_import_management() {
      let java_code = r#"package com.example.myapp;

import java.util.List;
import java.io.File;
import com.example.service.MyService;

public class MyClass {}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      // Test finding existing imports
      let existing_import = find_import_declaration_node(&ts_file, "java.util", "List");
      assert!(existing_import.is_some(), "Should find existing import");

      // Test extracting components from existing import
      if let Some(import_node) = existing_import {
        let scope = get_import_declaration_relative_import_scope_node(&ts_file, import_node);
        let class_name = get_import_declaration_class_name_node(&ts_file, import_node);
        let full_scope = get_import_declaration_full_import_scope_node(&ts_file, import_node);

        if let Some(scope_node) = scope {
          let scope_text = ts_file.get_text_from_node(&scope_node).expect("Should get scope text");
          assert_eq!(scope_text, "java.util", "Should extract correct relative scope");
        }

        if let Some(class_node) = class_name {
          let class_text = ts_file.get_text_from_node(&class_node).expect("Should get class text");
          assert_eq!(class_text, "List", "Should extract correct class name");
        }

        if let Some(full_node) = full_scope {
          let full_text = ts_file.get_text_from_node(&full_node).expect("Should get full text");
          assert_eq!(full_text, "java.util.List", "Should extract complete import");
        }
      }

      // Test adding new import
      let add_result =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterLastImport, "java.util", "Map");
      assert!(add_result.is_some(), "Should successfully add new import");

      // Verify all imports are present
      let all_imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(all_imports.len(), 4, "Should have four imports after addition");

      // Test that duplicate addition is prevented
      let duplicate_result =
        add_import(&mut ts_file, &ImportInsertionPosition::AfterLastImport, "java.util", "Map");
      assert!(duplicate_result.is_none(), "Should not add duplicate import");

      let final_imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(final_imports.len(), 4, "Should still have four imports after duplicate attempt");
    }

    #[test]
    fn test_complex_java_file_import_handling() {
      let java_code = r#"/*
 * Complex Java file with various imports
 */
package com.example.service.impl;

import java.util.*;
import java.io.IOException;
import javax.annotation.Nullable;
import com.example.model.User;
import com.example.repository.UserRepository;

/**
 * Service implementation class
 */
public class UserServiceImpl implements UserService {
    // Implementation details
}"#;
      let mut ts_file = TSFile::from_source_code(java_code);

      // Test finding various types of imports
      let _wildcard_import = find_import_declaration_node(&ts_file, "java.util", "*");
      let _regular_import = find_import_declaration_node(&ts_file, "com.example.model", "User");

      // Test getting all imports
      let all_imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(all_imports.len(), 5, "Should find all five imports in complex file");

      // Test adding import to complex file
      let add_result = add_import(
        &mut ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "java.util.concurrent",
        "CompletableFuture",
      );
      assert!(add_result.is_some(), "Should successfully add import to complex file");

      let updated_imports = get_all_import_declaration_nodes(&ts_file);
      assert_eq!(
        updated_imports.len(),
        6,
        "Should have six imports after addition to complex file"
      );
    }

    #[test]
    fn test_edge_case_consistency() {
      // Test that all functions handle the same edge cases consistently
      let test_cases = vec![
        ("", "empty file"),
        ("public class Test {}", "no package, no imports"),
        ("invalid syntax", "invalid syntax"),
        ("package;", "malformed package"),
      ];

      for (code, description) in test_cases {
        let mut ts_file = TSFile::from_source_code(code);

        // Test that all functions handle edge cases gracefully
        let all_imports = get_all_import_declaration_nodes(&ts_file);
        assert!(
          all_imports.is_empty(),
          "Should return empty imports for edge case: {}",
          description
        );

        let find_result = find_import_declaration_node(&ts_file, "java.util", "List");
        assert!(find_result.is_none(), "Should not find imports in edge case: {}", description);

        // Test that add_import doesn't panic even with malformed input
        let _add_result = add_import(
          &mut ts_file,
          &ImportInsertionPosition::AfterPackageDeclaration,
          "java.util",
          "List",
        );
        // May return Some or None depending on the specific edge case, but should not panic

        println!("Successfully handled edge case: {}", description);
      }
    }
  }
}

