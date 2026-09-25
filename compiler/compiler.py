import argparse
import struct
from argparse import Namespace

from assembler.assembler import Assembler
from .code_generator import CodeGenerator
from .preprocessor import Preprocessor
from .semantic_analyzer import *
from .lexer import Lexer
from .parser import Parser

class Compiler:
    def __init__(self):
        self.input: str = ""
        self.output: list[str] = []
        self.base_addr: int = 0


    def compile(self, verbose: bool = False):

        print_width = 40

        # Preprocessor:
        # removes and applies preprocessor directives
        preprocessor = Preprocessor(verbose)
        preprocessor.text = self.input
        preprocessor.process()
        self.input = preprocessor.text
        self.base_addr = preprocessor.base_addr

        # Lexer:
        # Create token list
        # Check for lexer errors
        lexer = Lexer()
        lexer.input = self.input
        lexer.tokenize()

        if verbose and False:
            print(f"{'=' * print_width} TOKENS {'=' * print_width}")
            print('\n'.join(f"{kind}" + " "*(15-len(kind)) + f": {value}" for kind, value in lexer.tokens))
            print()
            print()

        # Parser:
        # Parse input tokens into ast
        # Check for syntax errors
        parser = Parser()
        parser.tokens = lexer.tokens
        parser.parse_program()

        if verbose:
            print(f"{'=' * print_width} AST {'=' * print_width}")
            print(parser.ast)
            print()
            print()
            pass

        # Semantic analyzer:
        # Build type table
        # Build scope stack
        # Check for semantic errors
        # Probably 3 different phases
        semantic_analyzer = SemanticAnalyzer()
        semantic_analyzer.ast = parser.ast
        semantic_analyzer.verbose = verbose
        semantic_analyzer.analyze()



        # Code generator
        # Take verified ast, type table and scope stack
        # Create assembly instructions from ast
        code_generator = CodeGenerator()
        code_generator.ast = semantic_analyzer.ast
        code_generator.global_frame = semantic_analyzer.global_frame
        code_generator.type_table = semantic_analyzer.type_table
        code_generator.base_address = preprocessor.base_addr
        code_generator.generate()
        self.output = code_generator.output

        if verbose:
            print(f"{'=' * print_width} GENERATED CODE BY COMPILER {'=' * print_width}")
            print('\n'.join(code_generator.output))
            print()
            print()

        pass


def main(args: Namespace):
    input_path = args.input
    output_path = args.output or args.input.removesuffix(".nex") + ".bin"

    with open(input_path, 'r') as f:
        lines = f.read()

    compiler = Compiler()
    compiler.input = lines
    compiler.compile(args.verbose)

    # Now run the assembler on the output
    assembler = Assembler()
    assembler.input = '\n'.join(compiler.output) # Assembler expects one string
    assembler.base_addr = compiler.base_addr
    assembler.verbose = args.verbose
    assembler.assemble()

    bin_path: str = output_path

    with open(bin_path, 'wb') as f:
        for num in assembler.instr_bytes:
            f.write(struct.pack(f'<I', num))

        f.write(bytes(assembler.data_bytes))

    print("Output binary written to: " + bin_path)


if __name__ == "__main__":
    arg_parser = argparse.ArgumentParser(description="NEX compiler")
    arg_parser.add_argument("input", help="Input .nex file")
    arg_parser.add_argument("output", nargs="?", help="Output binary file (default: input.bin)")
    arg_parser.add_argument("--verbose", action="store_true", help="Print debug output")

    args = arg_parser.parse_args()

    main(args)