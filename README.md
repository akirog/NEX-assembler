# CPU

## Instructions

### IO

`io [dst], [src1], [src2], [port]`

I/O instruction. Port is up to one byte, but only 3 bits are currently used:

| Port    | Function     |
|---------|--------------|
| `0b000` | HALT         |
| `0b001` | SET INT PTR  |
| `0b010` | KEYBOARD     |
| `0b011` | SCREEN       |
| `0b100` | TIME         |
| `0b101` | ROM          |
| `0b110` | READ SSD     |
| `0b111` | WRITE SSD    |

**HALT**\
Halts the CPU's instruction counter. Takes no inputs/outputs. Does not fully stop CPU execution execution can still be manually resumed.

**SET INT PTR**\
Takes one input at `src1`. Sets the CPU's interrupt pointer to the value stored in `src1`.

**KEYBOARD**\
Reads the current key in the key buffer. Outputs to `dst`.

**SCREEN**\
Takes `src1` as command and `src2` as value. Refer to Turing Complete documentation for screen commands and values.

**TIME**\
Gets current nanoseconds (since some unspecified reference date) into `dst`. If `src1`'s first bit is set, returns the top 32 bits; otherwise the bottom 32.

**ROM**\
Reads address `src1` into `dst`.

---

# ASSEMBLER

# COMPILER

# KERNEL

# FILESYSTEM