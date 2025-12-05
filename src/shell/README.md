# shell

A lightweight, `no_std` compatible shell runtime component for building interactive command-line interfaces in Rust.

This crate provides the core `Shell` struct that orchestrates command execution, input parsing, and terminal management for shell-based applications. It's part of the [uRustShell](https://github.com/userx007/uRustShell) framework.

## Features

- **Zero-allocation design** — uses `heapless` data structures for embedded/constrained environments
- **Generic error handling** — accepts any error type that implements `Debug`
- **Dual dispatch model** — separate handlers for commands and shortcuts
- **Automatic terminal management** — sets up raw mode for interactive input
- **Integration with `shell-input`** — leverages autocomplete, history, and advanced editing features

## Usage

```rust
use shell::Shell;

// Define your command and shortcut handlers
fn is_shortcut(input: &str) -> bool {
    input.starts_with('+') || input.starts_with('.')
}

fn command_dispatcher(input: &str) -> Result<(), MyErrorType> {
    // Parse and execute commands
    Ok(())
}

fn shortcut_dispatcher(input: &str) -> Result<(), heapless::String<128>> {
    // Handle shortcuts
    Ok(())
}

// Create and run the shell
let mut shell = Shell::<
    10,   // NC: Number of commands
    32,   // FNL: Function name length
    128,  // IML: Input max length
    256,  // HTC: History total capacity
    16,   // HME: History max entries
    MyErrorType,
>::new(
    get_commands,
    get_datatypes,
    get_shortcuts,
    is_shortcut,
    command_dispatcher,
    shortcut_dispatcher,
    "> ",
);

shell.run();
```

## Generic Parameters

The `Shell` type requires several const generic parameters to configure its behavior:

- `NC` — Maximum number of registered commands
- `FNL` — Maximum length of function names
- `IML` — Maximum input line length
- `HTC` — Total capacity for history storage (in bytes)
- `HME` — Maximum number of history entries
- `ERRTYPE` — Error type returned by the command dispatcher (must implement `Debug`)

## Constructor Parameters

The `Shell::new()` constructor accepts the following function pointers:

- `get_commands` — Returns a static slice of `(name, signature)` tuples for available commands
- `get_datatypes` — Returns a help string describing supported parameter types
- `get_shortcuts` — Returns a help string listing available shortcuts
- `is_shortcut` — Predicate to determine if input should be treated as a shortcut
- `command_dispatcher` — Executes regular commands
- `shortcut_dispatcher` — Executes shortcut commands
- `prompt` — The prompt string to display

## Execution Model

The shell runs in a continuous loop, parsing input and dispatching to the appropriate handler:

1. User enters input via the integrated `InputParser`
2. Input is checked against the `is_shortcut` predicate
3. Either `shortcut_dispatcher` or `command_dispatcher` is called
4. Results are displayed with success/error formatting
5. Loop continues until the parser signals termination

## Error Handling

Both command and shortcut dispatchers can return errors:

- Command errors of type `ERRTYPE` are converted to strings for display
- Shortcut errors are already strings (constrained by `heapless::String<IML>`)
- All errors are formatted consistently: `Error: <message> for line '<input>'`

## Integration with uRustShell

This crate is designed to work within the uRustShell framework, which provides:

- Command registration via `.cfg` files
- Automatic parameter validation and type checking
- Autocomplete and command history
- Advanced line editing capabilities
- Shortcut support with major/minor groups

For a complete working example, see the [uRustShell repository](https://github.com/userx007/uRustShell).

## Dependencies

- `heapless` — Stack-allocated data structures
- `shell-input` — Input parsing and terminal management

## Platform Support

This crate is `no_std` compatible and suitable for embedded systems, though it does require terminal access for raw mode control.

## License

See the [uRustShell repository](https://github.com/userx007/uRustShell) for license information.