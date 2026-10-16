import re

from nex.compiler.frame_classes import Span, Position
from ..errors import CompileError


class Token(tuple):
    def __new__(cls, kind, value, span: Span|None=None):
        t = super().__new__(cls, (kind, value))
        t.span = span
        return t

    def __repr__(self):
        return f"({self[0]}, {self[1]}, {self.span.source[self.span.start.offset:self.span.end.offset]})" if self.span else f"({self[0]}, {self[1]})"



class Lexer:
    """Takes text and tokenizes it into a list of tokens and their original values."""
    def __init__(self):
        self.input: str = ""
        self.tokens: list[Token] = []
        self.token_patters = {
            "COMMENT": r'//.*|/\*[\s\S]*?\*/',
            "WHITESPACE": r'\s+',

            # Literals
            "INT_LITERAL": r'0[xX][0-9a-fA-F]+|0[bB][01]+|0[oO][0-7]+|\d+',
            "STRING_LITERAL": r'"[^"]*"',
            "CHAR_LITERAL": r"'([^'\\]|\\.)?'",
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
            "CONST": r'\bconst\b',

            # Logical operations
            "LOGICAL_OR": r'\|\|',
            "LOGICAL_AND": r'\&\&',

            # Arithmetic operations
            "ARROW": r'->',
            "PLUS": r'\+',
            "MINUS": r'-',
            "STAR": r'\*',
            "SLASH": r'/',
            "PERCENT": r'\%',
            "SHL": r'<<',
            "SHR": r'>>',
            "OR": r'\|',
            "XOR": r'\^',
            "AND": r'\&',

            # Comparison operations
            "LESS_EQUALS": r'<=',
            "GREATER_EQUALS": r'>=',
            "GREATER": r'\>',
            "LESS": r'<',
            "CMP_EQUALS": r'==',
            "CMP_NOT_EQUALS": r'!=',

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
        line_num = 1
        column = 0
        lines = self.input.splitlines()

        while position < len(self.input):
            # Check for assembly block
            if self.input[position:].startswith("asm ("):
                # Handle assembly block separate from regex
                start_span = Position(line_num, 0, position)
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

                line_num += self.input[position:end].count("\n")

                new_lines = []
                for line in text.split("\n"):
                    new_lines.append(line.split("//")[0].strip().strip('"'))

                position = end
                end_span = Position(line_num, 0, position)
                span = Span(start_span, end_span, self.input)

                self.tokens.append(Token("ASM_BLOCK", '\n'.join(new_lines), span))

                continue

            match = self.regex.match(self.input, position)


            if not match:
                start = Position(line_num, 0, position)
                end = Position(line_num, 0, position+10)

                raise CompileError("Lexer error", Span(start, end, self.input))


            kind = match.lastgroup
            value = match.group()
            start = Position(line_num, column, position)
            position = match.end()
            column += len(value)
            end = Position(line_num, column, position)

            if kind == "WHITESPACE":
                if value.count("\n") != 0:
                    column = 0
                    column += len(value.split("\n")[-1])
                line_num += value.count("\n")
                continue
            elif kind == "COMMENT":
                if value.count("\n") != 0:
                    column = 0
                line_num += value.count("\n")
                continue
            elif kind == "UNKNOWN":
                span = Span(start, end, self.input)
                raise CompileError(f"Unrecognized token: {value}", span)

            span = Span(start, end, self.input)

            self.tokens.append(Token(kind, value, span))