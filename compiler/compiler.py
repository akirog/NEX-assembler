import argparse
from lexer import Lexer
from parser import Parser

class Compiler:
    def __init__(self):
        self.input: str = ""
        self.output: list[str] = []



    def compile(self, verbose: bool = False):

        # Lexer:
        # Create token list
        # Check for lexer errors
        lexer = Lexer()
        lexer.input = self.input
        lexer.tokenize()

        if verbose:
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
            print(parser.ast)
            print()
            print()
            pass

        # Semantic analyzer:
        # Build type table
        # Build scope stack
        # Check for semantic errors
        # Probably 3 different phases


        # Code generator
        # Take verified ast, type table and scope stack
        # Create assembly instructions from ast



        pass



if __name__ == "__main__":
    arg_parser = argparse.ArgumentParser(description="NEX compiler")
    arg_parser.add_argument("input", help="Input .nex file")
    arg_parser.add_argument("output", nargs="?", help="Output NEX assembly file (default: input.nesm)")
    arg_parser.add_argument("--base-address", type=lambda x: int(x, 0), default=0, help="Base address for program (default: 0)")
    arg_parser.add_argument("--verbose", action="store_true", help="Print debug output")

    args = arg_parser.parse_args()

    input_path = args.input
    output_path = args.output or args.input.removesuffix(".nex") + ".nesm"

    with open(input_path, 'r') as f:
        lines = f.read()

    compiler = Compiler()
    compiler.input = lines
    compiler.compile(args.verbose)

    with open(output_path, 'w') as f:
        f.writelines(compiler.output)

    print("Output written to: " + output_path)

