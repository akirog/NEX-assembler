import sys
import struct
from dataclasses import dataclass


@dataclass
class Instruction:
    opcode: int = 000000 # Change this to nop when nop opcode is decided
    dest: int | None = None
    src1: int | None = None
    src2: int | None = None
    alu_imm: int | str | None = None
    jmp_imm: int | str | None = None
    mem_imm: int | str | None = None
    address: int = 0

    # ALU operation for R type alu instructions, none for imm type instructions
    alu_op: int | None = None

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
    "movh":  0b1110,
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
    "cmp":  0b1001,
}


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



class Assembler:
    def __init__(self):
        self.curr_address: int = 0 # Bytes offset
        self.labels: dict[str, int] = {}
        self.input: list[str] = []
        self.instructions: list[Instruction] = []
        self.output: list[int] = []

        self.consts: dict[str, int] = {}

        # Data directives stuff
        self.data_bytes: list[int] = []
        self.curr_section: str = "text"
        self.data_labels: dict[str, int] = {}


    def is_reg(self, reg_name: str) -> bool:
        if reg_name in reg_names:
            return True
        else:
            return False

    def is_imm(self, imm: str) -> bool:
        if imm.lstrip("-").isnumeric():
            return True
        else:
            return False

    def is_label(self, name: str) -> bool:
        if self.is_reg(name) or self.is_imm(name):
            return False
        else:
            return True


    def get_reg(self, name: str) -> int:
        if name in reg_names:
            return reg_names[name]
        else:
            raise SyntaxError("Invalid register name: " + name)

    def get_imm(self, name: str) -> int:
        if name in self.consts:
            return self.consts[name]
        elif name.lstrip("-").isnumeric():
            return int(name)
        else:
            raise SyntaxError("Invalid immediate: " + name)

    def get_inc_addr(self) -> int:
        addr = self.curr_address
        self.curr_address += 4 # 4 bytes per instruction
        return addr

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
                # Label declaration
                if self.curr_section == "text":
                    self.labels[parts[0].strip(":")] = self.curr_address
                elif self.curr_section == "data":
                    self.data_labels[parts[0].strip(":")] = len(self.data_bytes)

                continue

            elif parts[0] == "section":
                self.curr_section = parts[1].strip(":")

                continue

            elif parts[0] == "const":
                if self.curr_section == "consts":
                    self.consts[parts[1]] = int(parts[2])
                else:
                    raise SyntaxError("Invalid const: " + parts[0] + " outside of consts section")

                continue

            elif parts[0] in alu_ops:
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

            else:
                raise SyntaxError("Invalid instruction: " + parts[0])

            for i in range(start_idx, len(self.instructions)):
                self.instructions[i].debug_original_text = self.instructions[i].debug_original_text + text.strip()

            start_idx = len(self.instructions)


    def parse_db(self, text: str):
        """Parses a db instr to a byte"""

        db_idx = text.strip().index("db")
        content = text.strip()[db_idx + 2:].strip()

        tokens = [t.strip() for t in content.split(",")]

        for i in range(len(tokens)):
            token = tokens[i]
            if token.startswith('"'):
                # string
                inner = token.strip('"')
                for char in inner:
                    self.data_bytes.append(ord(char))

            elif token.isnumeric():
                self.data_bytes.append(int(token))

            elif token.split(" ")[1] == "*" and token.split(" ")[0].isnumeric() and token.split(" ")[2].isnumeric():
                # num1 * num2, or num1 repeated num2 times
                for r in range(int(token.split(" ")[2])):
                    self.data_bytes.append(int(token.split(" ")[0]))



    def parse_io(self, parts: list[str]):
        """Parses an io instruction like io, dst, src1, src2, port"""

        result = Instruction()

        result.opcode = misc_opcodes.get("io")
        result.dest = self.get_reg(parts[1])
        result.src1 = self.get_reg(parts[2])
        result.src2 = self.get_reg(parts[3])

        # Memory imm doesn't touch opcode, dst, src1, or src2 so we use it here, normal imm expects src2 to not be used
        result.mem_imm = self.get_imm(parts[4])

        result.address = self.get_inc_addr()
        self.instructions.append(result)


    def parse_pop(self, parts: list[str]):
        """Parses a pop instr to a load and sp add"""

        # First load value at sp
        load_instr = Instruction()

        load_instr.opcode = mem_ops.get("load")
        load_instr.dest = self.get_reg(parts[1])
        load_instr.src1 = self.get_reg("sp")

        load_instr.address = self.get_inc_addr()
        self.instructions.append(load_instr)
        load_instr.debug_original_text = "load " + parts[1] + ", [sp]"
        load_instr.debug_original_text += " " * (20 - len(load_instr.debug_original_text)) + "; "

        # Then increment sp
        sp_inc = Instruction()

        sp_inc.opcode = alu_imm_ops.get("add")
        sp_inc.dest = self.get_reg("sp")
        sp_inc.src1 = self.get_reg("sp")
        sp_inc.alu_imm = 4  # Sub 4 from stack pointer to make space for curr addr

        sp_inc.address = self.get_inc_addr()
        self.instructions.append(sp_inc)
        sp_inc.debug_original_text = "add, sp, sp, 4"
        sp_inc.debug_original_text += " " * (20 - len(sp_inc.debug_original_text)) + "; "



    def parse_push(self, parts: list[str]):
        """Parses a push instr to a sp sub and a store"""

        # First decrement sp
        sp_dec = Instruction()

        sp_dec.opcode = alu_imm_ops.get("sub")
        sp_dec.dest = self.get_reg("sp")
        sp_dec.src1 = self.get_reg("sp")
        sp_dec.alu_imm = 4  # Sub 4 from stack pointer to make space for curr addr

        sp_dec.address = self.get_inc_addr()
        self.instructions.append(sp_dec)
        sp_dec.debug_original_text = "sub, sp, sp, 4"
        sp_dec.debug_original_text += " " * (20 - len(sp_dec.debug_original_text)) + "; "


        # Then store the value at sp
        store_instr = Instruction()

        store_instr.opcode = mem_ops.get("store")
        store_instr.src1 = self.get_reg("sp")
        store_instr.src2 = self.get_reg(parts[1])

        store_instr.address = self.get_inc_addr()
        self.instructions.append(store_instr)
        store_instr.debug_original_text = "store [sp], " + parts[1]
        store_instr.debug_original_text += " " * (20 - len(store_instr.debug_original_text)) + "; "


    def parse_ret(self, parts: list[str]):
        """Parses a ret instruction into a load ret addr and jmp"""

        # First load the ret addr
        load_instr = Instruction()

        load_instr.opcode = mem_ops.get("load")
        load_instr.dest = self.get_reg("lp")
        load_instr.src1 = self.get_reg("sp")

        load_instr.address = self.get_inc_addr()
        self.instructions.append(load_instr)
        load_instr.debug_original_text = "load, lp, [sp]"
        load_instr.debug_original_text += " " * (20 - len(load_instr.debug_original_text)) + "; "


        # Jump to ret address
        jump_instr = Instruction()

        jump_instr.opcode = jmp_ops.get("jr")
        jump_instr.src1 = self.get_reg("lp")

        jump_instr.address = self.get_inc_addr()
        self.instructions.append(jump_instr)
        jump_instr.debug_original_text = "jr, lp"
        jump_instr.debug_original_text += " " * (20 - len(jump_instr.debug_original_text)) + "; "




    def parse_call(self, parts: list[str]):
        """Parses a call instr to a addr push and jmp"""

        # First make the sp decrement
        sp_dec = Instruction()

        sp_dec.opcode = alu_imm_ops.get("sub")
        sp_dec.dest = self.get_reg("sp")
        sp_dec.src1 = self.get_reg("sp")
        sp_dec.alu_imm = 4 # Sub 4 from stack pointer to make space for curr addr

        sp_dec.address = self.get_inc_addr()
        self.instructions.append(sp_dec)
        sp_dec.debug_original_text = "sub, sp, sp, 4"
        sp_dec.debug_original_text += " "*(20-len(sp_dec.debug_original_text)) + "; "


        # Mov ret addr to lp
        mov_instr = Instruction()

        mov_instr.opcode = alu_imm_ops.get("mov")
        mov_instr.dest = self.get_reg("lp")
        mov_instr.alu_imm = self.curr_address + 4*3 # ret addr is after this mov, store and jump

        mov_instr.address = self.get_inc_addr()
        self.instructions.append(mov_instr)
        mov_instr.debug_original_text = "mov lp, $+12"
        mov_instr.debug_original_text += " " * (20 - len(mov_instr.debug_original_text)) + "; "


        # Store ret addr from lp to sp
        store_instr = Instruction()

        store_instr.opcode = mem_ops.get("store")
        store_instr.src1 = self.get_reg("sp")
        store_instr.src2 = self.get_reg("lp")

        store_instr.address = self.get_inc_addr()
        self.instructions.append(store_instr)
        store_instr.debug_original_text = "store [sp], lp"
        store_instr.debug_original_text += " " * (20 - len(store_instr.debug_original_text)) + "; "


        # Jump instruction
        jmp_instr = Instruction()

        # Check if a reg is called
        if self.is_reg(parts[1]):
            # call to reg
            jmp_instr.opcode = jmp_ops.get("jr")

            jmp_instr.src1 = self.get_reg(parts[1])

        else:
            # call to label
            jmp_instr.opcode = jmp_ops.get("jmp")

            jmp_instr.jmp_imm = parts[1] # Label offset is handled in second pass

        jmp_instr.address = self.get_inc_addr()
        self.instructions.append(jmp_instr)
        jmp_instr.debug_original_text = "jmp " + parts[1]
        jmp_instr.debug_original_text += " " * (20 - len(jmp_instr.debug_original_text)) + "; "


        return


    def parse_mem(self, parts: list[str]):
        """Parses a memory instruction like load r0, 0x1000 or store r3, r1"""
        result: Instruction = Instruction()
        result.opcode = mem_ops.get(parts[0])

        if result.opcode == 0b100000:
            # Load
            # Structured as: load dst_reg, [src_reg + offset]

            result.dest = self.get_reg(parts[1])

            if not parts[2].startswith("["):
                raise SyntaxError("Invalid load address, memory loads should specify address as [reg + offset] for line: " + ' '.join(parts))

            parts[2] = parts[2].lstrip("[")
            parts[-1] = parts[-1].rstrip("]")

            result.src1 = self.get_reg(parts[2])

            if len(parts) == 5 and parts[3] == "+":
                if self.is_label(parts[4]):
                    result.mem_imm = parts[4]
                else:
                    result.mem_imm = self.get_imm(parts[4])
            elif len(parts) == 5 and parts[3] == "-":
                result.mem_imm = -self.get_imm(parts[4])
            elif len(parts) == 5:
                raise SyntaxError("Only addition and subtraction supported for mem offsets: " + ' '.join(parts))

        elif result.opcode == 0b100001:
            # Store
            # Structured as: stor [dst_reg + offset], src_reg

            parts[1] = parts[1].lstrip("[").rstrip("]")
            result.src1 = self.get_reg(parts[1])

            if len(parts) == 5 and parts[2] == "+":
                parts[3] = parts[3].rstrip("]")
                result.mem_imm = self.get_imm(parts[3])
            elif len(parts) == 5 and parts[2] == "-":
                parts[3] = parts[3].rstrip("]")
                result.mem_imm = -self.get_imm(parts[3])
            elif len(parts) == 5:
                raise SyntaxError("Only addition and subtraction supported for mem offsets: " + ' '.join(parts))

            result.src2 = self.get_reg(parts[-1])

        result.address = self.get_inc_addr()
        self.instructions.append(result)


    def parse_jmp(self, parts: list[str]):
        """Parses a jump instruction like jmp label or jr r0"""
        result: Instruction = Instruction()

        # Handle r type jmp first, special case
        if parts[0] == "jr":
            result.opcode = jmp_ops.get("jr")

            result.src1 = self.get_reg(parts[1])

            result.address = self.get_inc_addr()
            self.instructions.append(result)
            return

        else:
            result.opcode = jmp_ops.get(parts[0])

            if self.is_imm(parts[1]):
                result.jmp_imm = self.get_imm(parts[1])
            elif self.is_label(parts[1]):
                # Label, gets handled in second pass
                result.jmp_imm = parts[1]

        result.address = self.get_inc_addr()
        self.instructions.append(result)



    def parse_alu(self, parts: list[str]):
        """Parses an alu instruction like add"""
        result: Instruction = Instruction()

        result.opcode = 0b000000

        result.dest = self.get_reg(parts[1])
        result.src1 = self.get_reg(parts[2])

        if parts[0] == "cmp":
            if self.is_reg(parts[2]):
                result.alu_op = alu_ops.get("cmp")
                result.src1 = self.get_reg(parts[1])
                result.src2 = self.get_reg(parts[2])

            else:
                result.opcode = alu_imm_ops.get("cmp")
                result.src1 = self.get_reg(parts[1])
                result.alu_imm = self.get_imm(parts[2])


            result.opcode = alu_ops.get("cmp")

        elif len(parts) == 3:
            # If it's a single reg operation like not or neg we just want to set the opcode
            result.alu_op = alu_ops.get(parts[0])

        elif self.is_reg(parts[3]):
            # Reg type
            result.alu_op = alu_ops.get(parts[0])
            result.src2 = self.get_reg(parts[3])

        elif self.is_imm(parts[3]):
            # Imm type
            result.opcode = alu_imm_ops.get(parts[0])
            result.alu_imm = self.get_imm(parts[3])

        elif self.is_label(parts[3]):
            # Label addr
            result.opcode = alu_imm_ops.get(parts[0])
            result.alu_imm = parts[3]

        else:
            raise SyntaxError("Invalid src2 for line: " + ' '.join(parts))

        result.address = self.get_inc_addr()
        self.instructions.append(result)

    def first_pass(self):
        """Finds all label instructions and removes them, adding the label + address to self.labels"""
        pass


    def second_pass(self):
        """Second pass replaces all labels with their addresses"""

        for instr in self.instructions:
            if isinstance(instr.alu_imm, str):
                if instr.alu_imm in self.labels:
                    instr.alu_imm = self.labels.get(instr.alu_imm)
                elif instr.alu_imm in self.data_labels:
                    instr.alu_imm = self.data_labels.get(instr.alu_imm) + self.curr_address
                else:
                    raise SyntaxError("Label " + instr.alu_imm + " not found")

            if isinstance(instr.jmp_imm, str):
                if instr.jmp_imm in self.labels:
                    instr.jmp_imm = (self.labels.get(instr.jmp_imm) - instr.address) // 4
                else:
                    raise SyntaxError("Label " + instr.jmp_imm + " not found")

                prev = instr.jmp_imm

                if prev != instr.jmp_imm and not prev < 0:
                    raise EncodingWarning("Jmp imm might've been corrupted")

            if isinstance(instr.mem_imm, str):
                if instr.mem_imm in self.data_labels:
                    instr.mem_imm = self.data_labels.get(instr.mem_imm) + self.curr_address
                else:
                    raise SyntaxError("Label " + instr.mem_imm + " not found")

        pass

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

            self.output.append(output)

    def assemble(self):

        self.parse_input()
        print("Input parsed to instructions")

        self.first_pass()
        print("First pass done")

        self.second_pass()
        print("Second pass done")

        self.instr_to_bytes()
        print("Instructions converted to binary")




if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 assembler.py <input_file> <output_file>")
        sys.exit(1)

    input_path = sys.argv[1]

    if len(sys.argv) < 3:
        output_path = sys.argv[1].removesuffix(".nesm") + ".bin"
    else:
        output_path = sys.argv[2]

    with open(input_path, 'r') as f:
        lines = f.readlines()

    assembler = Assembler()
    assembler.input = lines
    assembler.assemble()

    format_width = 40

    print()
    print(f"{"="*format_width} Labels {"="*format_width}")

    for label in assembler.labels:
        print(f"{label} : {assembler.labels.get(label)}")

    print()
    print(f"{"="*format_width} Binary {"="*format_width}")

    for instr_num, num in enumerate(assembler.output):
        instr = assembler.instructions[instr_num]
        for label in assembler.labels:
            if assembler.labels.get(label) == instr.address:
                print(f"0x{instr.address:08x} : {instr.address:04d} : 0b{num:032b} : {label}:")

        print(f"0x{instr.address:08x} : {instr.address:04d} : 0b{num:032b} : {instr.debug_original_text}")

    print()
    print(f"{"=" * format_width} Data {"=" * format_width}")

    for byte in assembler.data_bytes:
        print(f"0x{byte:02x}")

    with open(output_path, 'wb') as f:
        for num in assembler.output:
            f.write(struct.pack(f'<I', num))

        f.write(bytes(assembler.data_bytes))

    print()
    print()

    print("Output written to: " + output_path)

