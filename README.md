# uRustShell

### ushell2 : the shell public interface
[![Crates.io](https://img.shields.io/crates/v/ushell2)](https://crates.io/crates/ushell2)
[![License](https://img.shields.io/crates/l/ushell2)](https://opensource.org/licenses/MIT)

### ushell_dispatcher : shell subcomponent 
[![Crates.io](https://img.shields.io/crates/v/ushell_dispatcher)](https://crates.io/crates/ushell_dispatcher)
[![License](https://img.shields.io/crates/l/ushell_dispatcher)](https://opensource.org/licenses/MIT)

### ushell_input : shell subcomponent 
[![Crates.io](https://img.shields.io/crates/v/ushell_input)](https://crates.io/crates/ushell_input)
[![License](https://img.shields.io/crates/l/ushell_input)](https://opensource.org/licenses/MIT)


## Description
This Rust crate provides a framework for building shell-based applications.
Once launched, the application presents an interactive prompt that allows users to execute custom-defined commands.

The key advantage of this crate is the **simplicity of adding new commands** with any parameter configuration.

In practice, defining a new command requires adding just **one line** to a configuration file.
For example, given the function:

```rust
fn send (port: &str, baud: u32, data: &[u8]) {
    // implementation
}
```

you can register it with a single line:

```
sDh : send_uart
```
From that point on, the command automatically benefits from full validation — including parameter count, types, and value ranges — without requiring any additional code.

Integration into your own code is also straightforward:

```rust
use shell_runner::Shell;

fn main() {
    Shell::new().run();
}
```

## Main Features

* **Autocomplete** for faster command entry
* **Command history** with recall support
* **Editing mode** with intuitive navigation:

  * Move cursor left/right using arrow keys
  * Insert text under the cursor
  * `DEL` deletes the character under the cursor
  * `Ctrl-D` deletes the entire line
  * `Ctrl-U` deletes from the cursor to the beginning
  * `Ctrl-K` deletes from the cursor to the end
* **Automatic parameter validation** (checks number, type, and range)
* **Simple command registration** — add new commands with a single line in a configuration file
* **Lightweight implementation** — commands are regular functions without special wrappers
* **`no_std` by default**, making it suitable for heapless or embedded environments

  * Optional heap usage can be enabled for larger command histories
* **Flexible parameter types**, including:

  * Unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
  * Signed integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
  * Floating-point: `f32`, `f64`
  * Other: `char`, `bool`, `string`
  * Byte arrays as hex strings (e.g. `AABBCC` → `{0xAA, 0xBB, 0xCC}`)
* **Flexible number formats**: decimal (`1234`), hexadecimal (`0x3264`), octal (`0o3344`), binary (`0b11110011`)
* **Shortcut support** for quick command execution (e.g. `##`, `.!aa`, etc.)


## Parameter description
As a fast hint, the unsigned values having a bigger absolute value are marked with upper case symbols..

| Symbol | Type   | Remark                                              |
|--------|--------|-----------------------------------------------------|
| B      | u8     | byte  - unsigned                                    |
| W      | u16    | word  - unsigned                                    |
| D      | u32    | dword - unsigned                                    |
| Q      | u64    | qword - unsigned                                    |
| X      | u128   | xword - unsigned                                    |
| Z      | usize  | size  - unsigned                                    |
| F      | f64    | double                                              |
| b      | i8     | byte  - signed                                      |
| w      | i16    | word  - signed                                      |
| d      | i32    | dword - signed                                      |
| q      | i64    | qword - signed                                      |
| x      | i128   | xword - signed                                      |
| z      | isize  | size  - signed                                      |
| f      | f32    | float                                               |
| c      | char   | char                                                |
| t      | bool   | 1,true,True,TRUE, 0,false,False,FALSE               |
| s      | string | hello or "hey you"                                  |
| h      | array  | in hexadecimal, hexlified form, e.g. 12A6FFE3677    |
| v      | void   | no params ..                                        |

## Building arguments rule
This is quite simple and the array below shows few example ..

| Function implementation                            | Rule | Config entry   | Shell call                    |
|----------------------------------------------------|------|----------------|-------------------------------|
| fn init ( )                                   {..} | v    | v   : init     | init                          |
| fn read (descr:i8, nbytes:u32)                {..} | bD   | bD  : read     | read 0 1024                   |
| fn write (filename:&str, nbytes:u64, val:u8)  {..} | sQB  | sQb : write    | write output.txt 32768 0xFF   |
| fn led (onoff: bool)                          {..} | t    | t   : led      | led true                      |
| fn astring (input:&str)                       {..} | s    | s   : astring  | astring "Hello World!"        |
| fn greeting (s1: &str, s2:&str)               {..} | ss   | ss  : greeting | greeting hello "Mr. Robinson" |
| fn send (port:&str, baud:u32, data:&[u8])     {..} | sDh  | sDh : send     | send COM2 115200 56EFA23C     |



## Example Command Configuration: `uRustShell\src\usercode\src\commands.cfg`

    v     : crate::uc::init,
    bD    : crate::uc::read,
    sQB   : crate::uc::write,
    t     : crate::uc::led,
    s     : crate::uc::astring
            crate::uc::bstring
            crate::uc::cstring,
    ss    : crate::uc::greeting,
    sDh   : crate::uc::send_uart,

**Note:** As shown above, multiple commands can share a common rule (e.g., the commands `astring`, `bstring`, and `cstring`).

## Running examples

#### Listing registered commands and other information with `###`

    > ###

    ⚡ Commands:
     astring : s
     bstring : s
     cstring : s
    greeting : ss
        init : v
         led : t
        read : bD
        send : sDh
       write : sQB

    ⚡ Shortcuts:
    ### : list all
    ##  : list cmds
    #q  : exit
    #h  : list history
    #c  : clear history
    #N  : exec from history at index N

    ⚡ User shortcuts:
    ++ | +l | +m | +? | +~ | .. | .z | .k | -. | -t | -u | -w

    📝 Arg types:
    B:u8   | W:u16  | D:u32 | Q:u64 | X:u128 | Z:usize | F:f64
    b:i8   | w:i16  | d:i32 | q:i64 | x:i128 | z:isize | f:f32
    v:void | c:char | s:str | t:bool | h:hexstr


#### Running commangs with expected arguments

    >
    >
    >
    > led 0
    led | OFF
    ✅ Success: led 0
    >
    > led 1
    led | ON
    ✅ Success: led 1
    >
    > init
    init | no-args
    ✅ Success: init
    > greeting hello "people around"
    greeting | [hello] : [people around]
    ✅ Success: greeting hello "people around"
    >
    > read 0 1024
    read | descriptor: 0, bytes:1024
    ✅ Success: read 0 1024
    > send COM3 9600 11223344eeffaa56
    send | port: COM3 baudrate: 9600, data:[17, 34, 51, 68, 238, 255, 170, 86]
    ✅ Success: send COM3 9600 11223344eeffaa56
    >

#### Running commands with wrong parameters

    >
    > led 2
    ❌ Error: BadBool for line 'led 2'
    >
    > astring hello world
    ❌ Error: WrongArity { expected: 1 } for line 'astring hello world'
    > init port
    ❌ Error: WrongArity { expected: 0 } for line 'init port'
    > astring
    ❌ Error: WrongArity { expected: 1 } for line 'astring '
    > read 256 1024
    ❌ Error: BadSigned for line 'read 256 1024'
    > hello
    ❌ Error: UnknownFunction for line 'hello'
    >

#### Listing history

    > #h
      0 : led 0
      1 : led 1
      2 : init
      3 : greeting hello "people around"
      4 : read 0 1024
      5 : send COM3 9600 11223344eeffaa56
      6 : led 5
      7 : read -5 2048
      8 : led 2
      9 : astring hello world
     10 : init port
     11 : astring
     12 : read 256 1024
    📈 Left entries/bytes: 3/95
    >

#### Running commangs from history (with the index from the list)

    >
    > #7
    read | descriptor: -5, bytes:2048
    ✅ Success: read -5 2048
    > #3
    greeting | [hello] : [people around]
    ✅ Success: greeting hello "people around"
    > #1
    led | ON
    ✅ Success: led 1
    > #0
    led | OFF
    ✅ Success: led 0
    >

#### Cleaning history

    > #c
    🧹 History cleared
    > #h
    ⚠️ History is empty
    > #0
    ⚠️ No history entry at index 0
    >

## Shortcuts

### Shell shortcuts (internal handling)

    ### : list all, the commands and info
    ##  : list commands only
    #h  : list history
    #c  : clear history
    #N  : exec from history at index N
    #q  : exit

### User shortcuts

Beside the commands, the shell supports also shortcuts (groups of symbols) with associated functions

#### Shortcuts Configuration: `uRustShell\src\usercode\src\shortcuts.cfg`

Shortcuts are organized into **major/minor groups**, such as `++`, `+l`, or `+m` (see examples below).

* For instance, typing `++` will execute the function `shortcut_plus_plus`.
* If extra data is appended after the shortcut (e.g. `++data`), that data is passed as an argument to the function.
* It is the function’s responsibility to handle this input — for example, to reject the call if no arguments are expected or to parse and process the provided data.


#### Configuration example:

    + : { + : crate::us::shortcut_plus_plus,
          l : crate::us::shortcut_plus_l,
          m : crate::us::shortcut_plus_m,
          ? : crate::us::shortcut_plus_question_mark,
          ~ : crate::us::shortcut_plus_tilde
        },

    . : { . : crate::us::shortcut_dot_dot,
          z : crate::us::shortcut_dot_z,
          k : crate::us::shortcut_dot_k
        },

    - : { . : crate::us::shortcut_minus_dot,
          t : crate::us::shortcut_minus_t,
          u : crate::us::shortcut_minus_u,
          w : crate::us::shortcut_minus_w
        },

The list of available shortcuts can be displayed using the `###` command.

    ⚡ Shortcuts:
    ++ | +l | +m | +? | +~ | .. | .z | .k | -. | -t | -u | -w

The shortcut implementations are defined in `uRustShell\src\usercode\src\shortcuts.rs`.

#### ### Examples of Calling Shortcuts

    > ++
    Executing ++ with param: ''
    ✅ Success: ++
    > ++hello
    Executing ++ with param: 'hello'
    ✅ Success: ++hello
    > ++ aa bb cc
    Executing ++ with param: 'aa bb cc'
    ✅ Success: ++ aa bb cc
    > .z
    Executing .z with param: ''
    ✅ Success: .z
    > .z88
    Executing .z with param: '88'
    ✅ Success: .z88
    >

## Local configuration

This file, `uRustShell\src\shell_config\src\lib.rs`, contains the shell configuration.

    pub const PROMPT: &str                  = "> ";
    pub const INPUT_MAX_LEN :usize          = 128;
    pub const HISTORY_TOTAL_CAPACITY :usize = 256;
    pub const HISTORY_MAX_ENTRIES :usize    = 16;
    pub const MAX_HEXSTR_LEN :usize         = 64;

To prevent users from entering more data than configured, keyboard input is automatically blocked once the limit is reached.
If you require additional input capacity, you can increase it in the configuration shown above.

## Building and executing

* Clone the repository locally.
* Assuming that the Rust build environment is already installed, simply run `cargo run -p demo_app`.
* This command will build and execute the test application using the provided test commands.
* Replace these test commands with your own to obtain a fully functional, shell-based application tailored to your needs.
* In principle, you should only need to modify the code in the `usercode` folder.

## Template for Quick Start

An alternative approach is to extract the `urShell.zip` file from the `template` folder in the repository to your local machine and build it using `cargo run`. This template uses pre-published components from `crates.io`, making it the fastest way to integrate the shell into your project.