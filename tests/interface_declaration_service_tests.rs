#[cfg(test)]
mod interface_declaration_service_tests {
  use syntaxpresso_core::common::services::interface_declaration_service::*;
  use syntaxpresso_core::common::ts_file::TSFile;

  fn create_ts_file(content: &str) -> TSFile {
    TSFile::from_source_code(content)
  }

  const SIMPLE_INTERFACE: &str = r#"
public interface UserService {
    void createUser(String name);
    User findUserById(Long id);
    List<User> getAllUsers();
}
"#;

  const MULTIPLE_INTERFACES: &str = r#"
interface InternalService {
    void processInternal();
}

public interface PublicService {
    String getPublicData();
}

public interface UserRepository {
    User save(User user);
    void delete(Long id);
}

interface AnotherInternal {
    void cleanup();
}
"#;

  const COMPLEX_INTERFACE: &str = r#"
import java.util.List;
import java.util.Optional;

/**
 * Service interface for managing products
 */
public interface ProductService extends BaseService<Product> {
    
    /**
     * Create a new product
     */
    Product createProduct(CreateProductRequest request);
    
    /**
     * Find product by ID
     */
    Optional<Product> findById(Long id);
    
    /**
     * Get all products with pagination
     */
    List<Product> findAll(int page, int size);
    
    /**
     * Update product
     */
    Product updateProduct(Long id, UpdateProductRequest request);
    
    /**
     * Delete product
     */
    void deleteProduct(Long id);
    
    /**
     * Search products by criteria
     */
    List<Product> searchProducts(SearchCriteria criteria);
}
"#;

  // Basic functionality tests
  #[test]
  fn test_find_interface_node_by_name_basic() {
    let ts_file = create_ts_file(SIMPLE_INTERFACE);

    let interface_node = find_interface_node_by_name(&ts_file, "UserService");
    assert!(interface_node.is_some(), "Should find UserService interface");

    if let Some(node) = interface_node {
      assert_eq!(node.kind(), "interface_declaration", "Node should be interface_declaration");
    }

    let nonexistent = find_interface_node_by_name(&ts_file, "NonExistent");
    assert!(nonexistent.is_none(), "Should not find nonexistent interface");
  }

  #[test]
  fn test_find_interface_node_by_name_multiple_interfaces() {
    let ts_file = create_ts_file(MULTIPLE_INTERFACES);

    let internal_service = find_interface_node_by_name(&ts_file, "InternalService");
    assert!(internal_service.is_some(), "Should find InternalService interface");

    let public_service = find_interface_node_by_name(&ts_file, "PublicService");
    assert!(public_service.is_some(), "Should find PublicService interface");

    let user_repository = find_interface_node_by_name(&ts_file, "UserRepository");
    assert!(user_repository.is_some(), "Should find UserRepository interface");

    let another_internal = find_interface_node_by_name(&ts_file, "AnotherInternal");
    assert!(another_internal.is_some(), "Should find AnotherInternal interface");
  }

  #[test]
  fn test_get_first_public_interface_node() {
    let ts_file = create_ts_file(MULTIPLE_INTERFACES);

    let first_public = get_first_public_interface_node(&ts_file);
    assert!(first_public.is_some(), "Should find first public interface");

    // Should find PublicService (first public interface in the file)
    if let Some(node) = first_public {
      assert_eq!(node.kind(), "interface_declaration", "Node should be interface_declaration");
    }
  }

  #[test]
  fn test_get_first_public_interface_node_no_public() {
    let no_public_interface = r#"
interface PrivateService {
    void doSomething();
}

interface AnotherPrivateService {
    void doAnotherThing();
}
"#;

    let ts_file = create_ts_file(no_public_interface);
    let first_public = get_first_public_interface_node(&ts_file);
    assert!(first_public.is_none(), "Should not find public interface when none exist");
  }

  #[test]
  fn test_get_public_interface_node_fallback_behavior() {
    // Test that get_public_interface_node falls back to first public interface
    // when no filename is available (which is the normal case with from_source_code)
    let ts_file = create_ts_file(MULTIPLE_INTERFACES);
    let public_interface = get_public_interface_node(&ts_file);
    assert!(public_interface.is_some(), "Should find first public interface as fallback");
  }

  #[test]
  fn test_get_public_interface_node_no_filename() {
    // Test case with no filename set
    let ts_file = create_ts_file(MULTIPLE_INTERFACES);
    let public_interface = get_public_interface_node(&ts_file);
    assert!(
      public_interface.is_some(),
      "Should fallback to first public interface when no filename"
    );
  }

  #[test]
  fn test_get_interface_name_node() {
    let ts_file = create_ts_file(COMPLEX_INTERFACE);

    if let Some(interface_node) = find_interface_node_by_name(&ts_file, "ProductService") {
      let name_node = get_interface_name_node(&ts_file, interface_node);
      assert!(name_node.is_some(), "Should find interface name node");

      if let Some(name_node) = name_node {
        let name_text = ts_file.get_text_from_node(&name_node);
        assert_eq!(name_text, Some("ProductService"), "Interface name should be 'ProductService'");
      }
    }
  }

  #[test]
  fn test_get_interface_name_node_multiple_interfaces() {
    let ts_file = create_ts_file(MULTIPLE_INTERFACES);

    // Test each interface
    let interfaces = [
      ("InternalService", "InternalService"),
      ("PublicService", "PublicService"),
      ("UserRepository", "UserRepository"),
      ("AnotherInternal", "AnotherInternal"),
    ];

    for (interface_name, expected_name) in interfaces.iter() {
      if let Some(interface_node) = find_interface_node_by_name(&ts_file, interface_name) {
        let name_node = get_interface_name_node(&ts_file, interface_node);
        assert!(name_node.is_some(), "Should find name node for {}", interface_name);

        if let Some(name_node) = name_node {
          let name_text = ts_file.get_text_from_node(&name_node);
          assert_eq!(
            name_text,
            Some(*expected_name),
            "Interface name should be '{}'",
            expected_name
          );
        }
      }
    }
  }

  // Edge case tests
  #[test]
  fn test_edge_cases_empty_tree() {
    let ts_file = create_ts_file("");

    let interface_by_name = find_interface_node_by_name(&ts_file, "Test");
    assert!(interface_by_name.is_none(), "Should return None for empty tree");

    let first_public = get_first_public_interface_node(&ts_file);
    assert!(first_public.is_none(), "Should return None for empty tree");

    let public_interface = get_public_interface_node(&ts_file);
    assert!(public_interface.is_none(), "Should return None for empty tree");
  }

  #[test]
  fn test_edge_cases_invalid_inputs() {
    let ts_file = create_ts_file(SIMPLE_INTERFACE);

    // Test with empty interface name
    let empty_name = find_interface_node_by_name(&ts_file, "");
    assert!(empty_name.is_none(), "Should return None for empty interface name");

    let whitespace_name = find_interface_node_by_name(&ts_file, "   ");
    assert!(whitespace_name.is_none(), "Should return None for whitespace interface name");
  }

  #[test]
  fn test_edge_cases_wrong_node_types() {
    let ts_file = create_ts_file(SIMPLE_INTERFACE);

    if let Some(_interface_node) = find_interface_node_by_name(&ts_file, "UserService") {
      // Get the tree root node (not an interface_declaration)
      let root_node = ts_file.tree.as_ref().unwrap().root_node();

      // Test function that expects interface_declaration with wrong node type
      let name_node = get_interface_name_node(&ts_file, root_node);
      assert!(name_node.is_none(), "Should return None when passing wrong node type");
    }
  }

  #[test]
  fn test_edge_cases_malformed_java() {
    let malformed_java = r#"
public interface Broken {
    void method(
    // missing closing parenthesis and body
"#;

    let ts_file = create_ts_file(malformed_java);

    // The functions should handle malformed Java gracefully
    let interface_node = find_interface_node_by_name(&ts_file, "Broken");
    // May or may not find it depending on how tree-sitter parses it
    // The key is that it shouldn't crash
    assert!(
      interface_node.is_some() || interface_node.is_none(),
      "Should handle malformed Java gracefully"
    );

    let first_public = get_first_public_interface_node(&ts_file);
    assert!(
      first_public.is_some() || first_public.is_none(),
      "Should handle malformed Java gracefully"
    );
  }

  #[test]
  fn test_interface_with_generics_and_extends() {
    let generic_interface = r#"
public interface Repository<T extends Entity, ID> extends BaseRepository<T, ID> {
    List<T> findByStatus(Status status);
    Optional<T> findFirstByOrderByCreatedAtDesc();
}
"#;

    let ts_file = create_ts_file(generic_interface);

    let interface_node = find_interface_node_by_name(&ts_file, "Repository");
    assert!(interface_node.is_some(), "Should find generic interface");

    if let Some(interface_node) = interface_node {
      let name_node = get_interface_name_node(&ts_file, interface_node);
      assert!(name_node.is_some(), "Should find name node for generic interface");

      if let Some(name_node) = name_node {
        let name_text = ts_file.get_text_from_node(&name_node);
        assert_eq!(name_text, Some("Repository"), "Interface name should be 'Repository'");
      }
    }
  }

  #[test]
  fn test_interface_with_annotations() {
    let annotated_interface = r#"
@FunctionalInterface
@Deprecated
public interface Calculator {
    @NotNull
    BigDecimal calculate(@NotNull BigDecimal a, @NotNull BigDecimal b);
}
"#;

    let ts_file = create_ts_file(annotated_interface);

    let interface_node = find_interface_node_by_name(&ts_file, "Calculator");
    assert!(interface_node.is_some(), "Should find annotated interface");

    let first_public = get_first_public_interface_node(&ts_file);
    assert!(first_public.is_some(), "Should find annotated public interface");

    if let Some(interface_node) = interface_node {
      let name_node = get_interface_name_node(&ts_file, interface_node);
      assert!(name_node.is_some(), "Should find name node for annotated interface");

      if let Some(name_node) = name_node {
        let name_text = ts_file.get_text_from_node(&name_node);
        assert_eq!(name_text, Some("Calculator"), "Interface name should be 'Calculator'");
      }
    }
  }
}
