import argparse
import struct
from argparse import Namespace
from pathlib import Path

from ..assembler.assembler import assemble
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




def compile_source(source: str, verbose: bool = False) -> tuple[str, int]:
    compiler = Compiler()
    compiler.input = source
    compiler.compile(verbose)

    return "\n".join(compiler.output), compiler.base_addr


def compile_file(input_path: str, output_path: str = None, emit_asm=False, verbose: bool = False) -> str:
    if output_path is None:
        output_path = input_path
        output_path = output_path.removesuffix(".nex")
        output_path = "build/" + output_path.split("/")[-1] + ".bin"

    asm, base = compile_source(Path(input_path).read_text(), verbose=verbose)

    if emit_asm:
        Path(output_path).with_suffix(".nesm").write_text(asm)

    out = Path(output_path)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(assemble(asm, base_address=base, verbose=verbose))
    return output_path