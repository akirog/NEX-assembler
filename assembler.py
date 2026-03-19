import sys
import struct
from dataclasses import dataclass
from operator import truediv


@dataclass
class Instruction:
    opcode: int = 000000 # Change this to nop when nop opcode is decided
    dest: int | None = None
    src1: int | None = None
    src2: int | None = None
    imm: int | str | None = None
    jmp_imm: int | str | None = None
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


jmp_ops = {
    "jmp":  0b0000,
    "jz":   0b0001,
    "je":   0b0001,
    "jnz":  0b0010,
    "jne":  0b0010,
    "jg":   0b0011,
    "jl":   0b0100,
    "jge":  0b0101,
    "jle":  0b0110,
    "jb":   0b0111,
    "ja":   0b1000,
    "jbe":  0b1001,
    "jae":  0b1010,
    "jr":   0b1011,
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
}



class Assembler:
    def __init__(self):
        self.curr_address: int = 0
        self.labels: dict[str, int] = {}
        self.input: list[str] = []
        self.instructions: list[Instruction] = []
        self.output: list[int] = []


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
            raise SyntaxError("Invalid register name: ", name)


    def parse_input(self):
        

        for text in self.input:
            
            parts = [p.strip(",") for p in text.strip().split(";")[0].split(" ")] or []

            new_parts = []

            for part in parts:
                if part != "":
                    new_parts.append(part)

            parts = new_parts

            if not parts or parts[0] == '' or parts[0].startswith(";"):
                continue

            instr: Instruction = Instruction()

            if parts[0].endswith(":"):
                # Label declaration
                
                self.labels[parts[0].strip(":")] = self.curr_address
                continue

            elif parts[0] in alu_ops:
                instr = self.parse_alu(parts)
                
            elif parts[0] in jmp_ops:
                instr = self.parse_jmp(parts)

            elif parts[0] in mem_ops:
                instr = self.parse_mem(parts)

            instr.debug_original_text = text.strip('\n')
            instr.address = self.curr_address
            self.instructions.append(instr)
            self.curr_address += 1


    def parse_mem(self, parts: list[str]) -> Instruction:
        """Parses a memory instruction like load r0, 0x1000 or store r3, r1"""
        result: Instruction = Instruction()
        result.opcode = mem_ops.get(parts[0])

        if result.opcode == 0b100000:
            # Load
            # Structured as: load dst_reg, [src_reg + offset]

            result.dest = self.get_reg(parts[1])

            if not parts[2].startswith("["):
                raise SyntaxError("Invalid load address, memory loads should specify address as [reg + offset] for line: ", ' '.join(parts))

            parts[2] = parts[2].lstrip("[")
            parts[-1] = parts[-1].rstrip("]")

            result.src1 = self.get_reg(parts[2])

            if len(parts) == 5 and parts[3] == "+":
                result.imm = int(parts[4])

        elif result.opcode == 0b100001:
            # Store
            # Structured as: stor [dst_reg + offset], src_reg

            parts[1] = parts[1].lstrip("[").rstrip("]")
            result.src1 = self.get_reg(parts[1])

            if len(parts) == 5 and parts[2] == "+":
                parts[3] = parts[3].rstrip("]")
                result.imm = int(parts[3])
            elif len(parts) == 5 and parts[2] != "+":
                raise SyntaxError("As of now only addition is supported in memory operation address offsets")

            result.src2 = int(parts[-1].strip("r"))

        return result


    def parse_jmp(self, parts: list[str]) -> Instruction:
        """Parses a jump instruction like jmp label or jr r0"""
        result: Instruction = Instruction()

        result.opcode = 0b010000

        # Handle r type jmp first, special case
        if parts[0] == "jr":
            result.opcode |= jmp_ops.get("jr")

            result.src1 = self.get_reg(parts[1])

            return result
        
        else:
            result.opcode |= jmp_ops.get(parts[0])

            if self.is_imm(parts[1]):
                result.jmp_imm = int(parts[1])
            elif self.is_label(parts[1]):
                # Label, gets handled in second pass
                result.jmp_imm = parts[1]

        return result



    def parse_alu(self, parts: list[str]) -> Instruction:
        """Parses an alu instruction like add"""
        result: Instruction = Instruction()

        result.opcode = 0b000000

        result.dest = self.get_reg(parts[1])
        result.src1 = self.get_reg(parts[2])

        if self.is_reg(parts[3]):
            # Reg type
            result.alu_op = alu_ops.get(parts[0])
            result.src2 = self.get_reg(parts[3])

        elif self.is_imm(parts[3]):
            # Imm type
            result.opcode = alu_ops.get(parts[0]) + 1
            result.imm = int(parts[3])

        elif self.is_label(parts[3]):
            # Label addr
            result.opcode = alu_ops.get(parts[0]) + 1
            result.imm = parts[3]

        else:
            raise SyntaxError("Invalid src2 for line: " + ' '.join(parts))

        return result

    def first_pass(self):
        """Finds all label instructions and removes them, adding the label + address to self.labels"""
        pass


    def second_pass(self):
        """Second pass replaces all labels with their addresses"""

        for instr in self.instructions:
            if isinstance(instr.imm, str):
                instr.imm = self.labels.get(instr.imm) * 4
            
            if isinstance(instr.jmp_imm, str):
                instr.jmp_imm = self.labels.get(instr.jmp_imm) - instr.address

                prev = instr.jmp_imm

                instr.jmp_imm &= 0x3FFFFFF

                if prev != instr.jmp_imm and not prev < 0:
                    raise EncodingWarning("Jmp imm might've been corrupted")

        pass

    def instr_to_bytes(self):
        for instr in self.instructions:
            
            output = 0x00000000

            output |= ((instr.opcode or 0) & 0x3F) << 26
            output |= ((instr.dest or 0) & 0xF) << 22
            output |= ((instr.src1 or 0) & 0xF) << 18
            output |= ((instr.src2 or 0) & 0xF) << 14
            output |= ((instr.alu_op or 0) & 0xF) << 10
            output |= ((instr.imm or 0) & 0x3FFFF) << 0
            output |= ((instr.jmp_imm or 0) & 0x3FFFFFF) << 0

            self.output.append(output)

    def assemble(self):

        self.parse_input()

        self.first_pass()

        self.second_pass()

        self.instr_to_bytes()
            



if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 assembler.py <input_file> <output_file>")
        sys.exit(1)
    
    input_path = sys.argv[1]
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
                print(f"0x{(instr.address*4):04x} : {instr.address:04d} : 0b{num:032b} : {label}:")

        print(f"0x{(instr.address*4):04x} : {instr.address:04d} : 0b{num:032b} : {instr.debug_original_text}")

    
    with open(output_path, 'wb') as f:
        for num in assembler.output:
            f.write(struct.pack(f'<I', num))
