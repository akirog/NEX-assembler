



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
                filename = line[len("#include "):]

                with open(filename, 'r') as f:
                    new_lines.extend(f.readlines())

                included_file = True

                if self.verbose:
                    print(f"Included file: {filename}")

            elif line.startswith("#baseaddr "):
                self.base_addr = int(line[len("#baseaddr "):])

            elif line.startswith("#define "):
                parts = line.split(" ")

                find = parts[1].strip()
                replace = parts[2].strip()

                replace_map[find] = replace

            else:
                print(f"Could not match preprocessor directive: {line}")

        for key, value in replace_map.items():
            for i, line in enumerate(new_lines):
                new_lines[i] = line.replace(key, value)

        self.text = '\n'.join(new_lines)
        print(self.text)

        if included_file:
            self.process()