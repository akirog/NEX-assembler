import operator
import struct
import argparse
from dataclasses import dataclass
from typing import Any, List


class Assembler:
    def __init__(self):
        self.verbose = False
        self.base_addr: int = 0
        self.input: List[str] = []

        # Type : value
        self.tokens: List[(str, str)] = []


    def parse_input(self, line: str):
        self.input.append(line)


    def debug_print(self):
        pass

    def debug_print(self, text: str = ""):
        if self.verbose:
            print(text)


    def assemble(self):
        format_width = 40

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

