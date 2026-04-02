import operator
import struct
import argparse
from dataclasses import dataclass
from typing import Any


@dataclass
class Instruction:
    opcode: int = 000000    # [31-26]
    dest: int | None = None # [25-22]
    src1: int | None = None # [21-18]
    src2: int | None = None # [17-14]

    # Imm used for imm type alu instructions, 18 bits signed
    alu_imm: int | None = None  # [17-0]

    # Imm used for relative jumps, 26 bits signed
    jmp_imm: int | None = None  # [25-0]

    # Imm used for memory offsets, 14 bits signed
    mem_imm: int | None = None  # [13-0]

    # ALU operation for R type alu instructions, none for imm type instructions
    alu_op: int | None = None   # [13-10]

    # Extra flags for certain instructions
    flags: int | None = None    # [can fill up  everything ig]

    # Address of this instruction
    address: int = 0

    debug_original_text: str = ""



alu_ops = {
    "add":  0b0000,
    "sub":  0b0001,
    "mul":  0b0010,
    "div":  0b0011,
    "mod":  0b0100,
    "and":  0b0101,
    "or":   0b0110,
    "xor":  0b0111,
    "not":  0b1000,
    "neg":  0b1001,
    "shl":  0b1010,
    "shr":  0b1011,
    "sar":  0b1100,
    "mov":  0b1101,
    "cmp":  0b1111,
}

alu_imm_ops = {
    "add":  0b0001,
    "sub":  0b0010,
    "mul":  0b0011,
    "div":  0b0100,
    "mod":  0b0101,
    "and":  0b0110,
    "or":   0b0111,
    "xor":  0b1000,
    "shl":  0b1011,
    "shr":  0b1100,
    "sar":  0b1101,
    "mov":  0b1110,
    "movh": 0b1111,
    "cmp":  0b1001,
}

single_op_alu_ops = [
    "not",
    "neg"
]

jmp_ops = {
    "jmp":  0b010000,
    "jz":   0b010001,
    "je":   0b010001,
    "jnz":  0b010010,
    "jne":  0b010010,
    "jg":   0b010011,
    "jl":   0b010100,
    "jge":  0b010101,
    "jle":  0b010110,
    "jb":   0b010111,
    "ja":   0b011000,
    "jbe":  0b011001,
    "jae":  0b011010,
    "jr":   0b011011,
}

mem_ops = {
    "load": 0b100000,
    "stor": 0b100001,
    "store": 0b100001,
}


reg_names = {
    "r0":  0,
    "r1":  1,
    "r2":  2,
    "r3":  3,
    "r4":  4,
    "r5":  5,
    "r6":  6,
    "r7":  7,
    "r8":  8,
    "r9":  9,
    "r10": 10,
    "r11": 11,
    "r12": 12,
    "r13": 13,
    "r14": 14,
    "r15": 15,
    "lp": 13,
    "bp": 14,
    "sp": 15,
    "ra": 0,
    "a0": 1,
    "a1": 2,
    "a2": 3,
    "a3": 4,
    "a4": 5,
}


misc_opcodes = {
    "io": 0b100010
}


macro_opcodes = {
    "call": 5,  # sub sp, mov lp, movh lp, store lp, jmp
    "ret":  2,  # load lp, jmp
    "push": 2,  # sub sp, store
    "pop":  2,  # load, add sp
}


python_operations_map: dict[str, Any] = {
    "+": operator.add,
}

INTERRUPT_OPCODE = 0b101000
INT_RET_OPCODE = 0b101001
GETINTCODE_opcode = 0b101010

def get_reg(name: str) -> int:
    if name in reg_names:
        return reg_names[name]
    else:
        raise SyntaxError("Invalid register name: " + name)


def is_reg(reg_name: str) -> bool:
    if reg_name in reg_names:
        return True
    else:
        return False


class Assembler:
    def __init__(self):
        self.base_addr: int = 0 # Where program gets loaded, usually 0 for now but things like os expect to be at high addr
        self.curr_address: int = 0 # Where in bytes current instruction is
        self.data_offset: int = 0 # Where the data section starts
        self.labels: dict[str, int] = {}
        self.input: list[str] = []
        self.instructions: list[Instruction] = []
        self.output: list[int] = []
        self.consts: dict[str, int] = {}

        self.verbose = False

        # Data directives stuff
        self.data_bytes: list[int] = []
        self.curr_section: str = "text"
        self.data_labels: dict[str, int] = {}


    def _get_consts(self, imm_type: str = "alu") -> dict[str, int]:
        consts: dict[str, int] = dict(self.consts)

        # Add labels
        for label_name, label_val in self.labels.items():
            val = label_val
            if imm_type == "jmp":
                # Jmp wants relative instruction addr
                val = (label_val - self.curr_address) // 4

            consts[label_name] = val

        # Add data labels
        for label_name, label_val in self.data_labels.items():
            val = label_val + self.data_offset
            if imm_type == "jmp":
                # Shouldn't really jump to data section but oh well
                val = (label_val+self.data_offset - self.curr_address) // 4

            consts[label_name] = val

        return consts


    def is_imm(self, imm: str, imm_type: str = "alu") -> bool:
        consts = self._get_consts(imm_type)

        try:
            eval(imm, consts)
            return True
        except Exception:
            return False

    def is_label(self, name: str) -> bool:
        if is_reg(name) or self.is_imm(name):
            return False
        else:
            return True

    def get_imm(self, imm: str, imm_type: str = "alu") -> int:
        consts = self._get_consts(imm_type)

        try:
            return int(eval(imm, consts))
        except Exception:
            raise SyntaxError("Invalid immediate: " + imm)

    def get_inc_addr(self) -> int:
        addr = self.curr_address
        self.curr_address += 4 # 4 bytes per instruction
        return addr

    def debug_print(self, text: str = ""):
        if self.verbose:
            print(text)

    def parse_input(self):

        start_idx = 0

        for text in self.input:

            parts = [p.strip(",") for p in text.strip().split(";")[0].split(" ")] or []

            new_parts = []

            for part in parts:
                if part != "":
                    new_parts.append(part)

            parts = new_parts

            if not parts or parts[0] == '' or parts[0].startswith(";"):
                continue

            if parts[0].endswith(":"):
                # Label declaration in first pass
                continue

            elif parts[0] == "section":
                self.curr_section = parts[1].rstrip(":").lstrip(".")
                continue

            elif parts[0] == "const":
                # Const declaration in first pass
                continue

            elif parts[0] in alu_ops or parts[0] in alu_imm_ops:
                self.parse_alu(parts)

            elif parts[0] in jmp_ops:
                self.parse_jmp(parts)

            elif parts[0] in mem_ops:
                self.parse_mem(parts)

            elif parts[0] == "call":
                self.parse_call(parts)

            elif parts[0] == "ret":
                self.parse_ret(parts)

            elif parts[0] == "push":
                self.parse_push(parts)

            elif parts[0] == "pop":
                self.parse_pop(parts)

            elif parts[0] == "io":
                self.parse_io(parts)

            elif parts[0] == "db":
                self.parse_db(text)

            elif parts[0] == "iret":
                self.parse_iret(text)

            elif parts[0] == "int":
                self.parse_int(text)

            elif parts[0] == "getint":
                self.parse_get_int(text)

            else:
                raise SyntaxError("Invalid instruction: " + parts[0])

            for i in range(start_idx, len(self.instructions)):
                self.instructions[i].debug_original_text = self.instructions[i].debug_original_text + text.strip()

            start_idx = len(self.instructions)


    def parse_int(self, text: str):
        """Parses an interrupt formatted as: int <code>"""

        parts = text.strip().split(" ")

        if len(parts) != 2:
            raise SyntaxError("Invalid instruction: " + parts[0])

        code = int(parts[1])
        opcode = INTERRUPT_OPCODE

        instruction = Instruction()
        instruction.opcode = opcode
        instruction.alu_imm = code
        instruction.address = self.get_inc_addr()

        self.instructions.append(instruction)


    def parse_iret(self, text: str):
        """Parses an interrupt return, literally just the opcode"""

        instruction = Instruction()
        instruction.opcode = INT_RET_OPCODE
        instruction.address = self.get_inc_addr()

        self.instructions.append(instruction)


    def parse_get_int(self, text: str):
        """Parses a get interrupt code instruction, formatted as: getint <reg>"""

        parts = text.strip().split(" ")
        if len(parts) != 2:
            raise SyntaxError("Invalid instruction: " + parts[0])

        instruction = Instruction()
        instruction.opcode = GETINTCODE_opcode
        instruction.dest = get_reg(parts[1])
        instruction.address = self.get_inc_addr()

        self.instructions.append(instruction)


    def parse_db(self, text: str, write: bool = True) -> int:
        """Parses a db instr to bytes, returns amount of bytes"""

        content = text.strip().lstrip("db").strip()

        tokens = [t.strip() for t in content.split(",")]

        length = 0

        for i in range(len(tokens)):
            token = tokens[i]

            if token.startswith('"'):
                # string
                inner = token.strip('"')
                for char in inner:
                    if write:
                        self.data_bytes.append(ord(char))
                    length += 1

            elif len(token.split(" ")) == 3:
                if not write:
                    length += 4
                    continue

                parts = token.split(" ")
                operation = parts[1]
                left = parts[0]
                right = parts[2]

                print()

                if operation not in python_operations_map or not self.is_imm(left) or not self.is_imm(right):
                    raise SyntaxError("Uh oh error!")

                operation = python_operations_map.get(operation)

                left = self.get_imm(left)

                if right in self.labels:
                    right = self.labels[right]
                else:
                    right = self.get_imm(right)

                result = operation(left, right)

                for j in range(4):
                    self.data_bytes.append((result >> (8 * j)) & 0xFF)

                length += 4

            elif self.is_imm(token):
                if write:
                    self.data_bytes.append(self.get_imm(token))
                length += 1

            elif "times" in token:
                star = token.index("times")
                value_str = token[0:star].strip()
                count_str = token[star + 5:].strip()
                if self.is_imm(value_str) and self.is_imm(count_str):
                    for r in range(self.get_imm(count_str)):
                        if write:
                            self.data_bytes.append(self.get_imm(value_str))
                        length += 1

        return length


    def parse_io(self, parts: list[str]):
        """Parses an io instruction like io, dst, src1, src2, port"""
        if len(parts) != 5:
            raise SyntaxError("Invalid io instruction: " + ' '.join(parts) + '\n' + "Should be io, dst, src1, src2, port")

        result = Instruction()

        result.opcode = misc_opcodes.get("io")
        result.dest = get_reg(parts[1])
        result.src1 = get_reg(parts[2])
        result.src2 = get_reg(parts[3])

        # Memory imm doesn't touch opcode, dst, src1, or src2 so we use it here, normal imm expects src2 to not be used
        result.mem_imm = self.get_imm(parts[4], "mem")

        result.address = self.get_inc_addr()
        self.instructions.append(result)


    def parse_pop(self, parts: list[str]):
        """Parses a pop instr to a load and sp add"""
        if len(parts) != 2:
            raise SyntaxError("Invalid pop instruction: " + ' '.join(parts) + '\n' + "Should be pop, dst")

        # First load value at sp
        load_instr = Instruction()

        load_instr.opcode = mem_ops.get("load")
        load_instr.dest = get_reg(parts[1])
        load_instr.src1 = get_reg("sp")

        load_instr.address = self.get_inc_addr()
        self.instructions.append(load_instr)
        load_instr.debug_original_text = "load " + parts[1] + ", [sp]"
        load_instr.debug_original_text += " " * (20 - len(load_instr.debug_original_text)) + "; "

        # Then increment sp
        sp_inc = Instruction()

        sp_inc.opcode = alu_imm_ops.get("add")
        sp_inc.dest = get_reg("sp")
        sp_inc.src1 = get_reg("sp")
        sp_inc.alu_imm = 4  # Sub 4 from stack pointer to make space for curr addr

        sp_inc.address = self.get_inc_addr()
        self.instructions.append(sp_inc)
        sp_inc.debug_original_text = "add, sp, sp, 4"
        sp_inc.debug_original_text += " " * (20 - len(sp_inc.debug_original_text)) + "; "



    def parse_push(self, parts: list[str]):
        """Parses a push instr to a sp sub and a store"""
        if len(parts) != 2:
            raise SyntaxError("Invalid push instr: " + parts[0] + '\n' + "Should be push, dst")

        # First decrement sp
        sp_dec = Instruction()

        sp_dec.opcode = alu_imm_ops.get("sub")
        sp_dec.dest = get_reg("sp")
        sp_dec.src1 = get_reg("sp")
        sp_dec.alu_imm = 4  # Sub 4 from stack pointer to make space for curr addr

        sp_dec.address = self.get_inc_addr()
        self.instructions.append(sp_dec)
        sp_dec.debug_original_text = "sub, sp, sp, 4"
        sp_dec.debug_original_text += " " * (20 - len(sp_dec.debug_original_text)) + "; "


        # Then store the value at sp
        store_instr = Instruction()

        store_instr.opcode = mem_ops.get("store")
        store_instr.src1 = get_reg("sp")
        store_instr.src2 = get_reg(parts[1])

        store_instr.address = self.get_inc_addr()
        self.instructions.append(store_instr)
        store_instr.debug_original_text = "store [sp], " + parts[1]
        store_instr.debug_original_text += " " * (20 - len(store_instr.debug_original_text)) + "; "


    def parse_ret(self, parts: list[str]):
        """Parses a ret instruction into a load ret addr and jmp"""
        if len(parts) != 1:
            raise SyntaxError("Invalid ret instruction: " + ' '.join(parts) + '\n' + "Should be ret")

        # First load the ret addr
        load_instr = Instruction()

        load_instr.opcode = mem_ops.get("load")
        load_instr.dest = get_reg("lp")
        load_instr.src1 = get_reg("sp")

        load_instr.address = self.get_inc_addr()
        self.instructions.append(load_instr)
        load_instr.debug_original_text = "load, lp, [sp]"
        load_instr.debug_original_text += " " * (20 - len(load_instr.debug_original_text)) + "; "


        # Jump to ret address
        jump_instr = Instruction()

        jump_instr.opcode = jmp_ops.get("jr")
        jump_instr.src1 = get_reg("lp")

        jump_instr.address = self.get_inc_addr()
        self.instructions.append(jump_instr)
        jump_instr.debug_original_text = "jr, lp"
        jump_instr.debug_original_text += " " * (20 - len(jump_instr.debug_original_text)) + "; "




    def parse_call(self, parts: list[str]):
        """Parses a call instr to a addr push and jmp"""
        if len(parts) != 2:
            raise SyntaxError("Invalid call instruction: " + ' '.join(parts) + '\n' + "Should be call reg/label")

        # First make the sp decrement
        sp_dec = Instruction()

        sp_dec.opcode = alu_imm_ops.get("sub")
        sp_dec.dest = get_reg("sp")
        sp_dec.src1 = get_reg("sp")
        sp_dec.alu_imm = 4 # Sub 4 from stack pointer to make space for curr addr

        sp_dec.address = self.get_inc_addr()
        self.instructions.append(sp_dec)
        sp_dec.debug_original_text = "sub, sp, sp, 4"
        sp_dec.debug_original_text += " "*(20-len(sp_dec.debug_original_text)) + "; "


        # Mov ret addr to lp
        target_addr = self.curr_address + 4*4 # ret addr is after this mov, movh, store and jump

        mov_instr = Instruction()

        mov_instr.opcode = alu_imm_ops.get("mov")
        mov_instr.dest = get_reg("lp")
        mov_instr.alu_imm = target_addr

        mov_instr.address = self.get_inc_addr()
        self.instructions.append(mov_instr)
        mov_instr.debug_original_text = "mov lp, $+16"
        mov_instr.debug_original_text += " " * (20 - len(mov_instr.debug_original_text)) + "; "

        # Mov high
        movh_instr = Instruction()

        movh_instr.opcode = alu_imm_ops.get("movh")
        movh_instr.dest = get_reg("lp")
        movh_instr.src1 = get_reg("lp")
        movh_instr.alu_imm = target_addr >> 16

        movh_instr.address = self.get_inc_addr()
        self.instructions.append(movh_instr)
        movh_instr.debug_original_text = "movh lp, $+12"
        movh_instr.debug_original_text += " " * (20 - len(mov_instr.debug_original_text)) + "; "


        # Store ret addr from lp to sp
        store_instr = Instruction()

        store_instr.opcode = mem_ops.get("store")
        store_instr.src1 = get_reg("sp")
        store_instr.src2 = get_reg("lp")

        store_instr.address = self.get_inc_addr()
        self.instructions.append(store_instr)
        store_instr.debug_original_text = "store [sp], lp"
        store_instr.debug_original_text += " " * (20 - len(store_instr.debug_original_text)) + "; "


        # Jump instruction
        jmp_instr = Instruction()

        # Check if a reg is called
        if is_reg(parts[1]):
            # call to reg
            jmp_instr.opcode = jmp_ops.get("jr")

            jmp_instr.src1 = get_reg(parts[1])

        elif self.is_imm(parts[1]):
            # call to label
            jmp_instr.opcode = jmp_ops.get("jmp")
            jmp_instr.jmp_imm = self.get_imm(parts[1], "jmp")

        else:
            raise SyntaxError("Invalid instruction: " + ' '.join(parts))

        jmp_instr.address = self.get_inc_addr()
        self.instructions.append(jmp_instr)
        jmp_instr.debug_original_text = "jmp " + parts[1]
        jmp_instr.debug_original_text += " " * (20 - len(jmp_instr.debug_original_text)) + "; "


        return


    def parse_mem(self, parts: list[str]):
        """Parses a memory instruction like load r0, 0x1000 or store r3, r1"""
        # can either be:
        # load reg, [reg]
        # store [reg], reg
        # load reg, [reg + something]
        # store [reg + something], reg

        # Store can specify byte to only store one byte

        if len(parts) < 3 or len(parts) > 6:
            raise SyntaxError("Invalid memory instruction: " + ' '.join(parts))

        result: Instruction = Instruction()
        result.opcode = mem_ops.get(parts[0])

        if result.opcode == mem_ops.get("load"):
            # Load
            # Structured as: load dst_reg, [src_reg + offset]

            result.dest = get_reg(parts[1])

            if not parts[2].startswith("["):
                raise SyntaxError("Invalid load address, memory loads should specify address as [reg + offset] for line: " + ' '.join(parts))

            parts[2] = parts[2].lstrip("[")
            parts[-1] = parts[-1].rstrip("]")

            result.src1 = get_reg(parts[2])

            if len(parts) == 5 and parts[3] == "+":
                if self.is_imm(parts[4]):
                    result.mem_imm = self.get_imm(parts[4], "mem")
                else:
                    raise SyntaxError("Invalid load offset: " + ' '.join(parts))

            elif len(parts) == 5 and parts[3] == "-":
                if self.is_imm(parts[4]):
                    result.mem_imm = -self.get_imm(parts[4], "mem")
                else:
                    raise SyntaxError("Invalid load offset: " + ' '.join(parts))

            elif len(parts) == 5:
                raise SyntaxError("Only addition and subtraction supported for mem offsets: " + ' '.join(parts))


        elif result.opcode == mem_ops.get("store"):
            # Store
            # Structured as: stor [dst_reg + offset], src_reg

            parts_offset = 0
            if parts[1] == "byte":
                parts_offset = 1
                result.opcode |= 0b00010

            parts[1+parts_offset] = parts[1+parts_offset].lstrip("[").rstrip("]")
            result.src1 = get_reg(parts[1+parts_offset])


            if len(parts) == 5+parts_offset and parts[2+parts_offset] == "+":
                parts[3+parts_offset] = parts[3+parts_offset].rstrip("]")
                result.mem_imm = self.get_imm(parts[3+parts_offset], "mem")
            elif len(parts) == 5+parts_offset and parts[2+parts_offset] == "-":
                parts[3+parts_offset] = parts[3+parts_offset].rstrip("]")
                result.mem_imm = -self.get_imm(parts[3+parts_offset], "mem")
            elif len(parts) == 5+parts_offset:
                raise SyntaxError("Only addition and subtraction supported for mem offsets: " + ' '.join(parts))

            result.src2 = get_reg(parts[-1])

        result.address = self.get_inc_addr()
        self.instructions.append(result)


    def parse_jmp(self, parts: list[str]):
        """Parses a jump instruction like jmp label or jr r0"""
        if len(parts) != 2:
            raise SyntaxError("Invalid jump instruction: " + ' '.join(parts) + '\n' + "Should be jmp_op label/imm or jr reg")

        result: Instruction = Instruction()

        # Handle r type jmp first, special case
        if parts[0] == "jr":
            result.opcode = jmp_ops.get("jr")

            result.src1 = get_reg(parts[1])

            result.address = self.get_inc_addr()
            self.instructions.append(result)
            return

        else:
            result.opcode = jmp_ops.get(parts[0])

            if self.is_imm(parts[1]):
                result.jmp_imm = self.get_imm(parts[1], "jmp")

        result.address = self.get_inc_addr()
        self.instructions.append(result)



    def parse_alu(self, parts: list[str]):
        """Parses an alu instruction like add"""

        result: Instruction = Instruction()

        result.opcode = 0b000000
        result.dest = get_reg(parts[1])


        # Check mov first
        if parts[0] == "mov":

            if is_reg(parts[2]):
                # Reg to Reg
                result.src2 = get_reg(parts[2])

                # R type opcode
                result.alu_op = alu_ops.get(parts[0])

                if len(parts) == 4:
                    # Expect flags
                    result.flags = eval(parts[3])

            else:
                # Imm to Reg
                result.opcode = alu_imm_ops[parts[0]]

                if self.is_imm(parts[2]):
                    # Number / const
                    result.alu_imm = self.get_imm(parts[2], "alu")
                else:
                    raise SyntaxError("Invalid mov imm: " + ' '.join(parts))


            result.address = self.get_inc_addr()
            self.instructions.append(result)
            return

        elif parts[0] == "movh":
            # Movh is only for imm
            if not is_reg(parts[1]):
                raise SyntaxError("Invalid movh reg: " + ' '.join(parts))

            if not self.is_imm(parts[2]):
                raise SyntaxError("Invalid movh imm: " + ' '.join(parts))

            result.opcode = alu_imm_ops.get("movh")

            result.src1 = get_reg(parts[1])
            result.alu_imm = self.get_imm(parts[2], "alu") >> 16 # Only top half of num

            result.address = self.get_inc_addr()
            self.instructions.append(result)
            return



        if parts[0] == "cmp":
            if is_reg(parts[2]):
                # cmp reg to reg
                result.alu_op = alu_ops.get("cmp")
                result.src1 = get_reg(parts[1])
                result.src2 = get_reg(parts[2])

            else:
                # cmp reg to imm
                result.opcode = alu_imm_ops.get("cmp")
                result.src1 = get_reg(parts[1])
                result.alu_imm = self.get_imm(parts[2], "alu")

            result.address = self.get_inc_addr()
            self.instructions.append(result)
            return

        result.src1 = get_reg(parts[2])

        if parts[0] in single_op_alu_ops:
            # If it's a single reg operation like not or neg we just want to set the opcode
            result.alu_op = alu_ops.get(parts[0])

        elif is_reg(parts[3]):
            # Reg type
            result.alu_op = alu_ops.get(parts[0])
            result.src2 = get_reg(parts[3])

        elif self.is_imm(parts[3]):
            # Imm type
            result.opcode = alu_imm_ops.get(parts[0])
            result.alu_imm = self.get_imm(parts[3], "alu")

        else:
            raise SyntaxError("Invalid src2 for line: " + ' '.join(parts))

        result.address = self.get_inc_addr()
        self.instructions.append(result)


    def first_pass(self):
        """Finds all labels, data labels and consts"""
        self.curr_section = "text"

        curr_addr = self.curr_address
        data_addr = 0

        for text in self.input:
            parts = [p.strip(",") for p in text.strip().split(";")[0].split()]
            if not parts or parts[0] == '':
                continue

            if parts[0].endswith(":"):
                # Label
                if self.curr_section == "text":
                    # normal label
                    self.labels[parts[0].rstrip(":")] = curr_addr
                elif self.curr_section == "data":
                    # data label
                    self.data_labels[parts[0].rstrip(":")] = data_addr
            elif parts[0] == "section":
                self.curr_section = parts[1].rstrip(":").lstrip(".")
            elif parts[0] == "const":
                # Const
                self.consts[parts[1]] = self.get_imm(' '.join(parts[2:]))
            elif parts[0] == "db":
                # use parse db to get length of bytes without writing to data
                data_addr += self.parse_db(text, False)

            elif parts[0] in macro_opcodes:
                curr_addr += macro_opcodes[parts[0]]*4
            else:
                curr_addr += 4

        self.data_offset = curr_addr

        for label, value in self.data_labels.items():
            self.labels[label] = curr_addr + value

        self.curr_section = "text"
        return


    def instr_to_bytes(self):
        for instr in self.instructions:

            output = 0x00000000

            output |= ((instr.opcode or 0) & 0x3F) << 26
            output |= ((instr.dest or 0) & 0xF) << 22
            output |= ((instr.src1 or 0) & 0xF) << 18
            output |= ((instr.src2 or 0) & 0xF) << 14
            output |= ((instr.alu_op or 0) & 0xF) << 10

            output |= ((instr.alu_imm or 0) & 0x3FFFF) << 0

            output |= ((instr.jmp_imm or 0) & 0x3FFFFFF) << 0

            output |= ((instr.mem_imm or 0) & 0x3FFF) << 0

            output |= ((instr.flags or 0) & 0xFFFFFFFF) << 0


            self.output.append(output)

    def assemble(self):
        self.curr_address = self.base_addr

        self.first_pass()
        self.debug_print("First pass done")

        self.parse_input()
        self.debug_print("Input parsed to instructions")

        self.instr_to_bytes()
        self.debug_print("Instructions converted to binary")

        format_width = 40

        self.debug_print()
        self.debug_print(f"{"=" * format_width} Labels {"=" * format_width}")

        for label in self.labels:
            self.debug_print(f"{label} : {self.labels.get(label)}")

        self.debug_print()
        self.debug_print(f"{"=" * format_width} Data Labels {"=" * format_width}")

        for label in self.data_labels:
            self.debug_print(f"{label} : {self.data_labels.get(label)}")

        self.debug_print()
        self.debug_print(f"{"=" * format_width} Binary {"=" * format_width}")

        for idx, value in enumerate(self.output):
            instr = self.instructions[idx]
            for label in self.labels:
                if self.labels.get(label) == instr.address:
                    self.debug_print(f"0x{instr.address:08x} : {instr.address:04d} : 0b{value:032b} : {label}:")

            self.debug_print(
                f"0x{instr.address:08x} : {instr.address:04d} : 0b{value:032b} : {instr.debug_original_text}")

        self.debug_print()
        self.debug_print(f"{"=" * format_width} Data {"=" * format_width}")

        for i in range(0, len(self.data_bytes), 4):
            for j in range(i, min(i+4, len(self.data_bytes))):
                if self.verbose:
                    print(f"0x{self.data_bytes[j]:02x}, ", end='')

            self.debug_print("")

        self.debug_print()
        self.debug_print()




if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="NEX assembler")
    parser.add_argument("input", help="Input .nesm file")
    parser.add_argument("output", nargs="?", help="Output binary file (default: input.bin)")
    parser.add_argument("--base-address", type=lambda x: int(x, 0), default=0, help="Base address for program (default: 0)")
    parser.add_argument("--verbose", action="store_true", help="Print debug output")

    args = parser.parse_args()

    input_path = args.input
    output_path = args.output
    if output_path is None:
        output_path = args.input.removesuffix(".nesm") + ".bin"

    with open(input_path, 'r') as f:
        lines = f.readlines()

    assembler = Assembler()
    assembler.input = lines
    assembler.base_addr = args.base_address

    assembler.verbose = args.verbose

    assembler.assemble()

    with open(output_path, 'wb') as f:
        for num in assembler.output:
            f.write(struct.pack(f'<I', num))

        f.write(bytes(assembler.data_bytes))

    print("Output written to: " + output_path)

