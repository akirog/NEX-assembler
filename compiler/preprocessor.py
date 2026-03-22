



class Preprocessor:
    def __init__(self, verbose: bool = False):
        self.text: str = ""
        self.verbose = verbose



    def process(self):
        included_file = False
        lines = self.text.split('\n')

        for i, line in enumerate(lines):
            if not line.startswith('#'):
                continue

            if line.startswith("#include "):
                # Replace line with the file
                filename = line[len("#include "):]

                # Copy the lines after the inclusion
                after_lines = lines[i+1:].copy()

                lines = lines[:i-1]

                with open(filename, 'r') as f:
                    lines.extend(f.readlines())

                lines.extend(after_lines)

                included_file = True

                if self.verbose:
                    print(f"Included file: {filename}")

        self.text = '\n'.join(lines)


        if included_file:
            self.process()