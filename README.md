<div align="center">
  <img width="500" alt="syntaxpresso" src="https://github.com/user-attachments/assets/be0749b2-1e53-469c-8d99-012024622ade" />
</div>

<div align="center">
  <img alt="rust" src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust" />
  <img alt="GitHub branch check runs" src="https://img.shields.io/github/check-runs/syntaxpresso/core/develop">
  <img alt="GitHub Downloads (all assets, latest release)" src="https://img.shields.io/github/downloads/syntaxpresso/core/latest/total">
</div>

# Syntaxpresso Core

A standalone Rust-based CLI backend for IDE plugins that provides advanced Java code generation and manipulation capabilities using Tree-Sitter.

## Overview

Syntaxpresso Core is designed as a backend service for IDE plugins, offering comprehensive Java code generation and manipulation through a CLI interface. The tool specializes in JPA (Java Persistence API) entity management, providing developers with automated code generation for complex Java persistence scenarios.

## Features

### Available Commands

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

## Development

### Architecture

Communication follows a **unidirectional request-response model** handled via standard input/output (stdio). The syntaxpresso-core is a stateless CLI application that only prints a single JSON response to stdout before exiting; it never sends commands back to the IDE.

- Request (IDE to Core): The IDE plugin spawns the compiled syntaxpresso-core binary as a new process for each request.
- All required information (the command, file paths, options) is passed as CLI arguments at spawn time.
- Execution (Core): The Rust application parses the arguments, executes the requested command, and performs all logic internally.
- Response (Core to IDE): Upon completion, the Rust core serializes a standard Response object into a single JSON string and prints it to stdout.
- Result (IDE): The IDE plugin captures this stdout, parses the JSON, and uses the structured data (e.g., file paths, success status, or error details) to update its state. The Rust process then terminates.

<div align="center">
  <img width="500" alt="syntaxpresso-archtecture" src="https://github.com/user-attachments/assets/ddd3cd2d-3f03-4bbf-b855-8fc17248b3c2" />
</div>

### Structure

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

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run quality checks
6. Submit a pull request

## Support

For issues, questions, or contributions, please visit the [GitHub repository](https://github.com/syntaxpresso/core).

## Changelog

See [Releases](https://github.com/syntaxpresso/core/releases) for version history and changes.
