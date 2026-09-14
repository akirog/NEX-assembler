import operator
import re
import struct
import argparse
from dataclasses import dataclass
from typing import Any, List


class Lexer:
    def __init__(self):
        self.input = ""
        self.output = []

        self.patterns = {
            "R_ALU": r'add|sub|or',
            "REG": r'r\d|a\d|t\d',
            "NUM": r'\d+',

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
                raise SyntaxError("Lexer error: Unable to match input with regex expression")

            position += match.end()

            skippers = ["WHITESPACE", "COMMA"]
            if match.lastgroup in skippers:
                continue

            token = (match.lastgroup, match.string[:match.end()])
            self.output.append(token)



        print(self.output)




class Assembler:
    def __init__(self):
        self.verbose = False
        self.base_addr: int = 0
        self.input: str = ""

        # Type : value
        self.tokens: List[(str, str)] = []

        # Instruction bytes
        self.output: List[int] = []
        # Bytes for the .data section
        self.data_bytes: List[int] = []


    def assemble(self):
        lexer = Lexer()
        lexer.input = self.input
        lexer.compile()




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
        for num in assembler.output:
            f.write(struct.pack(f'<I', num))

        f.write(bytes(assembler.data_bytes))

    print("Output written to: " + output_path)

