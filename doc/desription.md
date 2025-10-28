Detailed description of each function found in your `lib.rs`, explaining what it does and how it fits into the macro system:

### Summary of Functions

| Function Name         | Description |
|-----------------------|-------------|
| **define_api**        | Main procedural macro entry point. Parses macro input and generates a dispatcher module with tokenization, argument parsing, and function dispatch logic. |
| **get_function_names**| Returns a sorted list of function names registered in the dispatcher module for diagnostics or UI purposes. |
| **get_cmd_specs**     | Returns static pairs of (function name, descriptor) for diagnostics or help output. |
| **tokenize**          | Splits a command line into tokens, respecting double quotes for string arguments. Caller provides the output buffer. |
| **dispatch**          | Convenience function that allocates a fixed-size buffer and dispatches a command line to the appropriate function. |
| **dispatch_with_buf** | Embedded-friendly dispatch function where the caller provides the token buffer. Parses and invokes the target function. |
| **is_space**          | Helper function to check if a byte is an ASCII space or tab. |
| **parse_bool**        | Parses a string into a boolean value. Accepts variants like `"true"`, `"1"`, `"false"`, `"0"`. |
| **parse_char**        | Parses a one-character string into a `char`. Returns `None` if the string has more than one character. |
| **parse_u**           | Generic parser for unsigned integers using the `FromStr` trait. |
| **parse_i**           | Generic parser for signed integers using the `FromStr` trait. |
| **parse_f**           | Generic parser for floating-point numbers using the `FromStr` trait. |
| **path_last_ident**   | Extracts the last identifier from a `syn::Path`, typically the function name. |
| **sanitize_ident**    | Sanitizes a string to be a valid Rust identifier by replacing non-alphanumeric characters with underscores. |
| **get_descriptor_help** | Returns a help string showing the mapping between descriptor characters and Rust types. |

Would you like me to generate a Markdown or HTML version of this table for documentation purposes?


### 🔧 `define_api(input: TokenStream) -> TokenStream`

**Purpose:**
This is the **main procedural macro**. It parses the macro input (either a DSL string or structured mapping) and generates a Rust module that includes:

- Tokenization logic
- Argument parsing
- Function dispatching
- Diagnostics helpers

**Key responsibilities:**

- Parses the macro input into descriptors and function paths.
- Deduplicates descriptors and computes maximum argument counts.
- Generates:
  - `CallCtx` (stack-based argument buffer)
  - Per-descriptor parsers
  - Per-function wrappers
  - Dispatcher logic
  - Diagnostic helpers like `PARAM_SPECS`, `get_function_names`, etc.

---

### 🧠 `get_function_names() -> Vec<&'static str>`

**Purpose:**
Returns a sorted list of all function names registered in the dispatcher. Useful for diagnostics, UIs, or CLI help.

---

### 📋 `get_cmd_specs() -> &'static [(&'static str, &'static str)]`

**Purpose:**
Returns a static list of `(function_name, descriptor)` pairs. This helps users understand what arguments each function expects.

---

### 🧾 `get_descriptor_help() -> &'static str`

**Purpose:**
Returns a human-readable help string that maps descriptor characters (like `b`, `W`, `f`, etc.) to their corresponding Rust types. Useful for CLI help or documentation.

---

### 🧩 `tokenize(line: &str, out: &mut [&str]) -> Result<usize, DispatchError>`

**Purpose:**
Splits a command line into tokens, respecting quoted strings (e.g., `"hello world"` is one token). The caller provides the output buffer.

---

### 🚀 `dispatch(line: &str) -> Result<(), DispatchError>`

**Purpose:**
Convenience function that allocates a fixed-size buffer and dispatches the command line to the appropriate function.

---

### 🧵 `dispatch_with_buf(line: &str, toks: &mut [&str]) -> Result<(), DispatchError>`

**Purpose:**
Low-level dispatch function for embedded use. The caller provides the token buffer. It:

1. Tokenizes the input
2. Finds the matching function
3. Parses arguments into `CallCtx`
4. Calls the function

---

### 🧼 `is_space(b: u8) -> bool`

**Purpose:**
Checks if a byte is an ASCII space or tab. Used during tokenization.

---

### ✅ `parse_bool(s: &str) -> Option<bool>`

**Purpose:**
Parses a string into a boolean. Accepts `"1"`, `"true"`, `"TRUE"` as `true`, and `"0"`, `"false"`, `"FALSE"` as `false`.

---

### 🔤 `parse_char(s: &str) -> Option<char>`

**Purpose:**
Parses a single-character string into a `char`. Returns `None` if the string has more than one character.

---

### 🔢 `parse_u<T: FromStr>(s: &str) -> Option<T>`

**Purpose:**
Generic parser for unsigned integers (`u8`, `u16`, etc.).

---

### 🔣 `parse_i<T: FromStr>(s: &str) -> Option<T>`

**Purpose:**
Generic parser for signed integers (`i8`, `i16`, etc.).

---

### 🔬 `parse_f<T: FromStr>(s: &str) -> Option<T>`

**Purpose:**
Generic parser for floating-point numbers (`f32`, `f64`).

---

### 🧭 `path_last_ident(p: &syn::Path) -> Option<String>`

**Purpose:**
Extracts the last identifier from a `syn::Path`, typically the function name.

---

### 🧽 `sanitize_ident(s: &str) -> String`

**Purpose:**
Converts a string into a valid Rust identifier by replacing non-alphanumeric characters with underscores. Used for generating wrapper function names.

---

## Key Flow:
Macro Input: DSL or structured mapping is passed to define_api!.

## Macro Expansion:
Generates CallCtx, PARAM_SPECS, MAX_* constants.
Creates per-descriptor parsers and per-function wrappers.

## Runtime:
dispatch() or dispatch_with_buf() is called with a command line.
tokenize() splits the input.
find_entry() locates the function.

Arguments are parsed into CallCtx.
The function is invoked via a wrapper.