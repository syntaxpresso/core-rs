// Package Declaration Service Integration Tests
// This module contains comprehensive tests for all package declaration service functions

use syntaxpresso_core::common::services::package_declaration_service::*;
use syntaxpresso_core::common::ts_file::TSFile;

#[cfg(test)]
mod package_declaration_tests {
    use super::*;

    // Tests for get_package_declaration_node function
    mod get_package_declaration_node_tests {
        use super::*;

        #[test]
        fn test_valid_multi_component_package() {
            let ts_file = TSFile::from_source_code("package com.example.myapp;\n\npublic class Test {}");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_some(), "Should find package declaration node");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert!(text.contains("package com.example.myapp"), "Should contain full package declaration");
        }

        #[test]
        fn test_valid_single_component_package() {
            let ts_file = TSFile::from_source_code("package myapp;\n\npublic class Test {}");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_some(), "Should find single component package declaration");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert!(text.contains("package myapp"), "Should contain single component package");
        }

        #[test]
        fn test_no_package_declaration() {
            let ts_file = TSFile::from_source_code("public class Test {}");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_none(), "Should return None when no package declaration exists");
        }

        #[test]
        fn test_empty_file() {
            let ts_file = TSFile::from_source_code("");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_none(), "Should return None for empty file");
        }

        #[test]
        fn test_package_with_whitespace() {
            let ts_file = TSFile::from_source_code("   package    com.example.myapp   ;\n\npublic class Test {}");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_some(), "Should handle whitespace around package declaration");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert!(text.contains("com.example.myapp"), "Should extract package name despite whitespace");
        }

        #[test]
        fn test_package_with_comments() {
            let ts_file = TSFile::from_source_code(r#"
                /* Package declaration */
                package com.example.myapp;
                
                // Some comment
                public class Test {}
            "#);
            
            let result = get_package_declaration_node(&ts_file);
            assert!(result.is_some(), "Should handle comments around package declaration");
        }

        #[test]
        fn test_deep_nested_package() {
            let ts_file = TSFile::from_source_code("package com.company.project.module.submodule.feature;");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_some(), "Should handle deeply nested package declarations");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert!(text.contains("com.company.project.module.submodule.feature"), "Should contain full deep package name");
        }

        #[test]
        fn test_invalid_java_syntax() {
            let ts_file = TSFile::from_source_code("invalid java syntax here");
            let result = get_package_declaration_node(&ts_file);
            
            assert!(result.is_none(), "Should return None for invalid Java syntax");
        }
    }

    // Tests for get_package_class_name_node function
    mod get_package_class_name_node_tests {
        use super::*;

        #[test]
        fn test_multi_component_package_class_name() {
            let ts_file = TSFile::from_source_code("package com.example.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_name_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract class name from multi-component package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "myapp", "Should return rightmost component as class name");
        }

        #[test]
        fn test_deep_nested_package_class_name() {
            let ts_file = TSFile::from_source_code("package com.company.project.module.submodule.feature;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_name_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract class name from deep nested package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "feature", "Should return rightmost component from deep package");
        }

        #[test]
        fn test_single_component_package_class_name() {
            let ts_file = TSFile::from_source_code("package myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_name_node(&ts_file, package_node);
            
            // Single component packages don't have scoped identifier structure
            // This should return None as there's no scope/name separation
            assert!(result.is_none(), "Single component package should return None for class name");
        }

        #[test]
        fn test_two_component_package_class_name() {
            let ts_file = TSFile::from_source_code("package com.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_name_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract class name from two-component package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "myapp", "Should return second component as class name");
        }

        #[test]
        fn test_class_name_with_whitespace() {
            let ts_file = TSFile::from_source_code("package   com.example.myapp   ;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_name_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should handle whitespace in package declaration");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "myapp", "Should extract clean class name despite whitespace");
        }
    }

    // Tests for get_package_class_scope_node function
    mod get_package_class_scope_node_tests {
        use super::*;

        #[test]
        fn test_multi_component_package_scope() {
            let ts_file = TSFile::from_source_code("package com.example.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract scope from multi-component package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.example", "Should return scope part without rightmost component");
        }

        #[test]
        fn test_deep_nested_package_scope() {
            let ts_file = TSFile::from_source_code("package com.company.project.module.submodule.feature;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract scope from deep nested package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.company.project.module.submodule", "Should return all but rightmost component");
        }

        #[test]
        fn test_two_component_package_scope() {
            let ts_file = TSFile::from_source_code("package com.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract scope from two-component package");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com", "Should return first component as scope");
        }

        #[test]
        fn test_single_component_package_scope() {
            let ts_file = TSFile::from_source_code("package myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_scope_node(&ts_file, package_node);
            
            // Single component packages don't have scope/name separation
            assert!(result.is_none(), "Single component package should return None for scope");
        }

        #[test]
        fn test_scope_with_whitespace() {
            let ts_file = TSFile::from_source_code("package   com.example.myapp   ;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_class_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should handle whitespace in package declaration");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.example", "Should extract clean scope despite whitespace");
        }
    }

    // Tests for get_package_scope_node function
    mod get_package_scope_node_tests {
        use super::*;

        #[test]
        fn test_complete_multi_component_identifier() {
            let ts_file = TSFile::from_source_code("package com.example.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract complete scoped identifier");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.example.myapp", "Should return complete package identifier");
        }

        #[test]
        fn test_complete_deep_nested_identifier() {
            let ts_file = TSFile::from_source_code("package com.company.project.module.submodule.feature;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract complete deep nested identifier");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.company.project.module.submodule.feature", "Should return complete deep package identifier");
        }

        #[test]
        fn test_complete_two_component_identifier() {
            let ts_file = TSFile::from_source_code("package com.myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should extract complete two-component identifier");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.myapp", "Should return complete two-component identifier");
        }

        #[test]
        fn test_single_component_identifier() {
            let ts_file = TSFile::from_source_code("package myapp;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_scope_node(&ts_file, package_node);
            
            // Single component packages may not have scoped identifier structure in tree-sitter
            // This could return None or the single identifier depending on parser behavior
            // We'll test for the actual behavior rather than assuming
            if let Some(node) = result {
                let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
                assert_eq!(text, "myapp", "If present, should return the single component");
            }
            // Both None and Some("myapp") are acceptable for single component packages
        }

        #[test]
        fn test_complete_identifier_with_whitespace() {
            let ts_file = TSFile::from_source_code("package   com.example.myapp   ;");
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            let result = get_package_scope_node(&ts_file, package_node);
            
            assert!(result.is_some(), "Should handle whitespace in package declaration");
            let node = result.unwrap();
            let text = ts_file.get_text_from_node(&node).expect("Should get text from node");
            assert_eq!(text, "com.example.myapp", "Should extract clean identifier despite whitespace");
        }
    }

    // Integration tests combining all functions
    mod integration_tests {
        use super::*;

        #[test]
        fn test_comprehensive_package_parsing() {
            let ts_file = TSFile::from_source_code("package com.company.project.myapp;\n\npublic class MyClass {}");
            
            // Get package declaration
            let package_node = get_package_declaration_node(&ts_file);
            assert!(package_node.is_some(), "Should find package declaration");
            let package_node = package_node.unwrap();
            
            // Test all extraction functions
            let class_name = get_package_class_name_node(&ts_file, package_node);
            let scope = get_package_class_scope_node(&ts_file, package_node);
            let complete = get_package_scope_node(&ts_file, package_node);
            
            // Verify all components
            assert!(class_name.is_some(), "Should extract class name");
            let class_name_text = ts_file.get_text_from_node(&class_name.unwrap()).expect("Should get class name text");
            assert_eq!(class_name_text, "myapp");
            
            assert!(scope.is_some(), "Should extract scope");
            let scope_text = ts_file.get_text_from_node(&scope.unwrap()).expect("Should get scope text");
            assert_eq!(scope_text, "com.company.project");
            
            assert!(complete.is_some(), "Should extract complete identifier");
            let complete_text = ts_file.get_text_from_node(&complete.unwrap()).expect("Should get complete text");
            assert_eq!(complete_text, "com.company.project.myapp");
        }

        #[test]
        fn test_minimal_valid_package() {
            let ts_file = TSFile::from_source_code("package a.b;");
            
            let package_node = get_package_declaration_node(&ts_file);
            assert!(package_node.is_some(), "Should find minimal package declaration");
            let package_node = package_node.unwrap();
            
            let class_name = get_package_class_name_node(&ts_file, package_node);
            let scope = get_package_class_scope_node(&ts_file, package_node);
            let complete = get_package_scope_node(&ts_file, package_node);
            
            assert!(class_name.is_some(), "Should extract class name from minimal package");
            let class_name_text = ts_file.get_text_from_node(&class_name.unwrap()).expect("Should get class name text");
            assert_eq!(class_name_text, "b");
            
            assert!(scope.is_some(), "Should extract scope from minimal package");
            let scope_text = ts_file.get_text_from_node(&scope.unwrap()).expect("Should get scope text");
            assert_eq!(scope_text, "a");
            
            assert!(complete.is_some(), "Should extract complete identifier from minimal package");
            let complete_text = ts_file.get_text_from_node(&complete.unwrap()).expect("Should get complete text");
            assert_eq!(complete_text, "a.b");
        }

        #[test]
        fn test_complex_java_file() {
            let java_code = r#"
                /*
                 * Complex Java file with package declaration
                 */
                package com.example.service.impl;
                
                import java.util.List;
                import java.util.Map;
                
                /**
                 * Service implementation class
                 */
                public class ServiceImpl implements Service {
                    // Implementation details
                }
            "#;
            
            let ts_file = TSFile::from_source_code(java_code);
            let package_node = get_package_declaration_node(&ts_file).unwrap();
            
            // Verify all parsing functions work with complex file
            let class_name = get_package_class_name_node(&ts_file, package_node);
            let scope = get_package_class_scope_node(&ts_file, package_node);
            let complete = get_package_scope_node(&ts_file, package_node);
            
            let class_name_text = ts_file.get_text_from_node(&class_name.unwrap()).expect("Should get class name text");
            assert_eq!(class_name_text, "impl");
            
            let scope_text = ts_file.get_text_from_node(&scope.unwrap()).expect("Should get scope text");
            assert_eq!(scope_text, "com.example.service");
            
            let complete_text = ts_file.get_text_from_node(&complete.unwrap()).expect("Should get complete text");
            assert_eq!(complete_text, "com.example.service.impl");
        }

        #[test]
        fn test_edge_case_consistency() {
            // Test that all functions handle the same edge cases consistently
            let test_cases = vec![
                ("", "empty file"),
                ("public class Test {}", "no package"),
                ("invalid syntax", "invalid syntax"),
                ("package;", "malformed package"),
            ];
            
            for (code, description) in test_cases {
                let ts_file = TSFile::from_source_code(code);
                let package_node = get_package_declaration_node(&ts_file);
                
                if package_node.is_none() {
                    // If no package declaration found, other functions should handle gracefully
                    // This test ensures consistency across all functions
                    continue;
                }
                
                let package_node = package_node.unwrap();
                
                // These functions should not panic even with malformed input
                let _class_name = get_package_class_name_node(&ts_file, package_node);
                let _scope = get_package_class_scope_node(&ts_file, package_node);
                let _complete = get_package_scope_node(&ts_file, package_node);
                
                // Test passes if no panics occurred
                println!("Successfully handled edge case: {}", description);
            }
        }
    }
}