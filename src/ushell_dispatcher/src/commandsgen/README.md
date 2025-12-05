# Command Dispatcher Macro

A `no_std`, zero-heap Rust procedural macro that generates type-safe command dispatchers from compact descriptor strings.

## Features

- **Zero heap allocations** - All parsing happens on the stack with compile-time sized buffers
- **`no_std` compatible** - Uses only `core`, perfect for embedded systems
- **Type-safe** - Compile-time signature verification ensures your functions match their descriptors
- **Flexible integer parsing** - Supports decimal, hexadecimal (`0x`), octal (`0o`), and binary (`0b`) literals
- **Quote-aware tokenization** - Handles quoted strings with spaces
- **Comprehensive type support** - Integers, floats, booleans, chars, strings, and hex-encoded byte arrays

## Quick Start

```rust
use command_dispatcher::define_commands;

// Define your command handlers
fn set_led(pin: u8, state: bool) {
    println!("Setting LED on pin {} to {}", pin, state);
}

fn set_pwm(channel: u8, duty: u16, frequency: u32) {
    println!("PWM ch{}: {}% @ {}Hz", channel, duty, frequency);
}

fn echo(message: &str) {
    println!("Echo: {}", message);
}

// Generate the dispatcher
define_commands! {
    mod commands;
    hexstr_size = 32;
    "Bt: set_led, BWD: set_pwm, s: echo"
}

fn main() {
    // Dispatch commands from strings
    commands::dispatch("set_led 13 true").unwrap();
    commands::dispatch("set_pwm 0 50 1000").unwrap();
    commands::dispatch("echo \"Hello, world!\"").unwrap();
}
```

## Descriptor Format

Commands are defined using a compact descriptor syntax:

```
"<descriptor>: <function_path> [<function_path>...], <descriptor>: ..."
```

Each character in a descriptor represents one parameter type:

### Type Mapping Table

| Char | Type | Char | Type | Char | Type |
|------|------|------|------|------|------|
| `B` | `u8` | `W` | `u16` | `D` | `u32` |
| `b` | `i8` | `w` | `i16` | `d` | `i32` |
| `Q` | `u64` | `X` | `u128` | `Z` | `usize` |
| `q` | `i64` | `x` | `i128` | `z` | `isize` |
| `F` | `f64` | `f` | `f32` | | |
| `t` | `bool` | `c` | `char` | `s` | `&str` |
| `h` | `&[u8]` (hex) | `v` | void (no args) | |

### Examples

```rust
// u32, i32, f64, &str, bool
"DdFsb: my_complex_function"

// Single boolean flag
"t: toggle_mode"

// No arguments (void)
"v: reset status"

// Hex-encoded byte array
"h: send_packet"
```

## Macro Syntax

### Inline DSL

```rust
define_commands! {
    mod dispatcher_name;
    hexstr_size = <max_hex_bytes>;
    "<descriptors>"
}
```

### External File

```rust
define_commands! {
    mod dispatcher_name;
    hexstr_size = <max_hex_bytes>;
    path = "commands.txt"
}
```

**Parameters:**
- `mod dispatcher_name` - Name of the generated module
- `hexstr_size` - Maximum byte length for hex-decoded strings (required if using `h` type)
- Descriptor string or file path containing command definitions

## Usage Examples

### Basic Commands

```rust
define_commands! {
    mod cli;
    hexstr_size = 16;
    "D: set_value, t: enable, s: set_name"
}

fn set_value(val: u32) { /* ... */ }
fn enable(state: bool) { /* ... */ }
fn set_name(name: &str) { /* ... */ }

// Call from strings
cli::dispatch("set_value 42").unwrap();
cli::dispatch("enable true").unwrap();
cli::dispatch("set_name \"Device 1\"").unwrap();
```

### Integer Formats

All integer types support multiple bases:

```rust
cli::dispatch("set_value 255").unwrap();      // decimal
cli::dispatch("set_value 0xFF").unwrap();     // hexadecimal
cli::dispatch("set_value 0o377").unwrap();    // octal
cli::dispatch("set_value 0b11111111").unwrap(); // binary
```

### Hex Strings

The `h` type decodes hex strings into byte arrays:

```rust
define_commands! {
    mod net;
    hexstr_size = 6;
    "h: set_mac"
}

fn set_mac(addr: &[u8]) {
    println!("MAC: {:02X?}", addr);
}

net::dispatch("set_mac AABBCCDDEEFF").unwrap();
// Output: MAC: [AA, BB, CC, DD, EE, FF]
```

### Boolean Values

Flexible boolean parsing:

```rust
// All equivalent to true
cli::dispatch("enable 1").unwrap();
cli::dispatch("enable true").unwrap();
cli::dispatch("enable True").unwrap();
cli::dispatch("enable TRUE").unwrap();

// All equivalent to false
cli::dispatch("enable 0").unwrap();
cli::dispatch("enable false").unwrap();
```

### Embedded-Friendly Usage

For embedded systems, use `dispatch_with_buf` to control stack allocation:

```rust
let mut token_buffer: [&str; 10] = [""; 10];
commands::dispatch_with_buf("my_command arg1 arg2", &mut token_buffer).unwrap();
```

## Generated API

The macro generates a complete dispatcher module with:

### Functions

- `dispatch(line: &str) -> Result<(), DispatchError>` - Parse and execute a command
- `dispatch_with_buf(line: &str, buf: &mut [&str]) -> Result<(), DispatchError>` - Buffer-provided version
- `tokenize(line: &str, out: &mut [&str]) -> Result<usize, DispatchError>` - Tokenizer only
- `get_commands() -> &'static [(&'static str, &'static str)]` - List of (name, descriptor) pairs
- `get_function_names() -> Vec<&'static str>` - All registered command names
- `get_datatypes() -> &'static str` - Type mapping help text

### Constants

- `MAX_ARITY` - Maximum argument count across all commands
- `NUM_COMMANDS` - Total number of registered commands
- `MAX_*` - Per-type maximums (e.g., `MAX_U32`, `MAX_STR`)
- `DESCRIPTOR_HELP` - Human-readable type table

### Error Type

```rust
pub enum DispatchError {
    Empty,                      // No input
    UnknownFunction,            // Function not found
    WrongArity { expected: u8 }, // Argument count mismatch
    BadBool,                    // Invalid boolean
    BadChar,                    // Invalid character
    BadUnsigned,                // Invalid unsigned integer
    BadSigned,                  // Invalid signed integer
    BadFloat,                   // Invalid float
    BadHexStr,                  // Invalid hex string
}
```

## Advanced Features

### Introspection

```rust
// List all available commands
for (name, descriptor) in commands::get_commands() {
    println!("{}: {}", name, descriptor);
}

// Get type help
println!("{}", commands::get_datatypes());
```

### Custom Token Buffers

```rust
// Allocate exactly what you need
const BUFFER_SIZE: usize = 1 + commands::MAX_ARITY;
let mut tokens: [&str; BUFFER_SIZE] = [""; BUFFER_SIZE];
commands::dispatch_with_buf(input, &mut tokens)?;
```

## Performance

- **Zero runtime overhead** - All dispatch logic is monomorphized at compile time
- **Stack-only** - No heap allocations, suitable for `no_std` environments
- **Fast lookup** - Generated match statement provides O(1) function lookup
- **Compile-time checks** - Function signatures verified at build time

## Requirements

- Rust 2021 edition or later
- `heapless` crate (for hex string buffers in `no_std`)

## License

Licensed under:

- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.