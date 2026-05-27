# rscl

A Language Server Protocol (LSP) implementation for **Siemens TIA Portal SCL** (Structured Control Language), written in Rust.

## Features

- **Syntax diagnostics** — Reports multiple parse errors per file with precise locations
- **Keyword completions** — Context-aware completions for SCL keywords and types
- **PLC address recognition** — Tokenizes and validates PLC memory addresses (`IW0`, `MW10`, `Q0.1`, `DB1.DBW0`)
- **Resilient parser** — Continues parsing after errors to provide maximum feedback
- **Editor-agnostic** — Works with any LSP-compatible editor via stdio

### Supported SCL Constructs

- Block declarations: `FUNCTION`, `FUNCTION_BLOCK`, `DATA_BLOCK`, `ORGANIZATION_BLOCK`
- Variable sections: `VAR`, `VAR_INPUT`, `VAR_OUTPUT`, `VAR_IN_OUT`, `VAR_TEMP`, `CONST`
- Data types: `BOOL`, `BYTE`, `WORD`, `DWORD`, `INT`, `DINT`, `REAL`, `CHAR`, `STRING`, `TIME`, `DATE`, `ARRAY`, `STRUCT`
- Control flow: `IF/THEN/ELSIF/ELSE/END_IF`, `FOR/TO/BY/DO/END_FOR`, `WHILE/DO/END_WHILE`, `REPEAT/UNTIL/END_REPEAT`, `CASE/OF/END_CASE`
- Expressions: arithmetic, logical, comparisons, function calls, array/struct access

## Installation

### Option 1: Build from source (automatic)

Requires Rust toolchain. Installs the binary to `~/.cargo/bin/` which is already in your PATH.

```bash
git clone https://github.com/<user>/rscl.git
cd rscl
make install
```

Or directly via cargo:

```bash
cargo install --git https://github.com/<user>/rscl.git
```

### Option 2: Download from GitHub Releases (manual)

1. Download the binary for your platform from [Releases](https://github.com/<user>/rscl/releases)
2. Make it executable and move it to a directory in your PATH:

```bash
chmod +x rscl
mv rscl ~/.local/bin/
```

3. Ensure `~/.local/bin` is in your PATH. Add to `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Neovim Setup

1. Install `rscl` using one of the methods above.

2. Add the following to your Neovim config (or source `editors/neovim/scl.lua`):

```lua
-- Source the SCL plugin (provides LSP + syntax highlighting)
-- Adjust the path to where you cloned rscl
vim.cmd('source /path/to/rscl/editors/neovim/scl.lua')
```

Or add manually:

```lua
-- Add syntax highlighting
vim.opt.runtimepath:append("/path/to/rscl/editors/neovim")

-- Register .scl filetype
vim.filetype.add({
  extension = {
    scl = "scl",
  },
})

-- Configure the LSP server
vim.api.nvim_create_autocmd("FileType", {
  pattern = "scl",
  callback = function()
    vim.lsp.start({
      name = "rscl",
      cmd = { "rscl" },
      root_dir = vim.fs.dirname(vim.fs.find({ ".git" }, { upward = true })[1]) or vim.fn.getcwd(),
      capabilities = vim.lsp.protocol.make_client_capabilities(),
    })
  end,
})
```

3. Open any `.scl` file — diagnostics, completions, and syntax highlighting will activate automatically.

## Usage Examples

### Syntax Diagnostics

The LSP reports errors with precise locations. For example, this file has a missing type after the colon:

```scl
FUNCTION_BLOCK MyFB
VAR
    x : ;       // ← Error: "expected type specification" at the semicolon
    y : INT;    // ← OK, parser recovers and continues
END_VAR
BEGIN
    y := x + ;  // ← Error: "expected expression"
END_FUNCTION_BLOCK
```

The server reports **both** errors in a single pass — you don't have to fix one to see the next.

### Keyword Completions

Completions are context-aware. Triggering completion (e.g. `<C-x><C-o>` or your configured key) gives different results depending on where your cursor is:

**At top level** (outside any block):

```
FUNCTION_BLOCK, FUNCTION, DATA_BLOCK, ORGANIZATION_BLOCK
```

**Inside a block** (before BEGIN):

```
VAR, VAR_INPUT, VAR_OUTPUT, VAR_IN_OUT, VAR_TEMP, CONST, BEGIN
```

**Inside a VAR section** (after the colon):

```
BOOL, BYTE, WORD, DWORD, INT, DINT, REAL, CHAR, STRING, TIME, DATE, ARRAY, STRUCT, ...
```

**Inside the body** (after BEGIN):

```
IF, FOR, WHILE, REPEAT, CASE, RETURN, TRUE, FALSE, AND, OR, NOT, MOD, DIV, ...
```

### PLC Address Recognition

The lexer recognizes standard Siemens PLC memory addresses:

```scl
FUNCTION_BLOCK IO_Example
VAR
    sensor_val : INT;
END_VAR
BEGIN
    sensor_val := IW0;       // Input word 0
    MW10 := sensor_val * 2;  // Memory word 10
    Q0.1 := TRUE;            // Output bit 0.1
    DB1.DBW0 := 100;         // Data block 1, word 0
END_FUNCTION_BLOCK
```

Invalid addresses or unknown characters produce error diagnostics without stopping the parser.

### Full Example — PID Controller

```scl
FUNCTION_BLOCK PID_Controller
VAR_INPUT
    setpoint : REAL;
    process_value : REAL;
    kp : REAL;
    ki : REAL;
    kd : REAL;
    dt : REAL;
END_VAR
VAR_OUTPUT
    output : REAL;
END_VAR
VAR
    prev_error : REAL;
    integral : REAL;
END_VAR
BEGIN
    error := setpoint - process_value;
    integral := integral + error * dt;
    derivative := (error - prev_error) / dt;
    output := kp * error + ki * integral + kd * derivative;
    prev_error := error;

    IF output > 100.0 THEN
        output := 100.0;
    ELSIF output < 0.0 THEN
        output := 0.0;
    END_IF;
END_FUNCTION_BLOCK
```

## License

MIT
