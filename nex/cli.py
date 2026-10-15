import argparse
import time

from math import exp

from .assembler.assembler import assemble_file
from .compiler.compiler import compile_file
from .filesystem.fs_builder import build_fs
from .errors import *



def generate_error_message(e):
    message = f"\n"

    if e.span:
        if e.span.source_file:
            message += f"{e.span.source_file}\n"

        # Remove tabs
        original_line = e.span.source.splitlines()[e.span.start.line_num-1]
        line = ""
        for c in original_line:
            if c == "\t":
                line += " "
                continue
            line += c

        error_start = e.span.start.col
        error_end = e.span.end.col
        add = 0
        if e.span.start.line_num:
            line_message = f"Line {e.span.start.line_num}: "
            message += line_message
            add = len(line_message)

        message += f"{line} \n"
        message += f"{'~'*(error_start+add)}{'^'*(error_end-error_start)} \n"

    if e.message:
        message += e.message + "\n"

    if e.hint:
        message += e.hint + "\n"

    return message



# Command line interface for using nex
# CLI commands
def asm(args):
    print("Assembling...")
    try:
        path = assemble_file(args.input_file, args.output_path, args.base_address, args.verbose)
    except AssembleError as e:
        print(f"Assembly failed with error: {e}")
        return

    print("Assembling complete!")
    print(f"Assembled to: {path}")

def compile(args):
    print("Compiling...")
    try:
        path = compile_file(args.input_file, args.output_path, args.emit_asm, args.verbose)
    except CompileError as e:
        raise RuntimeError(generate_error_message(e))

    print("Compilation complete!")
    print(f"Compiled to: {path}")

def build(args):
    print("Building...")

    try:
        build_fs(args.verbose)
    except CompileError as e:
        raise RuntimeError(generate_error_message(e))
    except AssembleError as e:
        print(f"Assemble failed with error: {e}")

    print("Build complete!")


def main():
    parser = argparse.ArgumentParser(
        prog='nex',
        description='Nex cli tool for assembling, compiling, building etc',
    )
    parser.add_argument(
        "--elapsed",
        action="store_true",
        help="Display elapsed time of command"
    )

    subparsers = parser.add_subparsers(
        dest="command",
        required=True,
    )


    asm_parser = subparsers.add_parser(
        "asm",
        help="Assemble a .nesm file"
    )
    asm_parser.add_argument(
        "input_file",
        type=str,
        help="Input file to assemble"
    )
    asm_parser.add_argument(
        "output_path",
        type=str,
        nargs='?',
        help="Output path for the assembled file"
    )
    asm_parser.add_argument(
        "base_address",
        type=int,
        nargs='?',
        default=0,
        help="Base address of the assembled file"
    )
    asm_parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose mode"
    )



    compile_parser = subparsers.add_parser(
        "cc",
        help="Compile a .nex file"
    )
    compile_parser.add_argument(
        "input_file",
        type=str,
        help="Input file to compile"
    )
    compile_parser.add_argument(
        "output_path",
        type=str,
        nargs='?',
        help="Output path for the compiled file"
    )
    compile_parser.add_argument(
        "-e", "--emit_asm",
        action="store_true",
        help="Emit the assembled file"
    )
    compile_parser.add_argument(
        "-v" ,"--verbose",
        action="store_true",
        help="Verbose mode"
    )



    build_parser = subparsers.add_parser(
        "build",
        help="Build the filesystem"
    )
    build_parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose mode"
    )


    arguments = parser.parse_args()

    start_time = time.time()

    match arguments.command:
        case "asm":
            asm(arguments)
        case "cc":
            compile(arguments)
        case "build":
            build(arguments)
        case _:
            parser.print_help()

    end_time = time.time()

    time_elapsed = end_time - start_time

    if arguments.elapsed:
        print(f"Command completed in {time_elapsed:.01} seconds")


if __name__ == '__main__':
    main()