import operator
import re
import struct
import argparse
from dataclasses import dataclass
from typing import Any, List, Dict

INSTR_LENGTHS = {
    "R_ALU":    1,
    "I_ALU":    1,
    "MOV":      2,
    "R_JUMP":   1,
    "IO":       1,
    "INT":      1,
    "MEM":      1,
    "BRANCH":   1,
    "JUMP":     1,
}


REG_OPS = {
    "alu":  0b000000,
    "jr":   0b000001,
    "jrl":  0b000010,
    "io":   0b000011,
    "int":  0b011001,
}

IMM_OPS = {
    "storeb":   0b111,
    "store":    0b101,
    "loadb":    0b110,
    "load":     0b100,
}

JMP_OPS = {
    "j":    0b010111,
    "jal":  0b011000,
}


INT_OPS = {
    "trigint": 0b000,
    "getcode": 0b001,
    "setvec":  0b010,
    "iret":    0b011,
    "getra":   0b100,
    "setra":   0b101,
}

BRANCH_OPS = {
    "b":    0b010000,
    "bgt":  0b010001,
    "blt":  0b010010,
    "beq":  0b010011,
    "bne":  0b010100,
    "bgtu": 0b010101,
    "bltu": 0b010110,
}

REG_ALU_OPS = {
    "add":  0b0000,
    "sub":  0b0001,
    "and":  0b0010,
    "or":   0b0011,
    "xor":  0b0100,
    "neg":  0b0101,
    "shl":  0b0110,
    "shr":  0b0111,
    "mul":  0b1000,
    "div":  0b1001,
    "mod":  0b1010,
    "lt":   0b1011,
    "lte":  0b1100,
    "eq":   0b1101,
    "ne":   0b1110,
}

IMM_ALU_OPS = {
    "addi":  0b1000,
    "addhi": 0b1001,
    "ori":   0b1010,
    "subi":  0b1011,
    "shli":  0b1100,
    "shri":  0b1101,
    "muli":  0b1110,
    "divi":  0b1111,
}

# Registers aliases
REGISTER_ALIASES = {
    "zero": 0,
    "at":   1,
    "v0":   2,
    "v1":   3,
    "a0":   4,
    "a1":   5,
    "a2":   6,
    "a3":   7,
    "t0":   8,
    "t1":   9,
    "t2":   10,
    "t3":   11,
    "t4":   12,
    "t5":   13,
    "t6":   14,
    "t7":   15,
    "t8":   16,
    "t9":   17,
    "t10":  18,
    "t11":  19,
    "t12":  20,
    "t13":  21,
    "t14":  22,
    "t15":  23,
    "t16":  24,
    "t17":  25,
    "k0":   26,
    "k1":   27,
    "gp":   28,
    "sp":   29,
    "bp":   30,
    "ra":   31,
}


IO_ALIASES = {
    "HALT": 0,
    "SCREEN": 2
}


class Instruction:
    def to_bytes(self):
        raise NotImplementedError("To bytes not implemented on base class")

class RegInstruction(Instruction):
    def __init__(self, opcode: int = 0, src1: int = 0, src2: int = 0, dst: int = 0, fn: int = 0):
        self.opcode = opcode
        self.src1 = src1
        self.src2 = src2
        self.dst = dst
        self.fn = fn

    def to_bytes(self):
        value = 0
        value |= self.opcode << 26
        value |= self.src1 << 21
        value |= self.src2 << 16
        value |= self.dst << 11
        value |= self.fn

        return value

    def __repr__(self):
        return f"reg instruction: {self.opcode} {self.src1} {self.src2} {self.dst} {self.fn}"


class ImmInstruction(Instruction):
    def __init__(self, opcode: int = 0, src1: int = 0, dst: int = 0, imm: int = 0):
        self.opcode = opcode
        self.src1 = src1
        self.dst = dst
        self.imm = imm

    def to_bytes(self):
        value = 0
        value |= (self.opcode & 0b111111) << 26
        value |= (self.dst & 0b11111) << 21
        value |= (self.src1 & 0b11111) << 16
        assert(self.imm & 0xFFFF == self.imm)
        value |= (self.imm & 0xFFFF)

        print(f"{value:032b}")
        return value


    def __repr__(self):
        return f"imm instr: {self.opcode} {self.src1} {self.dst} {self.imm}"


class JumpInstruction(Instruction):
    def __init__(self, opcode: int = 0, offset: int = 0):
        self.opcode = opcode
        self.offset = offset

    def to_bytes(self):
        value = 0
        value |= (self.opcode & 0b111111) << 26
        value |= (self.offset & 0xFFFFFF)

        return value

    def __repr__(self):
        return f"jump instruction: {self.opcode} {self.offset}"


class Lexer:
    def __init__(self):
        self.input = ""
        self.output: List[(str, Any)] = []

        self.patterns = {
            "LABEL": r'[a-zA-Z_][a-zA-Z_0-9]*:',

            "SECTION": r'section \.?[a-zA-Z_][a-zA-Z0-9_]*',
            "DB": r'db',

            "I_ALU": r'addhi|addi|subi|ori|shli|shri|muli|divi',
            "R_ALU": r'add|sub|and|or|xor|neg|shl|shr|mul|div|mod|lt|lte|eq|ne',
            "MOV": r'mov',
            "R_JUMP": r'jrl|jr',
            "IO": r'io',
            "INT": r'trigint|getcode|setvec|iret|getra|setra',

            "MEM": r'loadb|load|storeb|store',
            "BRANCH": r'bgtu|bltu|bgt|blt|beq|bne|b',

            "JUMP": r'jal|j',

            "IO_OP": r'halt|screen|rom|ssdr|ssdw|time',

            "REG": r'r\d|a\d|t\d', # r0, a0, t0
            "NUM": r'(?:0x[0-9a-fA-F]+|0b[01]+|\d+)',

            "DB": r'db',
            "DW": r'dw',

            "LBRACKET": r'\[',
            "RBRACKET": r'\]',
            "PLUS": r'\+',
            "MINUS": r'\-',

            "DOLLAR": r'\$',

            "IDENTIFIER": r'[a-zA-Z_][a-zA-Z_0-9]*',

            "STRING": r'\"[a-zA-Z0-9_\s\!]*\"',

            "COMMENT": r';[^\n]*',

            "COMMA": r'\,',
            "WHITESPACE": r'\s',
        }

    def compile(self):
        print("|".join(f'(?<{g}>{p})' for g, p in self.patterns.items()))

        pattern = re.compile("|".join(f'(?P<{g}>{p})' for g, p in self.patterns.items()))

        print(pattern)

        position = 0
        while position < len(self.input):
            match = pattern.match(self.input[position:])
            if match is None:
                print(self.input[position:])
                raise SyntaxError(f"Lexer error: Unable to match input with regex expression {self.input[position:position+2]}")

            position += match.end()

            skippers = ["WHITESPACE", "COMMA", "COMMENT"]
            if match.lastgroup in skippers:
                continue

            token = (match.lastgroup, match.string[:match.end()])

            if token[0] == "LABEL":
                token = (token[0], token[1].rstrip(":"))
            self.output.append(token)

        print(self.output)


class Assembler:
    def __init__(self):
        self.verbose = False
        self.base_addr: int = 0
        self.input: str = ""

        # Type : value
        self.tokens: List[(str, Any)] = []
        self.token_idx: int = 0

        self.text_section_size: int = 0

        self.data_tokens: List[(str, Any)] = []

        self.data_labels: Dict[(str, int)] = {}
        self.labels: Dict[str, int] = {}
        self.instructions: List[Instruction] = []

        # Instruction bytes
        self.instr_bytes: List[int] = []
        # Bytes for the .data section
        self.data_bytes: List[int] = []


    def peek(self, idx: int = 0):
        return self.tokens[self.token_idx + idx]

    def consume(self, match: str = None):
        token = self.peek()
        if match and token[0] != match:
            raise SyntaxError(f"{token} did not match {match}")

        self.token_idx += 1
        return token

    def expect(self, token_type: str):
        assert(self.consume()[0] == token_type)

    def assemble(self):
        lexer = Lexer()
        lexer.input = self.input
        lexer.compile()

        self.tokens = lexer.output

        self.parse_numbers()

        self.separate_sections()

        self.collect_data_labels()
        self.collect_labels()
        print(self.labels)

        self.resolve_labels()
        self.parse_registers()
        print(self.tokens)

        print(self.data_tokens)
        self.parse_data_section()
        for num in self.data_bytes:
            print(f"data: {num:08b}")

        self.parse_instructions()
        for instr in self.instructions:
            print(instr)

        self.to_bytes()


    def to_bytes(self):
        for instr in self.instructions:
            self.instr_bytes.append(instr.to_bytes())


    def parse_instructions(self):
        curr_addr = 0

        while self.token_idx < len(self.tokens):
            if self.peek()[0] in INSTR_LENGTHS:
                curr_addr += INSTR_LENGTHS[self.peek()[0]] * 4

            if self.peek()[0] == "I_ALU":
                self.parse_i_alu(curr_addr)

            elif self.peek()[0] == "R_ALU":
                self.parse_r_alu()

            elif self.peek()[0] == "MOV":
                self.parse_mov(curr_addr)

            elif self.peek()[0] == "R_JUMP":
                self.parse_reg_jump()

            elif self.peek()[0] == "IO":
                self.parse_io()

            elif self.peek()[0] == "INT":
                self.parse_int()

            elif self.peek()[0] == "MEM":
                self.parse_mem(curr_addr)

            elif self.peek()[0] == "BRANCH":
                self.parse_branch(curr_addr)

            elif self.peek()[0] == "JUMP":
                self.parse_jump(curr_addr)

            else:
                raise NotImplementedError(f"{self.peek()} not implemented yet")


    def parse_jump(self, curr_addr):
        instr = JumpInstruction()
        instr.opcode = JMP_OPS[self.consume("JUMP")[1]]
        instr.offset = (self.parse_imm(curr_addr) - (curr_addr + self.base_addr)) >> 2

        self.instructions.append(instr)


    def parse_branch(self, curr_addr):
        instr = ImmInstruction()
        instr.opcode = BRANCH_OPS[self.consume("BRANCH")[1]]
        instr.dst = self.consume("REG")[1]
        instr.src1 = self.consume("REG")[1]
        instr.imm = (self.parse_imm(curr_addr) - (curr_addr + self.base_addr)) >> 2

        self.instructions.append(instr)


    def parse_mem(self, curr_addr):
        instr = ImmInstruction()
        opcode = self.consume("MEM")[1]
        instr.opcode = IMM_OPS[opcode]

        if opcode.startswith("store"):
            assert(self.peek()[0] == "LBRACKET")

        if self.peek()[0] == "LBRACKET":
            self.expect("LBRACKET")
            instr.src1 = self.consume("REG")[1]
            instr.imm = self.parse_imm(curr_addr, skip_first=True)
            self.expect("RBRACKET")
            instr.dst = self.consume("REG")[1]
        else:
            instr.dst = self.consume("REG")[1]
            self.expect("LBRACKET")
            instr.src1 = self.consume("REG")[1]
            instr.imm = self.parse_imm(curr_addr, skip_first=True)
            self.expect("RBRACKET")

        self.instructions.append(instr)


    def parse_int(self):
        instr = RegInstruction()
        instr.opcode = REG_OPS["int"]
        instr.fn = INT_OPS[self.consume("INT")[1]]
        instr.dst = self.consume("REG")[1]
        instr.src1 = self.consume("REG")[1]
        instr.src2 = self.consume("REG")[1]

        self.instructions.append(instr)


    def parse_io(self):
        instr = RegInstruction()
        instr.opcode = REG_OPS[self.consume("IO")[1]]
        instr.dst = self.consume("REG")[1]
        instr.src1 = self.consume("REG")[1]
        instr.src2 = self.consume("REG")[1]

        if self.peek()[0] == "NUM":
            instr.fn = self.consume("NUM")[1]
        else:
            instr.fn = IO_ALIASES[self.consume("IO_OP")[1]]

        self.instructions.append(instr)


    def parse_reg_jump(self):
        instr = RegInstruction()
        instr.opcode = REG_OPS[self.consume("R_JUMP")[1]]
        instr.src1 = self.consume("REG")[1]

        self.instructions.append(instr)


    def parse_mov(self, curr_addr):
        self.consume("MOV")
        instr = ImmInstruction()
        instr.opcode = IMM_ALU_OPS["addi"]
        instr.dst = self.consume("REG")[1]

        value = self.parse_imm(curr_addr)
        print(f"value: {value:b}")
        instr.imm = value & 0xFFFF

        self.instructions.append(instr)
        instr2 = ImmInstruction()

        instr2.opcode = IMM_ALU_OPS["addhi"]
        instr2.dst = instr.dst
        instr2.src1 = instr2.dst
        instr2.imm = value >> 16

        self.instructions.append(instr2)


    def parse_r_alu(self):
        instr = RegInstruction()
        instr.opcode = REG_OPS["alu"]
        opcode = self.consume("R_ALU")[1]
        instr.fn = REG_ALU_OPS[opcode]
        instr.dst = self.consume("REG")[1]
        instr.src1 = self.consume("REG")[1]

        if opcode != "neg":
            print("getting reg2")
            print(self.peek())
            instr.src2 = self.consume("REG")[1]

        self.instructions.append(instr)


    def parse_i_alu(self, curr_addr):
        instr = ImmInstruction()
        instr.opcode = IMM_ALU_OPS[self.consume("I_ALU")[1]]
        instr.dst = self.consume("REG")[1]
        instr.src1 = self.consume("REG")[1]
        instr.imm = self.parse_imm(curr_addr)

        self.instructions.append(instr)


    def parse_data_section(self):
        addr = 0

        i = 0
        while i < len(self.data_tokens):
            token = self.data_tokens[i]

            if token[0] in ["DB", "DW"]:
                while i + 1 < len(self.data_tokens) and self.data_tokens[i + 1][0] in ["NUM", "DOLLAR"]:
                    i += 1
                    addr += 1

                    value = 0

                    if self.data_tokens[i][0] == "NUM":
                        value = self.tokens[i][1]
                        i += 1
                    elif self.data_tokens[i][0] == "DOLLAR":
                        value = addr
                        i += 1
                    else: raise SyntaxError(f"unexpected token in data section: {token}")

                    while i < len(self.data_tokens) and self.data_tokens[i][0] in ["PLUS", "MINUS"]:
                        operation = self.data_tokens[i][0]
                        i += 1
                        right = 0
                        if self.data_tokens[i][0] == "NUM":
                            right = self.tokens[i][1]
                            i += 1
                        elif self.data_tokens[i][0] == "DOLLAR":
                            right = addr
                            i += 1
                        else:
                            raise SyntaxError(f"unexpected token in data section: {token}")

                        if operation == "PLUS":
                            value = value + right
                        else:
                            value = value - right

                    if token[0] == "DB":
                        if value & 0xFF != value:
                            raise ValueError(f"db value exceeds one byte {value}")

                        self.data_bytes.append(value)
                    else:
                        for i in range(4):
                            self.data_bytes.append((value >> 8*i)&0xFF)

            else:
                raise SyntaxError(f"unexpected token in data section: {token}")

            i += 1



    def parse_imm(self, curr_addr, skip_first=False, data_section=False):
        # 10 - $ + label
        if not skip_first:
            left = self.parse_primary_imm(curr_addr, data_section)
        else:
            left = 0

        while self.peek()[0] in ["PLUS", "MINUS"]:
            operation = self.consume()[0]
            right = self.parse_primary_imm(curr_addr, data_section)
            if operation == "PLUS":
                left = left + right
            else:
                left = left - right
        return left


    def parse_primary_imm(self, curr_addr, data_section=False):
        if self.peek()[0] == "NUM":
            return self.consume("NUM")[1]
        elif self.peek()[0] == "DOLLAR":
            return curr_addr + (self.base_addr if data_section else 0)
        else:
            raise ValueError(f"Could not parse primary imm: {self.peek()}")


    def parse_numbers(self):
        new_tokens: List[(str, Any)] = []

        for i in range(len(self.tokens)):
            token = self.tokens[i]

            if token[0] == "NUM":
                value = str(token[1])
                if value.isdigit():
                    token = ("NUM", int(value))
                elif value.startswith("0x"):
                    token = ("NUM", int(value, 16))
                elif value.startswith("0b"):
                    token = ("NUM", int(value, 2))

                new_tokens.append(token)
            elif token[0] == "STRING":
                string = token[1]
                for c in string:
                    value = ord(c)
                    new_tokens.append(("NUM", value))
            else:
                new_tokens.append(token)
        self.tokens = new_tokens

        new_data_tokens: List[(str, Any)] = []

        for i in range(len(self.data_tokens)):
            token = self.data_tokens[i]
            print(token)

            if token[0] == "NUM":
                value = str(token[1])
                if value.isdigit():
                    token = ("NUM", int(value))
                elif value.startswith("0x"):
                    token = ("NUM", int(value, 16))
                elif value.startswith("0b"):
                    token = ("NUM", int(value, 2))

                new_data_tokens.append(token)
            elif token[0] == "STRING":
                string = token[1]
                for c in string:
                    print(c)
                    value = ord(c)
                    new_data_tokens.append(("NUM", value))
            else:
                new_data_tokens.append(token)
        self.data_tokens = new_data_tokens


    def parse_registers(self):
        new_tokens = []

        for token in self.tokens:
            if token[0] == "REG":
                value = str(token[1])
                if value.startswith("r"):
                    token = ("REG", int(value.lstrip("r")))
                elif value in REGISTER_ALIASES:
                    token = ("REG", REGISTER_ALIASES[value])
                else:
                    raise SyntaxError(f"Could not parse register {value}")

            new_tokens.append(token)


        self.tokens = new_tokens

    def resolve_labels(self):
        new_tokens: List[(str, Any)] = []

        for token in self.tokens:
            if token[0] == "IDENTIFIER":
                if token[1] in self.labels:
                    token = ("NUM", self.labels[token[1]] + self.base_addr)
                elif token[1] in self.data_labels:
                    token = ("NUM", self.data_labels[token[1]])
                else:
                    raise SyntaxError(f"Label '{token[1]}' is not defined")
            new_tokens.append(token)

        self.tokens = new_tokens
        new_data_tokens = []
        for token in self.data_tokens:
            if token[0] == "IDENTIFIER":
                if token[1] in self.labels:
                    token = ("NUM", self.labels[token[1]] + self.base_addr)
                elif token[1] in self.data_labels:
                    token = ("NUM", self.data_labels[token[1]])
                else:
                    raise SyntaxError(f"Label '{token[1]}' is not defined")

            new_data_tokens.append(token)

        self.data_tokens = new_data_tokens


    def collect_labels(self):
        addr = 0
        new_tokens = []

        for token in self.tokens:
            new_tokens.append(token)
            if token[0] in INSTR_LENGTHS:
                addr += INSTR_LENGTHS[token[0]] * 4

            elif token[0] == "LABEL":
                if token[1] in self.labels:
                    raise SyntaxError(f"Label '{token[1]}' is already defined")
                new_tokens.pop()
                self.labels[token[1]] = addr

        self.text_section_size = addr
        self.tokens = new_tokens


    def collect_data_labels(self):
        addr = 0

        i = 0
        while i < len(self.data_tokens):
            token = self.data_tokens[i]

            if token[0] == "LABEL":
                if token[1] in self.data_labels:
                    raise ValueError(f"Label '{token[1]}' already defined")

                self.data_labels[token[1]] = addr
                # Skip adding label token
                self.data_tokens.pop(i)
                continue

            elif token[0] == "DB":
                while i + 1 < len(self.data_tokens) and self.data_tokens[i+1][0] in ["NUM", "IDENTIFIER", "DOLLAR"]:
                    i += 1
                    addr += 1
                    # Make sure not to increment / stop on operations
                    while self.data_tokens[i+1][0] == "PLUS" or self.data_tokens[i+1][0] == "MINUS":
                        i += 2
            elif token[0] == "DW":
                while i + 1 < len(self.data_tokens) and self.data_tokens[i+1][0] in ["NUM", "IDENTIFIER", "DOLLAR"]:
                    i += 1
                    addr += 4
                    # Make sure not to increment / stop on operations
                    while i + 1 < len(self.data_tokens) and (self.data_tokens[i + 1][0] == "PLUS" or self.data_tokens[i + 1][0] == "MINUS"):
                        i += 2

            i += 1


    def separate_sections(self):
        new_tokens = []

        section = "text"
        for token in self.tokens:
            if token[0] == "SECTION":
                section = token[1].split()[-1].lstrip(".")
                continue

            if section == "text":
                new_tokens.append(token)
            elif section == "data":
                self.data_tokens.append(token)
            else:
                raise SyntaxError(f"Could not parse section {section}")

        self.tokens = new_tokens


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
        text = f.read()

    assembler = Assembler()
    assembler.input = text
    assembler.base_addr = args.base_address

    assembler.verbose = args.verbose

    assembler.assemble()

    with open(output_path, 'wb') as f:
        for num in assembler.instr_bytes:
            f.write(struct.pack(f'<I', num))

        f.write(bytes(assembler.data_bytes))

    print("Output written to: " + output_path)

