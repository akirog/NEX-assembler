import sys
import struct
from dataclasses import dataclass


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
    "inc":  0b1101,
    "dec":  0b1110,
    "cmp":  0b1111,
}


jmp_ops = {
    "jmp":  0b0000,
    "je":   0b0001,
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


class Assembler:
    def __init__(self):
        self.curr_address: int = 0
        self.labels: dict[str, int] = {}
        self.input: list[str] = []
        self.instructions: list[Instruction] = []
        self.output: list[int] = []
    

    def parse_input(self):
        

        for text in self.input:
            
            parts = [p.strip(",") for p in text.strip().split(" ")] or []

            if not parts or parts[0] == '':
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

            instr.address = self.curr_address
            self.instructions.append(instr)
            self.curr_address += 1


    def parse_jmp(self, parts: list[str]) -> Instruction:
        """Parses a jump instruction like jmp label or jr r0"""
        instr: Instruction = Instruction()

        instr.opcode = 0b010000

        # Handle r type jmp first, special case
        if parts[0] == "jr":
            instr.opcode |= jmp_ops.get("jr")

            instr.src1 = parts[1].strip("r")

            return instr
        
        else:
            instr.opcode |= jmp_ops.get(parts[0])

            if parts[1].isnumeric():
                instr.jmp_imm = self.parse_imm(parts[1])
            else:
                # Label, gets handled in second pass
                instr.jmp_imm = parts[1]



    def parse_alu(self, parts: list[str]) -> Instruction:
        """Parses an alu instruction like add"""
        instr: Instruction = Instruction()

        instr.opcode = 0b000000

        instr.dest = self.parse_imm(parts[1].strip("r"))
        instr.src1 = self.parse_imm(parts[2].strip("r"))

        if parts[3].startswith("r"):
            # Reg type
            instr.alu_op = alu_ops.get(parts[0])
            instr.src2 = self.parse_imm(parts[3].strip("r"))

        elif parts[3].isnumeric():
            # Imm type
            instr.opcode = alu_ops.get(parts[0])+1
            instr.imm = self.parse_imm(parts[3])

        else:
            raise SyntaxError("Invalid src2 for line: ", ' '.join(parts))

        return instr

    def parse_imm(self, imm: str) -> int:
        return int(imm)
                

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

        pass

    def instr_to_bytes(self):
        for instr in self.instructions:
            
            output = 0x00000000

            output |= (instr.opcode or 0) << 26
            output |= (instr.dest or 0) << 22
            output |= (instr.src1 or 0) << 18
            output |= (instr.src2 or 0) << 14
            output |= (instr.alu_op or 0) << 10
            output |= (instr.imm or 0) << 0
            output |= (instr.jmp_imm or 0) << 0

            self.output.append(output)

    def assemble(self):

        self.parse_input()

        self.first_pass()

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
    print(f"{"="*format_width} Labels {"="*format_width}")

    for label in assembler.labels:
        print(f"{label} : {assembler.labels.get(label)}")
    
    print()
    print(f"{"="*format_width} Binary {"="*format_width}")

    for instr_num, num in enumerate(assembler.output):
        instr = assembler.instructions[instr_num]
        print(f"0x{(instr.address*4):04x} : {instr.address:04d} : 0b{num:032b}")

    
    with open(output_path, 'wb') as f:
        for num in assembler.output:
            f.write(struct.pack(f'<I', num))
