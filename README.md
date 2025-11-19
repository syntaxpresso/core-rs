<div align="center">
  <img width="500" alt="syntaxpresso" src="https://github.com/user-attachments/assets/be0749b2-1e53-469c-8d99-012024622ade" />
</div>

<div align="center">
  <img alt="rust" src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust" />
  <img alt="GitHub branch check runs" src="https://img.shields.io/github/check-runs/syntaxpresso/core/develop">
  <img alt="GitHub Downloads (all assets, latest release)" src="https://img.shields.io/github/downloads/syntaxpresso/core/latest/total">
</div>

# Syntaxpresso Core

A high-performance Rust-based CLI backend for IDE plugins that provides advanced Java code generation and manipulation capabilities using Tree-Sitter for precise AST manipulation.

> **Developer-Focused Documentation**: This README contains basic technical architecture, implementation details, and integration guides for developers building IDE plugins or extending the core.

## Quick Reference

**For Plugin Developers:**
- [Installation](#installation-for-developers) - Get the binary or build from source
- [Usage Examples](#usage-examples) - CLI command patterns and JSON responses
- [Plugin Integration Guide](#plugin-integration-guide) - Integrate with Neovim, VSCode, etc.
- [Architecture](#technical-architecture) - Understand the design and data flow

**For Core Contributors:**
- [Development Setup](#development) - Build, test, and contribute
- [Adding New Commands](#adding-new-commands) - Extend functionality
- [Tree-Sitter Patterns](#tree-sitter-query-examples) - Query examples
- [Contributing](#contributing) - Guidelines and process

## What is Syntaxpresso Core?

Syntaxpresso Core is a stateless CLI application designed as a backend service for IDE plugins and automation tools. It provides programmatic Java code generation and manipulation through a JSON-based CLI interface, with an optional interactive Terminal UI for standalone usage.

## Core Capabilities

- **AST-Based Code Manipulation**: Uses Tree-Sitter for precise, syntax-aware Java code parsing and modification
- **JPA Entity Management**: Specialized tooling for Java Persistence API entity generation, fields, and relationships
- **Incremental Parsing**: Leverages Tree-Sitter's incremental parsing for fast, efficient code updates
- **Repository Generation**: Automated Spring Data JPA repository interface creation
- **Project Discovery**: Scans Java projects to discover entities, packages, and mapped superclasses
- **Dual Interface**: Supports both programmatic CLI usage (for IDE integration) and interactive TUI (for standalone use)

## Technical Architecture

### Communication Model

Syntaxpresso Core follows a **stateless request-response model**:

1. **One Process Per Request**: Each invocation spawns a new process that handles a single command and exits
2. **JSON I/O**: All responses are emitted as JSON to stdout for easy parsing by IDE plugins
3. **Exit Codes**: Process exits with code 0 (success) or 1 (error)
4. **No Session State**: Each request is completely independent; no background daemon or persistent state

This architecture ensures:
- **Reliability**: Process isolation prevents state corruption
- **Simplicity**: No complex IPC or socket communication required
- **Language Agnostic**: Any language that can spawn processes and parse JSON can integrate
- **Resource Efficiency**: No background processes consuming memory when idle

### Core Technologies

- **Tree-Sitter**: Powers the AST parsing and manipulation engine
  - Incremental parsing for fast updates
  - Precise byte-level node positioning
  - Error-resilient parsing (can work with incomplete code)
- **Rust**: Ensures memory safety and high performance
- **Clap**: Provides type-safe CLI argument parsing
- **Serde**: Handles JSON serialization/deserialization

### Design Philosophy

1. **AST-First Approach**: All code modifications are performed through Tree-Sitter AST manipulation, not string concatenation or regex
2. **Incremental Updates**: Uses Tree-Sitter's incremental parsing to efficiently update only affected portions of the syntax tree
3. **Path Security**: All file operations validate paths against traversal attacks using canonicalization and containment checks
4. **Type Safety**: Strongly-typed commands and responses prevent runtime errors
5. **Separation of Concerns**: Clear layering between CLI, commands, services, and Tree-Sitter operations

## Binary Variants

The core is compiled in two variants to support different integration approaches:

### CLI-only (Default) - `syntaxpresso-core-{platform}-{arch}`

**Target Use Case**: Build IDE plugins with custom UI or programmatic code manipulation

- **Size**: ~3.4MB (optimized for minimal footprint)
- **Dependencies**: Core Rust libraries + Tree-Sitter only
- **Interface**: Pure CLI with JSON output
- **Performance**: No UI overhead

**Use when:**
- Building IDE plugins with your own custom interface (Neovim with custom Lua UI, VSCode with custom webviews, etc.)
- Programmatically manipulating Java code in scripts or automation tools
- Integrating into build pipelines or CI/CD systems
- Need only JSON-based programmatic access

### UI-enabled - `syntaxpresso-core-ui-{platform}-{arch}`

**Target Use Case**: Build IDE plugins using the built-in TUI or standalone terminal usage

- **Size**: ~4.0MB (includes Ratatui + Crossterm)
- **Dependencies**: Core libraries + TUI framework
- **Interface**: Interactive terminal forms + CLI fallback (supports all CLI commands)
- **Performance**: Minimal UI overhead (< 100ms startup)

**Use when:**
- Building IDE plugins that leverage the provided Terminal UI (e.g., Neovim plugin that spawns TUI forms)
- Using as a standalone terminal application without any IDE
- Need both programmatic CLI access AND interactive forms
- Want ready-to-use TUI without implementing your own interface

## Features & Command Reference

### Discovery & Information Commands

- **`get-all-jpa-entities`**: Scans project for all JPA entity classes (annotated with `@Entity`)
- **`get-all-jpa-mapped-superclasses`**: Finds all JPA mapped superclasses (annotated with `@MappedSuperclass`)
- **`get-jpa-entity-info`**: Extracts detailed metadata from an entity (fields, relationships, annotations)
- **`get-all-packages`**: Lists all Java package names in the project by scanning directory structure
- **`get-java-basic-types`**: Returns supported Java basic field types, optionally filtered for ID types

### File Generation Commands

- **`create-jpa-entity`**: Generates a new JPA entity class with package declaration and `@Entity` annotation
- **`create-java-file`**: Creates basic Java files (classes, interfaces, enums, records, annotations)
- **`create-jpa-repository`**: Generates Spring Data JPA repository interfaces extending `JpaRepository<Entity, ID>`

### Field Generation Commands

- **`create-jpa-entity-basic-field`**: Adds basic fields to entities with JPA column annotations
- **`create-jpa-entity-id-field`**: Creates ID fields with generation strategies (AUTO, IDENTITY, SEQUENCE, UUID)
- **`create-jpa-entity-enum-field`**: Adds enum fields with `@Enumerated` annotation and mapping type

### Relationship Management Commands

- **`create-jpa-one-to-one-relationship`**: Establishes bidirectional one-to-one entity relationships
- **`create-jpa-many-to-one-relationship`**: Creates many-to-one relationships with cascade and fetch type options

### UI Commands (UI-enabled binary only)

The UI-enabled binary includes interactive terminal forms for:

- **`ui create-java-file`**: Interactive form to create Java files
- **`ui create-jpa-entity`**: Interactive form to create JPA entities
- **`ui create-jpa-entity-basic-field`**: Interactive form to add fields to entities
- **`ui create-jpa-one-to-one-relationship`**: Interactive form to create entity relationships
- **`ui create-jpa-repository`**: Interactive form to create JPA repositories

```bash
# Launch interactive UI for creating a Java file
./syntaxpresso-core ui create-java-file --cwd /path/to/project

# Launch UI to add a field to an entity
./syntaxpresso-core ui create-jpa-entity-basic-field \
  --cwd /path/to/project \
  --entity-file-path /path/to/User.java \
  --entity-file-b64-src <base64-encoded-source>

# Launch UI to create a JPA repository for an entity
./syntaxpresso-core ui create-jpa-repository \
  --cwd /path/to/project \
  --entity-file-path /path/to/User.java \
  --entity-file-b64-src <base64-encoded-source>
```

## Installation for Developers

### Option 1: Download Pre-built Binaries

Download from [GitHub Releases](https://github.com/syntaxpresso/core/releases):

**CLI-only binaries** (recommended for IDE integration):
- `syntaxpresso-core-linux-amd64` - Linux x86_64
- `syntaxpresso-core-macos-amd64` - macOS Intel
- `syntaxpresso-core-macos-arm64` - macOS Apple Silicon
- `syntaxpresso-core-windows-amd64.exe` - Windows x86_64

**UI-enabled binaries** (for standalone use):
- `syntaxpresso-core-ui-linux-amd64` - Linux x86_64
- `syntaxpresso-core-ui-macos-amd64` - macOS Intel
- `syntaxpresso-core-ui-macos-arm64` - macOS Apple Silicon
- `syntaxpresso-core-ui-windows-amd64.exe` - Windows x86_64

### Option 2: Build from Source

Requires Rust 2024 Edition toolchain.

**CLI-only:**

```bash
cargo build --release
```

**UI-enabled:**

```bash
cargo build --release --features ui
```

Binary location: `target/release/syntaxpresso-core`

## Usage Examples

### CLI Interface (Programmatic)

All commands follow a consistent pattern with JSON responses suitable for parsing by IDE plugins:

```bash
# Discover all JPA entities in project
./syntaxpresso-core get-all-jpa-entities --cwd /path/to/project

# Get supported Java types
./syntaxpresso-core get-java-basic-types \
  --cwd /path/to/project \
  --field-type-kind all  # Options: all-types, id-types

# Create a JPA entity
./syntaxpresso-core create-jpa-entity \
  --cwd /path/to/project \
  --package-name com.example.domain \
  --file-name User

# Add a field to existing entity
./syntaxpresso-core create-jpa-entity-basic-field \
  --cwd /path/to/project \
  --entity-file-path /absolute/path/to/User.java \
  --field-name email \
  --field-type String \
  --field-unique \
  --field-nullable false

# Create bidirectional one-to-one relationship
./syntaxpresso-core create-jpa-one-to-one-relationship \
  --cwd /path/to/project \
  --owning-side-entity-file-path /absolute/path/to/User.java \
  --owning-side-field-name profile \
  --inverse-side-field-name user \
  --inverse-field-type UserProfile
```

**JSON Response Format:**

Success:
```json
{
  "command": "create-jpa-entity",
  "cwd": "/path/to/project",
  "succeed": true,
  "data": {
    "fileType": "User",
    "filePackageName": "com.example.domain",
    "filePath": "/path/to/project/src/main/java/com/example/domain/User.java"
  }
}
```

Error:
```json
{
  "command": "create-jpa-entity",
  "cwd": "/path/to/project",
  "succeed": false,
  "errorReason": "Package name 'invalid..package' contains invalid characters"
}
```

### Interactive UI Interface (UI-enabled binary only)

For standalone terminal usage, the UI-enabled binary provides interactive forms:

```bash
# Interactive Java file creation
./syntaxpresso-core ui create-java-file --cwd /path/to/project

# Interactive entity creation
./syntaxpresso-core ui create-jpa-entity --cwd /path/to/project

# Interactive field addition (requires existing entity)
./syntaxpresso-core ui create-jpa-entity-basic-field \
  --cwd /path/to/project \
  --entity-file-path /path/to/User.java \
  --entity-file-b64-src $(base64 -w 0 /path/to/User.java)

# Interactive repository generation
./syntaxpresso-core ui create-jpa-repository \
  --cwd /path/to/project \
  --entity-file-path /path/to/User.java \
  --entity-file-b64-src $(base64 -w 0 /path/to/User.java)
```

**UI Navigation:**
- `Tab` / `Shift+Tab` - Navigate fields
- `Enter` - Submit / Select
- `Esc` - Cancel/close
- `↑↓` - Navigate lists
- `i` / `a` - Enter insert mode

## Development

### System Requirements

- **Rust Toolchain**: Rust 2024 Edition or later
- **Cargo**: Latest stable version
- **Platform**: Linux, macOS, or Windows

### Project Structure

```
src/
├── commands/              # Command layer - CLI handlers
│   ├── services/          # Business logic services
│   ├── validators/        # Input validation
│   └── *_command.rs       # Individual command implementations
├── common/
│   ├── services/          # Tree-Sitter AST manipulation services
│   │   ├── annotation_service.rs
│   │   ├── class_declaration_service.rs
│   │   ├── field_declaration_service.rs
│   │   ├── import_declaration_service.rs
│   │   └── ...
│   ├── types/             # Type definitions and domain models
│   │   ├── java_basic_types.rs
│   │   ├── java_field_modifier.rs
│   │   ├── java_id_generation.rs
│   │   └── ...
│   ├── utils/             # Utility functions
│   │   ├── case_util.rs
│   │   ├── path_security_util.rs
│   │   └── path_util.rs
│   ├── query.rs           # Tree-Sitter query builder
│   └── ts_file.rs         # Core Tree-Sitter file abstraction
├── responses/             # Response type definitions
├── ui/                    # Terminal UI (feature-gated with 'ui')
│   ├── forms/             # Interactive TUI forms
│   ├── form_trait.rs      # Form interface definition
│   ├── runner.rs          # UI runtime
│   └── widgets.rs         # Reusable UI components
├── lib.rs                 # Library entry point
└── main.rs                # CLI entry point

tests/                     # Integration tests
```

### Architecture Layers

The codebase follows a clean layered architecture with strict separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│ Frontend (IDE Plugin / Script / Terminal)                   │
│  - Spawns binary with arguments                             │
│  - Captures JSON from stdout                                │
│  - Parses and consumes response                             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ Clap CLI Parser (main.rs)                                   │
│  - Validates arguments                                      │
│  - Routes to handler                                        │
└────────────────────────┬────────────────────────────────────┘
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
┌──────────────────────┐  ┌──────────────────────┐
│ Interactive UI Path  │  │ Programmatic Path    │
│ (#[cfg(feature="ui)])│  │ (Default)            │
│                      │  │                      │
│ - TUI forms          │  │ - Direct execution   │
│ - User interaction   │  │ - Calls commands     │
│ - Calls commands     │  │                      │
└──────────┬───────────┘  └──────────┬───────────┘
           │                         │
           └──────────┬──────────────┘
                      ▼
           ┌────────────────────────┐
           │ Command Layer          │
           │  src/commands/*.rs     │
           │                        │
           │  - Owns command names  │
           │  - Validates inputs    │
           │  - Calls services      │
           │  - Builds Response<T>  │
           └──────────┬─────────────┘
                      ▼
           ┌────────────────────────┐
           │ Service Layer          │
           │  src/commands/services/│
           │  src/common/services/  │
           │                        │
           │  - Business logic      │
           │  - Tree-Sitter ops     │
           │  - File I/O            │
           │  - Returns domain objs │
           └──────────┬─────────────┘
                      ▼
           ┌────────────────────────┐
           │ Tree-Sitter Layer      │
           │  src/common/ts_file.rs │
           │                        │
           │  - AST parsing         │
           │  - Incremental updates │
           │  - Query execution     │
           │  - Node manipulation   │
           └────────────────────────┘
```

### Key Components

#### TSFile (`src/common/ts_file.rs`)

The core abstraction for Tree-Sitter file operations:

```rust
pub struct TSFile {
    pub language: Language,
    parser: Parser,
    pub tree: Option<Tree>,
    pub source_code: String,
    // ...
}
```

**Key capabilities:**
- **Incremental Parsing**: `apply_incremental_edit()` updates AST efficiently
- **Query API**: Fluent query builder for Tree-Sitter queries
- **Node Manipulation**: `replace_text_by_node()`, `insert_text()`
- **Path Security**: `save_as()` with path traversal protection
- **Multiple Constructors**: Load from file, string, or base64-encoded source

**Example usage:**
```rust
// Parse a Java file
let mut ts_file = TSFile::from_file(Path::new("User.java"))?;

// Query for class declaration
let nodes = ts_file.query("(class_declaration name: (identifier) @name)")?;

// Modify a node efficiently (incremental parse)
if let Some(node) = nodes.first() {
    ts_file.replace_text_by_node(&node, "new content");
}

// Save with security validation
ts_file.save_as(Path::new("output.java"), Path::new("/safe/base/dir"))?;
```

#### Service Layer Pattern

Services encapsulate Tree-Sitter operations for specific Java constructs:

**Example: `ClassDeclarationService`**
```rust
impl ClassDeclarationService {
    pub fn insert_field_in_class_body(
        ts_file: &mut TSFile,
        class_node: &Node,
        field_code: &str,
    ) -> Result<(), String>
}
```

Each service:
- Operates on `TSFile` instances
- Uses Tree-Sitter queries to locate nodes
- Performs incremental updates
- Returns domain objects or errors

#### Response Types (`src/responses/`)

All commands return a standardized `Response<T>` structure:

```rust
pub struct Response<T> {
    pub command: String,
    pub cwd: String,
    pub succeed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "errorReason")]
    pub error_reason: Option<String>,
}
```

Success response:
```json
{
  "command": "create-jpa-entity",
  "cwd": "/path/to/project",
  "succeed": true,
  "data": {
    "fileType": "User",
    "filePackageName": "com.example.entities",
    "filePath": "/path/to/User.java"
  }
}
```

Error response:
```json
{
  "command": "create-jpa-entity",
  "cwd": "/path/to/project",
  "succeed": false,
  "errorReason": "Package name is invalid"
}
```

### Design Patterns

#### 1. Stateless Execution

Each invocation is independent:
- No shared state between invocations
- No background daemons
- Process spawns → executes → outputs JSON → exits

**Benefits:**
- Simple integration model
- No state corruption
- Easy to test and debug

#### 2. Incremental Parsing

All AST modifications use Tree-Sitter's incremental parsing:

```rust
// DON'T: Full reparse (slow)
ts_file.source_code.replace_range(start..end, new_text);
ts_file.tree = parser.parse(&ts_file.source_code, None);

// DO: Incremental update (fast)
ts_file.apply_incremental_edit(start_byte, end_byte, new_text);
```

**Performance impact:**
- Incremental: ~1-5ms for typical edits
- Full reparse: ~50-200ms for large files

#### 3. Path Security

All file saves validate against path traversal:

```rust
// Validates path is contained within base_dir
let validator = PathSecurityValidator::new(base_dir)?;
let safe_path = validator.validate_path_containment(user_path)?;
```

Prevents attacks like:
- `../../../../etc/passwd`
- Symlink escapes
- Relative path manipulation

#### 4. Feature-Gated UI

UI code is feature-gated to minimize binary size:

```rust
#[cfg(feature = "ui")]
pub mod ui;
```

**Build flags:**
- Default: `cargo build` → CLI-only (~3.4MB)
- With UI: `cargo build --features ui` → Full (~4.0MB)

### Development Workflow

#### Building from Source

**CLI-only variant:**
```bash
cargo build --release
```

**UI-enabled variant:**
```bash
cargo build --release --features ui
```

Output: `target/release/syntaxpresso-core`

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test class_declaration_service_tests

# Run with output
cargo test -- --nocapture
```

Test coverage includes:
- Tree-Sitter service operations
- Path security validation
- Import declaration management
- Annotation handling

Still creating tests.

#### Code Quality Checks

The project follows custom formatting parameters defined in rustfmt.toml.

### Adding New Commands

Follow these steps to add a new command:

1. **Define Response Type** (`src/responses/`)
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MyCommandResponse {
    pub field: String,
}
```

2. **Implement Command** (`src/commands/`)
```rust
pub fn my_command(args: Args) -> Response<MyCommandResponse> {
    // Call service layer
    match my_service::do_work(&args) {
        Ok(result) => Response::success("my-command", &args.cwd, result),
        Err(e) => Response::error("my-command", &args.cwd, e.to_string()),
    }
}
```

3. **Add CLI Binding** (`src/main.rs`)
```rust
#[derive(Subcommand)]
enum Commands {
    MyCommand {
        #[arg(long)]
        my_arg: String,
    },
}
```

4. **Implement Service** (`src/commands/services/` or `src/common/services/`)
```rust
pub fn do_work(args: &Args) -> Result<MyCommandResponse, String> {
    let mut ts_file = TSFile::from_file(&args.file)?;
    // Use Tree-Sitter to manipulate AST
    // ...
    Ok(MyCommandResponse { field: "result".to_string() })
}
```

### Integration Testing

Create a test Java project structure:

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_entity_creation() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create package structure
    let entity_dir = project_path.join("src/main/java/com/example/entities");
    fs::create_dir_all(&entity_dir).unwrap();
    
    // Test command
    let result = create_jpa_entity_command::execute(/* ... */);
    
    assert!(result.succeed);
    assert!(entity_dir.join("User.java").exists());
}
```

### Plugin Integration Guide

#### Neovim Plugin Development

To develop or integrate with the Neovim plugin:

1. **Build the core:**
   ```bash
   cargo build --release --features ui
   ```

2. **Configure plugin to use local build:**
   ```lua
   require("syntaxpresso").setup({
     executable_path = "/absolute/path/to/target/release/syntaxpresso-core",
   })
   ```

3. **Integration pattern:**
   ```lua
   -- Spawn process and capture JSON output
   local handle = vim.system({
     executable_path,
     "create-jpa-entity",
     "--cwd", cwd,
     "--package-name", package,
     "--file-name", filename
   })
   
   local result = handle:wait()
   local response = vim.json.decode(result.stdout)
   
   if response.succeed then
     -- Process response.data
   else
     -- Handle response.errorReason
   end
   ```

#### VSCode / Other IDE Integration

The CLI interface is language-agnostic. Example in TypeScript:

```typescript
import { spawn } from 'child_process';

function executeCommand(args: string[]): Promise<any> {
  return new Promise((resolve, reject) => {
    const proc = spawn('./syntaxpresso-core', args);
    let stdout = '';
    
    proc.stdout.on('data', (data) => stdout += data);
    proc.on('close', (code) => {
      if (code === 0) {
        resolve(JSON.parse(stdout));
      } else {
        reject(new Error(stdout));
      }
    });
  });
}

// Usage
const response = await executeCommand([
  'create-jpa-entity',
  '--cwd', projectPath,
  '--package-name', 'com.example.domain',
  '--file-name', 'User'
]);
```

### Performance Benchmarks

Typical operation timings on modern hardware:

| Operation | Time | Notes |
|-----------|------|-------|
| Process spawn | ~10-20ms | Cold start overhead |
| Parse 1000 LOC Java file | ~5-10ms | Initial parse |
| Incremental field addition | ~1-3ms | Uses incremental parsing |
| Full file reparse | ~50-200ms | Avoided by incremental updates |
| JSON serialization | <1ms | Negligible overhead |
| Total command execution | ~20-50ms | End-to-end for typical operations |

## Contributing

Contributions are welcome! 

This project follows standard Rust development practices.

Always code for interfaces.

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests and ensure they pass
5. Run code quality checks (`cargo fmt`, `cargo clippy`)
6. Commit your changes
7. Push to your fork
8. Open a Pull Request with a clear description

### Areas for Contribution

- **New Commands**: Additional JPA features, validation, etc.
- **Performance**: Optimize Tree-Sitter queries or parsing logic
- **Code improvements**: This project is far for perfect and you can make it better.
- **Testing**: Increase test coverage
- **Documentation**: Improve docs, add examples
- **UI/UX**: Enhance interactive forms

## Technical Support & Resources

- **Issues & Bug Reports**: [GitHub Issues](https://github.com/syntaxpresso/core/issues)
- **Discussions**: [GitHub Discussions](https://github.com/syntaxpresso/core/discussions)
- **Source Code**: [GitHub Repository](https://github.com/syntaxpresso/core)
- **Release Notes**: [GitHub Releases](https://github.com/syntaxpresso/core/releases)
