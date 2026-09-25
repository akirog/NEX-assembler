# NEX CPU Documentation

## Table of Contents
- [CPU](#cpu)
- [ASSEMBLER](#assembler)
- [COMPILER](#compiler)
- [KERNEL](#kernel)
- [FILESYSTEM](#filesystem)

---

# CPU

## ISA

### Instruction types

| Type   | Layout (bits)                                                                   |
|--------|---------------------------------------------------------------------------------|
| R-type | `opcode[31:26]` \| `src1[25:21]` \| `src2[20:16]` \| `dst[15:11]` \| `fn[10:0]` |
| I-type | `opcode[31:26]` \| `dst[25:21]` \| `src1[20:16]` \| `imm[15:0]`                 |
| J-type | `opcode[31:26]` \| `addr[25:0]`                                                 |

### R-type
Register instructions.

**Arithmetic**

- `opcode`: 0
- `dst`: destination register
- `src1`: source register 1
- `src2`: source register 2
- `fn`: arithmetic op

| fn       | op  |
|----------|-----|
| `0b0000` | ADD |
| `0b0001` | SUB |
| `0b0010` | AND |
| `0b0011` | OR  |
| `0b0100` | XOR |
| `0b0101` | NEG |
| `0b0110` | SHL |
| `0b0111` | SHR |
| `0b1000` | MUL |
| `0b1001` | DIV |
| `0b1010` | MOD |
| `0b1011` | LT  |
| `0b1100` | LTE |
| `0b1101` | EQ  |
| `0b1110` | NE  |

mov instructions can be emulated with add det, src, $zero

**Jump register**
jr and jrl
- `opcode`: `0x01-0x02`
- `src1`: jump destination register


**IO**
- `opcode`: `0x03`
- `fn`: io port

| io port | device    |
|---------|-----------|
| `0b000` | Halt      |
| `0b001` | Keyboard  |
| `0b010` | Screen    |
| `0b011` | Time      |
| `0b100` | ROM       |
| `0b101` | SSD read  |
| `0b110` | SSD write |

For use of these io devices refer to turing complete manuals\
device output is always sent to dst\
device input 1 is always src1\
device input 2 is always src2

halt has 5 halt codes, with value in src1 a halt is chosen\
0: halt\
1: pause\
2: err code 1\
3: err code 2\
4: err code 3


**interrupt**

- `opcode`: `0x19`
- `src2`: null

| `fn`    | `instr`   | Use                                   |
|---------|-----------|---------------------------------------|
| `0b000` | `trigint` | Trigger interrupt with code in `src1` |
| `0b001` | `getint`  | Get current interrupt code in `dst`   |
| `0b010` | `setvec`  | Set interrupt vector to `src1`        |
| `0b011` | `iret`    | Interrupt return                      |
| `0b100` | `getra`   | Get return address in `dst`           |
| `0b101` | `setra`   | Set return address to `src1`          |

### Internals

The interrupt handler has 3 internal registers:

- **`vec`** - address the handler jumps to on interrupt
- **`code`** - interrupt code for the current interrupt
- **`addr`** - return address for `iret`

**On interrupt (`int`):**
1. Store `ip + 4` into `addr`
2. Store the interrupt code into `code`
3. Jump to `vec`

**On `iret`:**
- Jump to `addr`


### Hardware interrupts
**Keyboard**\
int code: 0x10

### Queueing

- Interrupts auto-queue while a handler is running — **no nesting**; the next queued interrupt only fires after `iret`.
- Queue depth is **4** — any interrupts fired beyond that are dropped.
- The return address is stored internally in the interrupt handler and can be mutated via **Set return address** (`0b101`).
---

### I-type
Immediate instructions.


**Memory**

- `opcode`: 0b01_0 (load) / 0b01_1 (store)
- `dst`: load → destination register, store → source value reg
- `src1`: load/store → address register
- `imm`: offset

0b0_0_ = word load
0b0_1_ = byte load

Offset is a 16-bit signed offset from `addr`: ±2¹⁵ (±32768 bytes).


**Arithmetic**

- `dst`: destination register
- `src1`: source register 1
- `imm`: immediate value (also acts as operand 2, e.g. for SUB)

| opcode   | op    |
|----------|-------|
| `0b1000` | ADD   |
| `0b1001` | ADDHI |
| `0b1010` | OR    |
| `0b1011` | SUB   |
| `0b1100` | SHL   |
| `0b1101` | SHR   |
| `0b1110` | MUL   |
| `0b1111` | DIV   |

**Branch**

- `dst`: src1
- `src1`: src2
- `imm`: jump offset

| opcode     | branch           |
|------------|------------------|
| `0b010000` | uncond           |
| `0b010001` | greater          |
| `0b010010` | less             |
| `0b010011` | equal            |
| `0b010100` | not equal        |
| `0b010101` | above (unsigned) |
| `0b010110` | below (unsigned) |

The jumps offset of 16 bits signed shifted left by 2 gives 2^15 << 2 gives around 131KB jump range.\
Branches always put curr address in ra reg.

---

### J-type
Jump instructions.

**Jump**\
j and jal
- `opcode`: `0x17-0x18`
- `addr`: jump offset

The jump offset of 16 bits signed shifted left by 2

---

### Other

**NO-OP**

- `opcode`: 0
- `dst`: 0
- `fn`: 0

functionally just an add into ZERO reg so it is ignored functioning as a no op

---

## Registers

| Register  | Use                 |
|-----------|---------------------|
| `r0`      | Zero register       |
| `r1`      | Assembler temporary |
| `r2-r3`   | Return values       |
| `r4-r7`   | Function args       |
| `r8-r25`  | Temporaries         |
| `r26-r27` | Kernel              |
| `r28`     | Global pointer      |
| `r29`     | Stack pointer       |
| `r30`     | Base pointer        |
| `r31`     | Return address      |


### Aliases

| Register  | Alias    |
|-----------|----------|
| `r0`      | `zero`   |
| `r1`      | `at`     |
| `r2-r3`   | `v0-v1`  |
| `r4-r7`   | `a0-a3`  |
| `r8-r25`  | `t0-t17` |
| `r26-r27` | `k0-k1`  |
| `r28`     | `gp`     |
| `r29`     | `sp`     |
| `r30`     | `bp`     |
| `r31`     | `ra`     |


# ASSEMBLER

## Assembly instructions

### R-type — `op dst, src1, src2`

**Arithmetic** — opcode `0b000000`, selected by `fn`

| mnemonic | fn       | operands        |
|----------|----------|-----------------|
| add      | `0b0000` | dst, src1, src2 |
| sub      | `0b0001` | dst, src1, src2 |
| and      | `0b0010` | dst, src1, src2 |
| or       | `0b0011` | dst, src1, src2 |
| xor      | `0b0100` | dst, src1, src2 |
| neg      | `0b0101` | dst, src1       |
| shl      | `0b0110` | dst, src1, src2 |
| shr      | `0b0111` | dst, src1, src2 |
| mul      | `0b1000` | dst, src1, src2 |
| div      | `0b1001` | dst, src1, src2 |
| mod      | `0b1010` | dst, src1, src2 |
| lt       | `0b1011` | dst, src1, src2 |
| lte      | `0b1100` | dst, src1, src2 |
| eq       | `0b1101` | dst, src1, src2 |
| ne       | `0b1110` | dst, src1, src2 |

`mov dst, src` → pseudo → `add dst, src, r0`
`nop` → pseudo → all-zero word

**Jump register** — `jr src1` opcode `0b000001` · `jrl src1` opcode `0b000010` (link addr → `r31`/`ra`)

**IO** — opcode `0b000011`, operand `io dst, src1, src2, port` — `port` may be a literal or a named device (`halt`, `keyboard`, `screen`, `time`, `rom`, `ssdread`, `ssdwrite`) resolved by the assembler to its `fn` value

**Interrupt** — opcode `0b011001`, selected by `fn`

| mnemonic | fn      | operands |
|----------|---------|----------|
| trigint  | `0b000` | src1     |
| getint   | `0b001` | dst      |
| setvec   | `0b010` | src1     |
| iret     | `0b011` | (none)   |
| getra    | `0b100` | dst      |
| setra    | `0b101` | src1     |

### I-type

**Arithmetic** — `op dst, src1, imm`

| mnemonic | opcode     | 
|----------|------------|
| addi     | `0b001000` |
| addhi    | `0b001001` |
| ori      | `0b001010` |
| subi     | `0b001011` |
| shli     | `0b001100` |
| shri     | `0b001101` |
| muli     | `0b001110` |
| divi     | `0b001111` |

`mov dst, imm` → pseudo →
```
addhi dst, r0, imm.upper16
addi  dst, dst, imm.lower16
```

**Memory** — `load[b] dst, [src1 + imm]` / `store[b] [src1 + imm], src`

| mnemonic | opcode     |
|----------|------------|
| load     | `0b000100` |
| store    | `0b000101` |
| loadb    | `0b000110` |
| storeb   | `0b000111` |

**Branch** — `b<cond> src1, src2, offset`

offset can be a label or imm

| mnemonic | opcode     |
|----------|------------|
| b        | `0b010000` |
| bgt      | `0b010001` |
| blt      | `0b010010` |
| beq      | `0b010011` |
| bne      | `0b010100` |
| bgtu     | `0b010101` |
| bltu     | `0b010110` |

### J-type — `op addr`

| mnemonic | opcode                              |
|----------|-------------------------------------|
| j        | `0b010111`                          |
| jal      | `0b011000` (link addr → `r31`/`ra`) |


## Sections

```
section .text
section .data
```

- `section <name>` switches the assembler's current section. Name may be written with or without a leading dot (`.data` / `data` both fine — matches old assembler's `lstrip(".")`).
- `.text` holds instructions, `.data` holds raw bytes (from `db`).
- Only `.text` and `.data` exist for now — no `.bss`/`.rodata` etc. unless you want them.
- Sections can repeat (`section .text` / `.data` / `.text` again) and just keep appending to whichever buffer is active, in file order.

## Labels

```
label_name:
loop:
    add t0, t0, 1
    b loop, ...
```

- A label is any identifier followed by `:` on its own line.
- Its address depends on which section it's declared in:
  - In `.text` → absolute instruction address (byte offset from `base_addr`).
  - In `.data` → offset from the start of the data section, resolved to a real address only after the final instruction address is known (data section is placed right after code, same as v1).
- Labels are collected in a first pass so forward references work in both branches/jumps and `db`.
- Referencing a label as an operand gives you its address as an immediate — usage differs by context: raw address for `mov`/`load`/`store`, or `(target - current_addr) >> 2` for branch/jump offsets, matching the encoding's shifted-offset scheme.

## `db`/`dw` — define bytes / words

```
db 1, 2, 3
db "hello", 0
db 0 times 64
msg: db "NEX OS", 0
dw 0, -1 ; 32 bit value
```

- Only valid inside `.data`.
- Comma-separated list of items, each one of:
  - **String literal** (`"..."`) → each character emitted as one byte.
  - **Immediate/expression** → evaluated and emitted as a single byte.
  - **`<value> times <count>`** → emits `value` repeated `count` times (both must be resolvable immediates).
- All byte-producing — if you want 16/32-bit packed constants (like v1's `left + right` 4-byte case) let me know, since that was a special case in the old assembler I didn't carry over here on purpose; flagging rather than assuming you still want it.


# COMPILER

## Pipeline

`.nex` source goes through, in order:

1. **Preprocessor** (`preprocessor.py`) — textual, line-based:
   - `#include <path>` — splices the file's lines in directly (recursive; re-runs preprocessing after each include).
   - `#baseaddr <expr>` — sets the load/link address for the generated binary (`eval`'d, so `0x20000` etc. works).
   - `#define <find> <replace>` — dumb find/replace across the whole file, no macro arguments.
2. **Lexer** (`lexer.py`) — regex-based tokenizer. `asm ( ... )` blocks are hand-parsed by bracket-matching *before* the regex pass, so arbitrary raw assembly lines can live inside them.
3. **Parser** (`parser.py`) — recursive-descent, builds the AST (`ast_nodes.py`).
4. **Semantic analyzer** (`semantic_analyzer.py` + `semantic_analyzer_helpers/`) — builds the type table (structs get registered here), builds the scope/frame stack, resolves names and types.
5. **Code generator** (`code_generator.py`) — walks the typed AST and emits NEX assembly (`.nesm`-style text).
6. That assembly is fed straight into the `Assembler` to produce the final flat binary.

## Adding new syntax checklist
1. Add lexer compatibility if needed (ex. new symbols used)
2. Make Ast node 
3. Add parser functionality 
4. Update type and name resolver to traverse the nodes 
5. Update semantic checker if we use that in the future
6. Add to code generator



# NEX LANGUAGE

C-like syntax compiled down to NEX assembly.

## Types
`int` (4 bytes), `char` (1), `bool` (1), `uint8` (1), `void`, plus user-defined `struct`s. Pointers via `*`, arrays via `name[size]`. Casts: `(type)expr`.

## Declarations
```
int x = 5;
char msg[3];
struct Point { int x; int y; };
```

## Functions
```
ret_type name(type arg1, type arg2) {
    ...
    return value;
}
```
Arguments arrive in `a0`–`a3`; the compiler emits the prologue (`push ra`, `push bp`, `bp = sp`, allocate frame) and epilogue automatically.

## Control flow
`if` / `else`, `while`, `for`, `break`, `continue`, `return`.

## Built-ins
`sizeof(identifier)` — resolves to a compile-time `int` constant, the identifier's type size in bytes.

## Inline assembly
```
asm (
    "mov r0, some_label"
    "io r0, r0, r0, 1 ; comment"
);
```
Each line is passed through to the assembler almost verbatim (surrounding quotes stripped, trailing `//` comments stripped). Useful for anything the language doesn't expose yet (setting the interrupt vector, raw I/O ports, etc.) — see `kernel.nex` for examples.

## Preprocessor
See [COMPILER](#compiler) — `#include`, `#baseaddr`, `#define` are available inside `.nex` files too.


# KERNEL

## Interrupts
On an interrupt, the interrupt handler puts its stack and base pointer into k0 and k1,\
then it uses k0 and k1 to push all registers to the stack, then it puts bp and sp to k0 and k1.\
k0 is used as bp\
k1 is used as sp\
On an iret the interrupt handler puts bp and sp into k0 and k1, then use k0 and k1 to load\
back all registers from the stack, before returning with iret.


# FILESYSTEM

## Layout

`filesystem/fs_builder.py` builds `filesystem/fs.bin` from `filesystem/conf.conf`. The image has 3 fixed regions:

| Region     | Address   | Size            | Contents                       |
|------------|-----------|-----------------|--------------------------------|
| Kernel     | `0x00`    | up to `0x10000` | Raw kernel binary              |
| File table | `0x10000` | `0x1000`        | File count (4 bytes) + headers |
| Binaries   | `0x11000` | rest of image   | Concatenated file contents     |

Each file table header is 32 bytes:

| Offset        | Size | Field                     |
|---------------|------|---------------------------|
| `0x00`–`0x17` | 24   | File name (null-padded)   |
| `0x18`–`0x1B` | 4    | File size (bytes)         |
| `0x1C`–`0x1F` | 4    | Binary address (absolute) |

## `conf.conf` format

```
[KERNEL]
"KERNEL/kernel.nex"

[PROGRAMS]
shell = "KERNEL/shell.nex"
hello_world = "programs/print.bin"

[FILES]
text-file = "files/text_file.txt"
```

- `[KERNEL]` takes a single path (no `name =` prefix).
- `[PROGRAMS]` entries are compiled and get a file-table entry.
- `[FILES]` entries are copied byte-for-byte into the image untouched (no compilation) — for arbitrary data files.

## Auto-compilation

Any `.nex` path under `[KERNEL]` or `[PROGRAMS]` is compiled automatically — `fs_builder.py` calls `compiler.compiler.main()` on it before building the image, and uses the resulting `.bin`. `.bin` paths are used as-is. This means you never hand-compile the kernel or programs before building the filesystem; just point `conf.conf` at the `.nex` source.

## Building

```
python filesystem/fs_builder.py [--verbose]
```

**Known limit:** the file table is fixed at `0x1000` bytes, so with 32-byte headers it overflows past roughly 127 files.
