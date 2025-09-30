# MLia Development Guide

## Quick Start

This workspace is configured for optimal development of the MLia programming language. Below are the key shortcuts and workflows to get you started.

## Keyboard Shortcuts

### Running MLia Programs
- **F5**: Run MLia with example file (`docs/main.mlia`)
- **Ctrl+F5**: Run MLia interpreter (interactive mode)
- **Shift+F5**: Run MLia with custom file (prompts for file path)

### Building and Testing
- **Ctrl+Shift+B**: Build project (`cargo build`)
- **Ctrl+Shift+T**: Run all tests (`cargo test`)
- **Ctrl+Shift+C**: Check code (`cargo check`)
- **Ctrl+Shift+L**: Run Clippy lints (`cargo clippy`)
- **Ctrl+Shift+F**: Format code (`cargo fmt`)

### Component Testing
- **Ctrl+Alt+P**: Test parser module
- **Ctrl+Alt+T**: Test tokenizer module

### Debugging
- **F9**: Debug MLia interpreter
- **Shift+F9**: Debug MLia with example file

### Development Workflow
- **Ctrl+Shift+Enter**: Full build and test cycle
- **Ctrl+Alt+Enter**: Pre-commit checks (format, lint, test)
- **Ctrl+Alt+W**: Start watch mode (auto-rebuild on changes)

### Documentation
- **Ctrl+Alt+D**: Generate and open documentation

## VS Code Tasks

Access tasks via `Ctrl+Shift+P` → "Tasks: Run Task":

### Build Tasks
- `cargo build` - Standard build
- `cargo build release` - Optimized build
- `cargo check` - Fast syntax/type checking
- `cargo clippy` - Advanced linting
- `cargo fmt` - Code formatting
- `cargo clean` - Clean build artifacts

### Test Tasks
- `cargo test` - Run all tests
- `cargo test verbose` - Run tests with detailed output
- `test parser` - Test parser module only
- `test tokenizer` - Test tokenizer module only

### MLia Interpreter Tasks
- `run mlia` - Run interpreter in interactive mode
- `run mlia with example` - Run with `docs/main.mlia`
- `run mlia with custom file` - Run with user-specified file
- `run mlia release` - Run optimized build

### Development Tasks
- `full build and test` - Complete development cycle
- `pre-commit checks` - Quick validation before committing
- `cargo watch` - Auto-rebuild on file changes
- `cargo doc` - Generate documentation

### LLVM Preparation
- `check llvm-config` - Verify LLVM installation
- `install llvm dependencies` - Add LLVM codegen dependencies

## Debug Configurations

Available in the Run and Debug panel:

1. **Debug MLia Interpreter** - Debug the interpreter itself
2. **Debug MLia with Example File** - Debug execution of `docs/main.mlia`
3. **Debug MLia with Custom File** - Debug with user-specified file
4. **Debug Parser Tests** - Debug parser module tests
5. **Debug Tokenizer Tests** - Debug tokenizer module tests
6. **Debug All Tests** - Debug complete test suite

## File Structure

```
mlia/
├── src/
│   ├── main.rs         # Main interpreter entry point
│   ├── parser.rs       # Pomelo-based LR parser
│   └── tokenizer.rs    # Lexical analyzer
├── docs/
│   ├── main.mlia       # Example MLia program
│   ├── ejemplos.md     # More examples
│   └── bnf.txt         # Grammar specification
└── .vscode/
    ├── launch.json     # Debug configurations
    ├── tasks.json      # Build/test tasks
    ├── settings.json   # Workspace settings
    └── extensions.json # Recommended extensions
```

## MLia Language Features

The MLia language supports:

- **Variable Declarations**: `decl x <- value`
- **Functions**: `decl f <- fun x y -> body`
- **Pattern Matching**: `match expr with | pattern -> result`
- **While Loops**: `while condition do ... done`
- **Built-ins**: `print`, arithmetic operations
- **Comments**: `(* comment *)`

## Code Snippets

Type these prefixes and press Tab for code completion:

- `decl` - Variable declaration
- `fun` - Function declaration
- `match` - Pattern matching
- `while` - While loop
- `if` - If-then-else
- `print` - Print statement
- `comment` - Comment block

## Development Workflow

### Starting Development
1. Open the workspace in VS Code
2. Install recommended extensions when prompted
3. Run `Ctrl+Shift+Enter` to verify everything builds and tests pass

### Working on Parser
1. Edit `src/parser.rs`
2. Run `Ctrl+Alt+P` to test parser changes
3. Use `F9` to debug parser issues

### Working on Tokenizer
1. Edit `src/tokenizer.rs`
2. Run `Ctrl+Alt+T` to test tokenizer changes
3. Test with example files using `F5`

### Testing Language Features
1. Edit `docs/main.mlia` or create new `.mlia` files
2. Run with `F5` (example) or `Shift+F5` (custom file)
3. Debug language execution with `Shift+F9`

### Before Committing
- Run `Ctrl+Alt+Enter` for pre-commit checks
- Ensure all tests pass and code is formatted

## LLVM Backend Development

When ready to add LLVM codegen:

1. Run task "check llvm-config" to verify LLVM installation
2. Run task "install llvm dependencies" to add Inkwell
3. Create new module `src/codegen.rs`
4. Add LLVM IR generation functions
5. Integrate with existing parser AST

## Troubleshooting

### Rust-Analyzer Issues
- Press `Ctrl+Alt+R` to reload Rust-analyzer
- Check that Cargo.toml dependencies are up to date

### Build Issues
- Run `cargo clean` task
- Verify Rust toolchain: `rustc --version`

### LLVM Issues
- Ensure LLVM is installed: `llvm-config --version`
- Check LLVM version compatibility with Inkwell

### Performance Issues
- Use release builds for performance testing
- Profile with `cargo flamegraph` (requires installation)

## Extensions

The workspace recommends these key extensions:

- **rust-lang.rust-analyzer** - Rust language support
- **vadimcn.vscode-lldb** - Debugging support
- **usernamehw.errorlens** - Inline error display
- **eamodio.gitlens** - Enhanced Git integration
- **gruntfuggly.todo-tree** - TODO/FIXME highlighting

## Resources

- [Pomelo Documentation](https://docs.rs/pomelo)
- [Rust Book](https://doc.rust-lang.org/book/)
- [LLVM Documentation](https://llvm.org/docs/)
- [Inkwell Documentation](https://docs.rs/inkwell)
