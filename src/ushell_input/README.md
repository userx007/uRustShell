# ushell_input

A high-performance, embedded-friendly input handling library for building interactive shell and REPL applications in Rust. Designed for both standard and `no_std` environments with configurable heap/stack allocation.

## Features

- **🚀 Zero-Copy Input Processing** - Efficient text buffer with inline editing using `heapless` data structures
- **⌨️ Comprehensive Key Binding Support** - Full keyboard event handling including arrows, Ctrl combinations, and special keys
- **📜 Command History** - Browsable command history with navigation (Up/Down/PageUp/PageDown)
- **✨ Smart Autocomplete** - Real-time context-aware command completion with Tab cycling
- **🎨 Custom Rendering** - Flexible display system for prompts, inline suggestions, and visual feedback
- **🔧 Embedded-Ready** - Configurable compile-time buffer sizes with optional heap allocation
- **💡 Built-in Help System** - Integrated command listing and inline shortcuts (`#`, `##`, `#h`, etc.)
- **🎯 Generic Type System** - Const generics for zero-runtime-cost configuration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
shell_input = "0.1.0"
```

### Feature Flags

```toml
[dependencies.shell_input]
version = "0.1.0"
features = ["heap-history", "heap-input-buffer"]
```

- **`heap-history`** - Allocate history on the heap (default: stack)
- **`heap-input-buffer`** - Allocate input buffer on the heap (default: stack)


### Type Parameters Explained

```rust
InputParser<NC, FNL, IML, HTC, HME>
```

- **`NC`**: Number of autocomplete candidates (max commands to suggest)
- **`FNL`**: Function Name Length - max characters used for autocomplete matching
- **`IML`**: Input Max Length - maximum characters in input buffer
- **`HTC`**: History Total Capacity - number of history entries
- **`HME`**: History Max Entry - maximum characters per history entry

## Key Bindings

### Editing

| Key | Action |
|-----|--------|
| `Char` | Insert character at cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `Ctrl+U` | Delete from cursor to line start |
| `Ctrl+K` | Delete from cursor to line end |
| `Ctrl+D` | Clear entire buffer |

### Navigation

| Key | Action |
|-----|--------|
| `Arrow Left/Right` | Move cursor |
| `Home` | Move to line start |
| `End` | Move to line end |
| `Arrow Up/Down` | Navigate command history |
| `PageUp/PageDown` | Jump to first/last history entry |

### Completion

| Key | Action |
|-----|--------|
| `Tab` | Cycle autocomplete forward |
| `Shift+Tab` | Cycle autocomplete backward |
| `Enter` | Accept input |

## Built-in Commands

The parser provides special hashtag-prefixed commands:

- `#q` - Quit/exit (returns `false` from `parse_input`)
- `#` - List available commands
- `##` - Show full help (commands + shortcuts + arg types)
- `#h` - Display command history
- `#c` - Clear command history
- `#N` - Execute history entry at index N (e.g., `#0`, `#5`)

## Architecture

```
shell_input/
├── input/
│   ├── buffer.rs      - InputBuffer: Text editing with cursor management
│   │                    • Insertion, deletion, cursor movement
│   │                    • Clear, overwrite operations
│   │                    • Bounded buffer with compile-time size
│   │
│   ├── key_reader.rs  - Key: Platform-specific keyboard event capture
│   │                    • Raw key reading (arrows, Ctrl, special keys)
│   │                    • Cross-platform abstraction layer
│   │
│   ├── parser.rs      - InputParser: Main orchestrator (primary API)
│   │                    • Command autocompletion engine
│   │                    • History navigation integration
│   │                    • Key binding dispatch
│   │                    • Built-in help system
│   │
│   └── renderer.rs    - DisplayRenderer: Terminal output
│                        • Prompt rendering with cursor positioning
│                        • ANSI escape sequences
│                        • Visual feedback (bell, boundary markers)
│
├── history/
│   └── mod.rs         - History: Command history with circular buffer
│                        • Up/Down navigation
│                        • PageUp/PageDown for first/last entry
│                        • Clear and indexed retrieval
│
├── autocomplete/
│   └── mod.rs         - Autocomplete: Real-time suggestion engine
│                        • Prefix matching with candidate cycling
│                        • Tab/Shift+Tab for forward/backward
│                        • Preserves text beyond match window
│
└── terminal/
    └── mod.rs         - Terminal: Low-level terminal control
                         • Raw mode management
                         • Terminal state restoration
                         • RAII-based cleanup
```

## Design Philosophy

### Embedded-First

All data structures use compile-time sizing with `heapless` collections, making the library suitable for embedded systems and `no_std` environments. Optional heap allocation features provide flexibility for standard applications.

### Type-Safe Configuration

Const generics eliminate runtime configuration overhead and catch size mismatches at compile time:

```rust
// Compiler error if you try to store 128-char commands in 64-char history
let parser = InputParser::<10, 32, 64, 20, 128>::new(...);
```

### Zero-Cost Abstractions

The library uses Rust's type system to ensure optimal performance:
- No dynamic dispatch
- Compile-time size checking
- Inline-friendly implementations
- Minimal allocations

## Platform Support

Currently supports:
- Unix-like systems (Linux, macOS, BSD)
- Raw terminal mode input handling
- ANSI escape sequence rendering

Windows support is planned for future releases.

## Performance Characteristics

- **Input latency**: < 1ms for key processing
- **Memory footprint**: Configurable, typically 1-4KB stack or heap
- **No runtime allocations**: After initialization (without heap features)
- **Autocomplete**: O(n) where n = number of candidates

## Use Cases

- 🖥️ Interactive shell applications
- 🔧 REPL environments for embedded devices
- 🎮 Text-based game consoles
- 📟 CLI tools with advanced input requirements
- 🤖 Serial console interfaces
- 🔬 Debugging interfaces and test harnesses

## Integration with Shell Framework

`shell_input` is designed as the core input layer and can be used standalone or as part of a larger shell framework. For complete shell applications with command dispatching, see the `shell` crate which provides a higher-level wrapper around `InputParser`.

## Contributing

Contributions are welcome! Areas of interest:
- Windows platform support
- Additional key bindings
- Unicode input handling
- History persistence backends

Please submit issues and PRs to the repository.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built with:
- [`heapless`](https://crates.io/crates/heapless) - Static data structures for embedded systems
- Raw terminal mode input handling
- ANSI escape sequences for rendering