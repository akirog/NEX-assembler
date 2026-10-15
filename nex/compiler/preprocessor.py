import tomllib
from pathlib import Path
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

            if line.startswith("#include "):
                # Replace line with the file
                text = line.removeprefix("#include ").strip()

                if text.startswith('"') and text.endswith('"'):
                    filename = text[1:-1]
                elif text.startswith('<') and text.endswith('>'):
                    filename = text[1:-1]
                    cfg = load_config()
                    filename = cfg["paths"]["lib"] + filename
                else:
                    raise CompileError(f"Could not parse include path", None, f"might've forgotten quotes")


                try:
                    with open(filename, 'r') as f:
                        new_lines.extend(f.readlines())
                except FileNotFoundError:
                    raise SyntaxError(f"Could not resolve include path: {filename}")


                included_file = True

                if self.verbose:
                    print(f"Included file: {filename}")

            elif line.startswith("#baseaddr "):
                self.base_addr = eval(line.removeprefix("#baseaddr ").strip())

            elif line.startswith("#define "):
                parts = line.split(" ")

                find = parts[1].strip()
                replace = parts[2].strip()

                replace_map[find] = replace

            else:
                raise SyntaxError(f"Could not match preprocessor directive: {line}")

        for key, value in replace_map.items():
            for i, line in enumerate(new_lines):
                new_lines[i] = line.replace(key, value)

        self.text = '\n'.join(new_lines)

        if included_file:
            self.process()