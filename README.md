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
| `0b1000` | LT  |
| `0b1001` | LTE |
| `0b1010` | EQ  |
| `0b1011` | NE  |


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

**Memory**

- `opcode`: 0b01_0 (load) / 0b01_1 (store)
- `dst`: load → destination register, store → upper offset
- `src1`: load → upper offset, store → source value register
- `src2`: load/store → address register
- `fn`: lower offset

0b0_0_ = word load
0b0_1_ = byte load

Offset is a 16-bit signed offset from `addr`: ±2¹⁵ (±32768 bytes).


**Interrupt**
- `opcode`: `0x19`

| fn      | use                                   |
|---------|---------------------------------------|
| `0b000` | Trigger interrupt with code in `src1` |
| `0b001` | Get current interrupt code            |
| `0b010` | Set interrupt vector to `src1`        |
| `0b011` | Interrupt return (`iret`)             |
| `0b100` | Get return address                    |
| `0b101` | Set return address                    |

Interrupts auto-queue while a handler is running — no nesting, next queued interrupt fires only after `iret`.\
Queue depth is 4, any interrupts fired after that will be dropped

return address is stored internally in interrupt handler and is mutated by set return address

---

### I-type
Immediate instructions.

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
| `0b1110` | SUB   |
| `0b1111` | SHL   |

**Branch**

- `dst`: src1
- `src1`: src2
- `imm`: jump offset

| opcode     | branch |
|------------|--------|
| `0b010000` | B      |
| `0b010001` | BGT    |
| `0b010010` | BLS    |
| `0b010011` | BEQ    |
| `0b010100` | BNE    |
| `0b010101` | BA     |
| `0b010110` | BB     |

jump offset of 16 bits signed shifted left by 2 gives 2^15 << 2 gives around 131KB jump range

---

### J-type
Jump instructions.

**Jump**\
j and jal
- `opcode`: `0x17-0x18`
- `addr`: jump offset

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
| `r8-r15`  | Temporaries         |
| `r16-r23` | Callee-saved        |
| `r24-r25` | Temporaries         |
| `r26-r27` | Kernel              |
| `r28`     | Global pointer      |
| `r29`     | Stack pointer       |
| `r30`     | Frame pointer       |
| `r31`     | Return address      |


### Aliases

| Register  | Alias   |
|-----------|---------|
| `r0`      | `zero`  |
| `r1`      | `at`    |
| `r2-r3`   | `v0-v1` |
| `r4-r7`   | `a0-a3` |
| `r8-r15`  | `t0-t7` |
| `r16-r23` | `s0-s7` |
| `r24-r25` | `t8-t9` |
| `r26-r27` | `k0-k1` |
| `r28`     | `gp`    |
| `r29`     | `sp`    |
| `r30`     | `fp`    |
| `r31`     | `ra`    |


# ASSEMBLER

# COMPILER

## Adding new syntax checklist
1. Add lexer compatibility if needed (ex. new symbols used)
2. Make Ast node 
3. Add parser functionality 
4. Update type and name resolver to traverse the nodes 
5. Update semantic checker if we use that in the future
6. Add to code generator

# KERNEL

# FILESYSTEM