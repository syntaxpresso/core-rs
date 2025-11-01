<div align="center">
  <img width="500" alt="syntaxpresso" src="https://github.com/user-attachments/assets/be0749b2-1e53-469c-8d99-012024622ade" />
</div>

<div align="center">
  <img alt="rust" src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust" />
  <img alt="GitHub branch check runs" src="https://img.shields.io/github/check-runs/syntaxpresso/core/develop">
  <img alt="GitHub Downloads (all assets, latest release)" src="https://img.shields.io/github/downloads/syntaxpresso/core/latest/total">
</div>

# Syntaxpresso Core

A powerful Rust-based CLI backend for IDE plugins that provides advanced Java code generation and manipulation capabilities using tree-sitter parsing technology.

## Overview

Syntaxpresso Core is designed as a backend service for IDE plugins, offering comprehensive Java code generation and manipulation through a CLI interface. The tool specializes in JPA (Java Persistence API) entity management, providing developers with automated code generation for complex Java persistence scenarios.

## Features

### 🚀 Core Capabilities
- **Tree-sitter powered parsing**: Leverages tree-sitter-java for accurate Java code analysis and manipulation
- **JSON-based API**: All commands return structured JSON responses for easy IDE plugin integration
- **JPA-focused**: Specialized tooling for Java Persistence API entities and relationships
- **Cross-platform**: Native binaries available for Linux, macOS (Intel & ARM), and Windows

### 📋 Available Commands

#### Entity Management
- **`get-all-jpa-entities`**: Discover all JPA entities in a project
- **`get-all-jpa-mapped-superclasses`**: Find all JPA mapped superclasses
- **`create-jpa-entity`**: Generate new JPA entity classes
- **`get-jpa-entity-info`**: Extract detailed information from existing entities

#### Field Generation
- **`create-jpa-entity-basic-field`**: Add basic fields with JPA annotations
- **`create-jpa-entity-id-field`**: Create ID fields with generation strategies
- **`create-jpa-entity-enum-field`**: Add enum fields with proper JPA mapping

#### Relationship Management
- **`create-jpa-one-to-one-relationship`**: Establish one-to-one entity relationships
- **`create-jpa-many-to-one-relationship`**: Create many-to-one relationships with cascade options

#### Repository & File Operations
- **`create-jpa-repository`**: Generate Spring Data JPA repository interfaces
- **`create-java-file`**: Create basic Java files (classes, interfaces, enums)

## Installation

### Pre-built Binaries

Download the latest release for your platform:

- **Linux**: `syntaxpresso-core-linux-amd64`
- **macOS Intel**: `syntaxpresso-core-macos-amd64`
- **macOS ARM**: `syntaxpresso-core-macos-arm64`
- **Windows**: `syntaxpresso-core-windows-amd64.exe`

### From Source

```bash
git clone https://github.com/syntaxpresso/core-rs.git
cd core-rs
cargo build --release
```

## Usage

All commands follow a consistent pattern and return JSON responses:

```bash
# Basic entity creation
./syntaxpresso-core create-jpa-entity \
  --cwd /path/to/project \
  --package-name com.example.entities \
  --file-name User

# Add a basic field to an entity
./syntaxpresso-core create-jpa-entity-basic-field \
  --cwd /path/to/project \
  --entity-file-path /path/to/User.java \
  --field-name username \
  --field-type String \
  --field-unique \
  --field-nullable false

# Create a one-to-one relationship
./syntaxpresso-core create-jpa-one-to-one-relationship \
  --cwd /path/to/project \
  --owning-side-entity-file-path /path/to/User.java \
  --owning-side-field-name profile \
  --inverse-side-field-name user \
  --inverse-field-type UserProfile
```

### Response Format

All commands return structured JSON responses:

```json
{
  "success": true,
  "data": {
    // Command-specific response data
  }
}
```

Error responses follow this format:

```json
{
  "error": "execution_error",
  "message": "Detailed error description"
}
```

## IDE Plugin Integration

Syntaxpresso Core is designed to be consumed by IDE plugins. The CLI interface provides:

- **Consistent JSON output**: Easy to parse and integrate
- **Comprehensive validation**: Input validation with clear error messages
- **File path handling**: Robust path resolution and validation
- **Base64 encoding support**: For passing source code directly

## Architecture

The project is structured for maintainability and extensibility:

```
src/
├── commands/           # CLI command implementations
│   ├── services/       # Business logic services
│   └── validators/     # Input validation
├── common/
│   ├── services/       # Shared services (annotations, imports, etc.)
│   ├── types/          # Type definitions and configurations
│   └── utils/          # Utility functions
└── responses/          # Response type definitions
```

## Configuration Options

### Field Types
- Support for all Java primitive and wrapper types
- Custom type support with package resolution
- Temporal field handling (`@Temporal` annotations)
- Large object support (`@Lob` annotations)

### JPA Features
- **ID Generation**: AUTO, IDENTITY, SEQUENCE, TABLE strategies
- **Cascade Types**: ALL, PERSIST, MERGE, REMOVE, REFRESH, DETACH
- **Fetch Types**: EAGER, LAZY
- **Collection Types**: List, Set, Map support
- **Mapping Types**: JoinColumn, JoinTable customization

## Development

### Prerequisites
- Rust 2024 Edition
- Cargo package manager

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Code Quality
```bash
cargo fmt
cargo clippy
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run quality checks
6. Submit a pull request

## License

[Add your license information here]

## Support

For issues, questions, or contributions, please visit the [GitHub repository](https://github.com/syntaxpresso/core-rs).
