import re


class Lexer:
    """Takes text and tokenizes it into a list of tokens and their original values."""
    def __init__(self):
        self.input: str = ""
        self.tokens: list[tuple[str, str]] = []
        self.token_patters = {
            "COMMENT": r'//.*|/\*[\s\S]*?\*/',
            "WHITESPACE": r'\s+',

            # Literals
            "INT_LITERAL": r'\d+',
            "STRING_LITERAL": r'"[^"]*"',
            "CHAR_LITERAL": r'\'[^"]\'',
            "BOOL_LITERAL": r'\btrue\b|\bfalse\b',

            "STRUCT": r'\bstruct\b',

            # Control flow
            "IF": r'\bif\b',
            "ELSE": r'\belse\b',
            "WHILE": r'\bwhile\b',
            "FOR": r'\bfor\b',
            "RETURN": r'\breturn\b',
            "CONTINUE": r'\bcontinue\b',
            "BREAK": r'\bbreak\b',

            # Logical operations
            "LOGICAL_OR": r'\|\|',
            "LOGICAL_AND": r'\&\&',

            # Arithmetic operations
            "PLUS": r'\+',
            "MINUS": r'-',
            "STAR": r'\*',
            "SLASH": r'/',
            "PERCENT": r'\%',
            "SHL": r'<<',
            "SHR": r'>>',
            "OR": r'\|',
            "AND": r'\&',

            # Comparison operations
            "GREATER": r'\>',
            "LESS": r'<',
            "CMP_EQUALS": r'==',
            "CMP_NOT_EQUALS": r'!=',
            "LESS_EQUALS": r'<=',
            "GREATER_EQUALS": r'>=',

            # Unary operators
            "EXCLAMATION": r'!',

            # Delimiters
            "LPAREN": r'\(',
            "RPAREN": r'\)',
            "LBRACKET": r'\[',
            "RBRACKET": r'\]',
            "LBRACE": r'\{',
            "RBRACE": r'\}',

            # Assignment
            "EQUALS": r'=',

            # Punctuation
            "DOT": r'\.',
            "COMMA": r'\,',
            "COLON": r'\:',
            "SEMICOLON": r';',

            # Identifier
            "IDENTIFIER": r'[a-zA-Z_]\w*',

            # Unknown
            "UNKNOWN": r'.'
        }

        self.regex = re.compile("|".join(f"(?P<{name}>{pattern})" for name, pattern in self.token_patters.items()))


    def tokenize(self):

        position = 0

        while position < len(self.input):
            # Check for assembly block
            if self.input[position:].startswith("asm ("):
                # Handle assembly block separate from regex
                start = position + len("asm (")
                brackets = 0
                end = start

                while brackets >= 0:
                    if self.input[end] == "(":
                        brackets += 1
                    elif self.input[end] == ")":
                        brackets -= 1

                    end += 1

                text = self.input[start:end-1]

                new_lines = []
                for line in text.split("\n"):
                    new_lines.append(line.split("//")[0].strip().strip('"'))

                self.tokens.append(("ASM_BLOCK", '\n'.join(new_lines)))
                position = end
                continue

            match = self.regex.match(self.input, position)

            if not match:
                # Get line num
                line = 0
                pos = 0
                while pos < position:
                    if self.input[pos] == "\n":
                        line += 1

                raise SyntaxError("Lexing error on line: " + str(line))

            kind = match.lastgroup
            value = match.group()
            position = match.end()

            if kind == "WHITESPACE":
                continue
            elif kind == "COMMENT":
                continue
            elif kind == "UNKNOWN":
                raise SyntaxError("Unrecognized token")

            self.tokens.append((kind, value))


