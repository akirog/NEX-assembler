import tomllib
from pathlib import Path

from .frame_classes import Position, Span
from ..errors import CompileError


def load_config(path="nex.toml"):
    path = Path(path)
    with path.open("rb") as f:
        cfg = tomllib.load(f)
    cfg["_root"] = path.parent.resolve()
    return cfg



class Preprocessor:
    def __init__(self, verbose: bool = False):
        self.text: str = ""
        self.verbose = verbose
        self.base_addr: int = 0



    def process(self):
        included_file = False
        lines = self.text.split('\n')
        new_lines = []

        replace_map = {}

        for i, line in enumerate(lines):
            if not line.startswith('#'):
                new_lines.append(line)
                continue

            if line.startswith("#include"):
                # Replace line with the file
                text = line.removeprefix("#include ").strip()

                if text.startswith('"') and text.endswith('"'):
                    filename = text[1:-1]
                elif text.startswith('<') and text.endswith('>'):
                    filename = text[1:-1]
                    cfg = load_config()
                    filename = cfg["paths"]["lib"] + filename
                else:
                    span = Span(Position(i+1, len("#include "), 0), Position(i+1, len(line), 0), self.text)
                    raise CompileError(f"Could not parse include path", span, f"forgot quotes?")


                try:
                    with open(filename, 'r') as f:
                        new_lines.extend(f.readlines())
                except FileNotFoundError:
                    span = Span(Position(i + 1, len("#include ") + 1, 0), Position(i + 1, len(line) - 1, 0), self.text)
                    raise CompileError(f"File does not exist", span)


                included_file = True

                if self.verbose:
                    print(f"Included file: {filename}")

            elif line.startswith("#baseaddr"):
                try:
                    self.base_addr = eval(line.removeprefix("#baseaddr ").strip())
                except Exception:
                    span = Span(Position(i + 1, len("#baseaddr "), 0), Position(i + 1, len(line), 0), self.text)
                    raise CompileError(f"Could not evaluate base address expression", span)

            elif line.startswith("#define"):
                parts = line.split(" ")
                if len(parts) != 3:
                    span = Span(Position(i + 1, len("#define "), 0), Position(i + 1, len(line), 0), self.text)
                    raise CompileError(f"Wrong amount of operands for define", span, f"Intended use is #define [FIND] [REPLACE]")

                find = parts[1].strip()
                replace = parts[2].strip()

                replace_map[find] = replace

            else:
                span = Span(Position(i + 1, 0, 0), Position(i + 1, len(line), 0), self.text)
                raise CompileError(f"Unknown preprocessor directive", span)

        for key, value in replace_map.items():
            for i, line in enumerate(new_lines):
                new_lines[i] = line.replace(key, value)

        self.text = '\n'.join(new_lines)

        if included_file:
            self.process()