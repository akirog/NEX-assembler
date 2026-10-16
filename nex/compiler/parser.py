from itertools import takewhile
from typing import Callable

from .ast_nodes import *
from .frame_classes import *
from .lexer import Token
from ..errors import CompileError

builtin_type_literals = [
    "BOOL_LITERAL",
    "INT_LITERAL",
    "CHAR_LITERAL",
    "STRING_LITERAL",
]

operations = [
    "+",
    "-",
    "*",
    "/",
    "%",
    "&",
    "|",
    "^",
    "<<",
    ">>",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "&&",
    "||",
    "!",
]

precedence = {
    "||": 1,
    "&&": 2,
    "|": 3,
    "^": 4,
    "&": 5,
    "==": 6,
    "!=": 6,
    "<": 7,
    ">": 7,
    "<=": 7,
    ">=": 7,
    "<<": 8,
    ">>": 8,
    "+": 9,
    "-": 9,
    "*": 10,
    "/": 10,
    "%": 10,
}

unary_ops = [
    "!",
    "-"
]


def span_from(start: Span, end: Span) -> Span:
    if start.source != end.source:
        raise RuntimeError(f"Expected merged tokens to have same source")
    return Span(start.start, end.end, end.source)


class Parser:
    """Parses a list of tokens to an ast, and checks for syntax errors"""

    def __init__(self):
        self.position: int = 0
        self.tokens: list[Token] = []
        self.ast: ProgramNode = ProgramNode()
        self.type_names = {"int", "uint", "uint8", "char", "bool", "void"}

    def peek(self, offset: int = 0) -> Token:
        if self.position + offset >= len(self.tokens):
            return Token("EOF", "")

        return self.tokens[self.position + offset]

    def consume(self) -> Token:
        self.position += 1
        return self.tokens[self.position - 1]

    def expect(self, kind: str, error: Exception = None):
        if self.tokens[self.position][0] != kind:
            if error:
                raise error
            else:
                raise SyntaxError(f"Expected {kind}, but found {self.tokens[self.position-2:self.position+5]}")

        self.position += 1

    def accept(self, kind: str):
        if self.peek()[0] == kind:
            self.expect(kind)
            return True

        return False

    def parse_program(self):
        while self.position < len(self.tokens):
            self.ast.body.nodes.append(self.parse_statement())

    def parse_statement(self) -> AstNode:
        node: AstNode
        start = self.peek().span

        if self.peek()[0] == "IDENTIFIER":
            # Variable decl, func decl or variable assignment
            if self.peek(1)[0] == "IDENTIFIER":
                # Variable decl or function decl
                node = self.parse_variable_decl()

            elif (self.peek(1)[0] == "EQUALS" or
                  (self.peek(1)[1] in operations and self.peek(2)[0] == "EQUALS") or
                  self.peek(1)[0] == "LBRACKET"
            ):
                # Variable assignment
                node = self.parse_variable_assignment()

            elif self.peek(1)[0] == "LPAREN":
                # Function call or variable decl
                if self.peek()[1] in self.type_names:
                    node = self.parse_variable_decl()
                else:
                    node = self.parse_function_call()

            elif self.peek(1)[0] == "DOT":
                # Member assignment is also handled by variable assignment function
                node = self.parse_variable_assignment()

            elif self.peek(1)[0] == "STAR":
                # Also variable declaration
                node = self.parse_variable_decl()

            elif self.peek(1)[0] == "ARROW":
                # Variable assignment
                node = self.parse_variable_assignment()

            else:
                span = self.peek(1).span
                span.start.line_num = start.start.line_num
                raise CompileError(f"Invalid variable name", span)

        elif self.peek()[0] == "STAR":
            # Dereference, parse as variable assignment
            node = self.parse_variable_assignment()

        elif self.peek()[0] == "LPAREN":
            node = self.parse_variable_assignment()

        elif self.peek()[0] == "IF":
            node = self.parse_if()

        elif self.peek()[0] == "WHILE":
            node = self.parse_while()

        elif self.peek()[0] == "FOR":
            node = self.parse_for()

        elif self.peek()[0] == "STRUCT":
            node = self.parse_struct_definition()

        elif self.peek()[0] == "RETURN":
            node = self.parse_return()

        elif self.peek()[0] == "BREAK":
            node = BreakNode()
            self.expect("BREAK")
            self.expect("SEMICOLON", "expected semicolon after break")

        elif self.peek()[0] == "CONTINUE":
            node = ContinueNode()
            self.expect("CONTINUE")
            self.expect("SEMICOLON", "expected semicolon after continue")

        elif self.peek()[0] == "ASM_BLOCK":
            node = AssemblyBlockNode()
            node.assembly = self.consume()[1].split('\n')
            self.expect("SEMICOLON", "expected semicolon after asm block")

        else:
            span = self.peek().span
            span.start.line_num = start.start.line_num
            raise CompileError(f"Invalid statement", span)

        node.span = span_from(start, self.peek(-1).span)
        return node

    def parse_for(self) -> AstNode:
        node = ForNode()
        self.expect("FOR")

        err_message = f"Failed to parse for loop"
        self.expect("LPAREN", CompileError(err_message, self.peek().span))
        node.init_expr = self.parse_variable_decl()
        node.condition = self.parse_expression()
        self.expect("SEMICOLON", "expected semicolon after for loop update expr")
        node.update_expr = self.parse_variable_assignment(False)
        self.expect("RPAREN", f"expected end of for loop header, found {self.peek()}")

        self.expect("LBRACE", CompileError(err_message, self.peek(-1).span))
        node.body = self.parse_body()
        self.expect("RBRACE", f"expected end of for loop body, found {self.peek()}")
        return node

    def parse_while(self) -> AstNode:
        node = WhileNode()
        self.expect("WHILE")

        err_message = f"Failed to parse while loop"
        self.expect("LPAREN", f"expected start of while loop header, found {self.peek()}")
        node.condition = self.parse_expression()
        self.expect("RPAREN", f"expected end of for while header, found {self.peek()}")

        self.expect("LBRACE", CompileError(err_message, self.peek(-1).span, "Missing '{'?"))
        node.body = self.parse_body()
        self.expect("RBRACE", f"expected end of while loop body, found {self.peek()}")
        return node

    def parse_return(self) -> AstNode:
        node = ReturnNode()
        self.expect("RETURN")

        if self.peek()[0] != "SEMICOLON":
            node.ret_expr = self.parse_expression()

        self.expect("SEMICOLON", "expected semicolon after return statement")
        return node

    def parse_function_call(self) -> AstNode:
        node = FunctionCallNode()

        node.func = IdentifierNode(self.consume()[1])
        node.func.span = self.peek(-1).span

        self.expect("LPAREN")

        # Handle parameters
        while self.peek()[0] != "RPAREN":
            arg = self.parse_expression()

            node.args.append(arg)

            if self.peek()[0] == "COMMA":
                self.expect("COMMA")
            else:
                break

        self.expect("RPAREN")
        self.expect("SEMICOLON", "expected semicolon after function call")

        return node

    def parse_struct_definition(self) -> AstNode:
        node = StructDeclNode()
        self.expect("STRUCT")

        if self.peek()[0] != "IDENTIFIER":
            raise CompileError(f"Missing struct name", self.peek(-1).span)
        node.name = self.consume()[1]
        self.type_names.add(node.name)

        self.expect("LBRACE", CompileError(f"Missing left brace", self.peek(-1).span))

        # Struct fields parsing is one of a kind so we just do it here
        while self.peek()[0] != "RBRACE":
            field = FieldNode()
            field.type = PrimitiveType(self.consume()[1])

            name, wrap, _ = self.parse_type()
            field.type = wrap(field.type)
            field.name = name

            node.fields.append(field)
            self.expect("SEMICOLON", CompileError("expected semicolon after struct field definition", self.peek(-1).span))

        self.expect("RBRACE")
        self.expect("SEMICOLON", CompileError("expected semicolon after struct definition", self.peek(-1).span))
        return node

    def parse_if(self) -> AstNode:
        node = IfNode()
        self.expect("IF")

        self.expect("LPAREN")
        node.condition = self.parse_expression()
        self.expect("RPAREN")

        # Check for a single line if
        if self.peek()[0] != "LBRACE":
            node.body.nodes = [self.parse_statement()]

        else:
            self.expect("LBRACE")
            node.body = self.parse_body()
            self.expect("RBRACE")

        # Check for else or else if
        bottom_if_node: IfNode = node
        while self.peek()[0] == "ELSE":
            self.expect("ELSE")

            # else or else if
            else_node = IfNode()
            if self.peek()[0] == "LBRACE":
                # Parse as body, no else if
                # Condition of value 1 should always evaluate to true
                else_node.condition = ValueNode(1)
                else_node.condition.type = PrimitiveType("int")

            elif self.peek()[0] == "IF":
                # else if
                self.expect("IF")

                self.expect("LPAREN")
                else_node.condition = self.parse_expression()
                self.expect("RPAREN")

            else:
                raise SyntaxError(f"Unexpected token after else keyword: {self.peek()}")

            # Check for a single line statement
            if self.peek()[0] != "LBRACE":
                else_node.body.nodes = [self.parse_statement()]
            else:
                self.expect("LBRACE")
                else_node.body = self.parse_body()
                self.expect("RBRACE")

            bottom_if_node.else_node = else_node
            bottom_if_node = else_node

        return node

    def parse_variable_assignment(self, check_semicolon: bool = True) -> AssignmentNode:
        node = AssignmentNode()
        node.target = self.parse_target()

        compound_op: str | None = None
        if self.peek()[1] in operations:
            compound_op = self.consume()[1]

        self.expect("EQUALS")
        node.expression = self.parse_expression()

        if compound_op:
            binary_op = BinaryOpNode()

            binary_op.left = node.target
            binary_op.right = node.expression
            binary_op.operation = compound_op

            node.expression = binary_op

        if check_semicolon:
            self.expect("SEMICOLON")

        return node

    def parse_target(self) -> AstNode:
        """Parses a target, for example *x, point.y, or y"""

        if self.peek()[0] == "STAR":
            # Dereference
            self.expect("STAR")
            node = DereferenceNode()
            node.address_expression = self.parse_expression()
            return node

        elif self.peek()[0] == "AND":
            # Address of
            self.expect("AND")
            # Parse target here since address needs to be of a variable
            node = AddressOfNode()
            node.variable = self.parse_target()
            return node

        elif self.peek()[0] == "IDENTIFIER":
            variable = IdentifierNode(self.consume()[1])
            variable.span = self.peek(-1).span

            while self.peek()[0] == "DOT" or self.peek()[0] == "LBRACKET" or self.peek()[0] == "ARROW":
                if self.peek()[0] == "DOT":
                    # Member access
                    self.expect("DOT")
                    member_access = MemberAccessNode()
                    member_access.variable = variable
                    member_access.member = self.consume()[1]
                    variable = member_access
                elif self.peek()[0] == "ARROW":
                    self.expect("ARROW")
                    variable = DereferenceNode(variable)
                    member_access = MemberAccessNode()
                    member_access.variable = variable
                    member_access.member = self.consume()[1]
                    variable = member_access
                else:
                    # Array indexing
                    self.expect("LBRACKET")
                    index_expression = IndexExpressionNode()
                    index_expression.base = variable
                    index_expression.index = self.parse_expression()
                    variable = index_expression
                    self.expect("RBRACKET")

            return variable

        elif self.accept("LPAREN"):
            target = self.parse_target()
            self.expect("RPAREN")

            # Check for member access / array indexing
            while self.peek()[0] == "DOT" or self.peek()[0] == "LBRACKET" or self.peek()[0] == "ARROW":
                if self.peek()[0] == "DOT":
                    # Member access
                    self.expect("DOT")
                    member_access = MemberAccessNode()
                    member_access.variable = target
                    member_access.member = self.consume()[1]
                    target = member_access
                elif self.peek()[0] == "ARROW":
                    self.expect("ARROW")
                    target = DereferenceNode(target)
                    member_access = MemberAccessNode()
                    member_access.variable = target
                    member_access.member = self.consume()[1]
                    target = member_access
                else:
                    # Array indexing
                    self.expect("LBRACKET")
                    index_expression = IndexExpressionNode()
                    index_expression.base = target
                    index_expression.index = self.parse_expression()
                    target = index_expression
                    self.expect("RBRACKET")

            return target

        else:
            raise SyntaxError(f"Cant parse target: {self.peek()}")

    def parse_member_access(self, variable: AstNode) -> AstNode:
        """Parses a member access like p.x.y recursively, using the variable from the previous call as variable"""
        member_access = MemberAccessNode()
        member_access.variable = variable
        member_access.member = self.consume()[1]

        if self.peek()[0] == "DOT":
            # Nested member access
            self.expect("DOT")
            node = self.parse_member_access(member_access)
            return node

        else:
            # If we reached the end of the nesting we return the member access
            return member_access

    def parse_variable_decl(self) -> VariableDeclNode | FunctionDeclNode:
        node = VariableDeclNode()

        node.type = PrimitiveType(self.consume()[1])

        start_type = self.peek().span
        inner, wrap, fields = self.parse_type()
        node.type = wrap(node.type)

        node.name = inner
        end_type = self.peek(-1).span

        if self.peek()[0] == "SEMICOLON":
            self.expect("SEMICOLON")
            return node

        # Check for function declaration
        if self.peek()[0] == "LBRACE":
            if fields is None:
                raise CompileError(f"no fields defined for function declaration", span_from(start_type, end_type))
            node = self.parse_function_declaration(inner, node.type, fields)
            return node

        span = self.peek(-1).span
        span.start.line_num = self.peek(-2).span.start.line_num
        self.expect("EQUALS", CompileError(f"Expected semicolon or equals after variable declaration", span))

        # Check for array
        if isinstance(node.type, ArrayType) and self.peek()[0] == "LBRACE":
            node.init_value = self.parse_array_literal()

        else:
            # Parse normal expression
            node.init_value = self.parse_expression()

        if isinstance(node.type, ArrayType) and node.type.length is None:
            if not isinstance(node.init_value, StringLiteralNode):
                raise RuntimeError(f"Unable to get static length of array type: {node}")

            node.type.length = len(node.init_value.literal)

        span = self.peek(-1).span
        self.expect("SEMICOLON", CompileError(f"Missing semicolon after variable declaration", span))

        return node

    def parse_array_literal(self) -> ArrayLiteralNode:
        """Parses an array literal like { 0, 7, 2 }"""
        node = ArrayLiteralNode()
        # Normal array declaration
        self.expect("LBRACE")

        while self.peek()[0] != "RBRACE":
            if self.peek()[0] == "LBRACE":
                node.elements.append(self.parse_array_literal())
            else:
                node.elements.append(self.parse_expression())

            if not self.accept("COMMA"):
                break

        node.length = len(node.elements)

        self.expect("RBRACE", CompileError(f"Unclosed array literal", self.peek(-1).span))
        return node

    def parse_function_declaration(self, name, type, fields) -> FunctionDeclNode:
        node = FunctionDeclNode()

        node.type = type
        node.name = name
        node.args = fields

        self.expect("LBRACE")
        node.body = self.parse_body()
        self.expect("RBRACE")

        return node

    def parse_body(self) -> BodyNode:
        """Parses a body of code stopping at "}"."""
        node = BodyNode()
        while self.peek()[0] != "RBRACE" and self.peek()[0] != "EOF":
            node.nodes.append(self.parse_statement())
        if self.peek()[0] == "EOF":
            raise SyntaxError(f"Couldn't parse body, missing right brace")

        return node

    def parse_expression(self) -> AstNode:
        """Parse an expression like x + 5 - y recursively, stops if it finds a rparen, so giving x + 5) - y, it stops at 5 and leaves ")" as curr token."""
        left: AstNode
        start = self.peek().span

        left = self.parse_operand()

        expression_enders = ["RPAREN", "SEMICOLON", "COMMA", "RBRACKET", "RBRACE", "EQUALS"]
        if self.peek()[0] in expression_enders:
            return left

        if self.peek()[1] in operations:
            operation = self.consume()[1]
        else:
            span = self.peek().span
            span.start.line_num = start.start.line_num
            raise CompileError(f"Couldn't parse operation", span)

        right: AstNode

        if self.peek()[0] == "LPAREN":
            if self.is_type_start(1):
                right = self.parse_type_cast()

            else:
                right = self.parse_expression()

        else:
            right = self.parse_expression()

        node = BinaryOpNode()
        node.left = left
        node.right = right
        node.operation = operation

        if isinstance(right, BinaryOpNode):
            if right.operation not in precedence or operation not in precedence:
                raise RuntimeError(f"Couldn't find precedence of operation: {right.operation} or {operation}")

            if precedence[right.operation] < precedence[operation]:
                node.right = right.right
                node.operation = right.operation

                temp = BinaryOpNode()
                temp.operation = operation
                temp.right = right.left
                temp.left = left

                node.left = temp

        node.span = span_from(start, self.peek(-1).span)
        return node

    def parse_operand(self) -> AstNode:
        """Parses an operand like x, 5, *x, &y, *(something)"""
        unary_op = None

        start = self.peek().span

        # Check for struct initialization
        if self.peek()[0] == "LBRACE":
            self.expect("LBRACE")
            node = StructInitNode()

            while self.peek()[0] != "RBRACE":
                node.args.append(self.parse_expression())

                if self.peek()[0] == "COMMA":
                    self.expect("COMMA")
                else:
                    break

            end = self.peek().span
            self.expect("RBRACE")
            node.span = span_from(start, end)
            return node

        elif self.accept("STAR"):
            node = DereferenceNode()
            node.address_expression = self.parse_operand()
            end = self.peek(-1).span
            node.span = span_from(start, end)
            return node

        # Consume unary op
        while self.peek()[1] in unary_ops:
            if unary_op is None:
                unary_op = UnaryOpNode()
                unary_op.span = self.peek().span
                unary_op.operation = self.consume()[1]
            else:
                unary_op.right = UnaryOpNode()
                unary_op.right.span = self.peek().span
                unary_op.right.operation = self.consume()[1]

        # Consume atom
        atom: AstNode
        if self.peek()[0] == "LPAREN":
            atom_start = self.peek().span
            if self.is_type_start(1):
                atom = self.parse_type_cast()

            else:
                # Normal parenthesis expression
                self.expect("LPAREN")
                atom = self.parse_expression()
                self.expect("RPAREN")

            atom.span = span_from(atom_start, self.peek(-1).span)


        elif self.peek()[0] == "IDENTIFIER":
            atom = IdentifierNode(self.consume()[1])
            atom.span = self.peek().span


        elif self.peek()[0] in builtin_type_literals:
            atom = ValueNode()
            atom.span = self.peek().span
            if self.peek()[0] == "INT_LITERAL":
                try:
                    atom.value = eval(self.consume()[1])
                except Exception as e:
                    raise CompileError(f"Eval error: {e}", atom.span)

                atom.type = PrimitiveType("int")
            elif self.peek()[0] == "CHAR_LITERAL":
                value = self.consume()[1].strip("'")
                try:
                    atom.value = ord(bytes(value, "utf-8").decode("unicode_escape")) if len(value) != 0 else 0
                except Exception as e:
                    raise CompileError(f"Ord error: {e}", atom.span)

                atom.type = PrimitiveType("char")
            elif self.peek()[0] == "BOOL_LITERAL":
                atom.value = 1 if self.consume()[1] == "true" else 0
                atom.type = PrimitiveType("bool")
            elif self.peek()[0] == "STRING_LITERAL":
                atom = StringLiteralNode()
                atom.literal = self.consume()[1].strip('"') + "\0"

            else:
                raise SyntaxError(f"Couldn't parse operand: {self.peek()[0]}")

        elif self.peek()[0] == "AND":
            atom_start = self.peek().span
            self.expect("AND")
            var = self.parse_operand()
            atom = AddressOfNode()
            atom.variable = var
            atom.span = span_from(atom_start, self.peek(-1).span)

        else:
            raise CompileError("Couldn't parse operand", self.peek().span)

        # Loop postfixes
        while True:
            if self.peek()[0] == "LPAREN":
                # Function call
                node = FunctionCallNode()
                node.func = atom

                self.expect("LPAREN")
                while self.peek()[0] != "RPAREN":
                    node.args.append(self.parse_expression())

                    if self.peek()[0] == "RPAREN":
                        break
                    self.expect("COMMA")

                self.expect("RPAREN")
                atom = node

            elif self.peek()[0] == "LBRACKET":
                # Array access
                self.expect("LBRACKET")
                array_node = IndexExpressionNode()
                array_node.base = atom
                array_node.index = self.parse_expression()
                atom = array_node
                self.expect("RBRACKET")

            elif self.peek()[0] == "DOT":
                self.expect("DOT")

                atom = self.parse_member_access(atom)

            elif self.peek()[0] == "ARROW":
                self.expect("ARROW")
                atom = DereferenceNode(atom)
                atom = self.parse_member_access(atom)

            else:
                break

        inner_unary: None | UnaryOpNode = unary_op

        while inner_unary is not None and inner_unary.right is not None and isinstance(inner_unary.right, UnaryOpNode):
            inner_unary = inner_unary.right

        if inner_unary is not None:
            inner_unary.right = atom
            atom = inner_unary

        end = self.peek(-1).span
        atom.span = span_from(start, end)

        return atom


    def parse_type(self, get_name=True) -> tuple[str, Callable[[TypeNode], TypeNode], list[FieldNode] | None]:

        ptr_count = 0
        while self.accept("STAR"):
            ptr_count += 1

        name = ""
        inner_wrap: None | Callable[[TypeNode], TypeNode] = None
        fields: list[FieldNode] | None = None
        if self.peek()[0] == "LPAREN":
            self.expect("LPAREN")
            name, inner_wrap, fields = self.parse_type(get_name)
            self.expect("RPAREN", CompileError(f"Unclosed type parenthesis", self.peek(-1).span))

        else:
            # Check for inner type / name
            if get_name:
                if self.peek()[0] != "IDENTIFIER":
                    span = span_from(self.peek(-1).span, self.peek().span)
                    raise CompileError(f"No inner name in variable type", span)

                name = self.consume()[1]

                # Check for function declaration
                if self.peek()[0] == "LPAREN":
                    self.expect("LPAREN")
                    fields = []
                    # Parse function fields
                    while not self.accept("RPAREN"):
                        field = FieldNode()
                        field.type = PrimitiveType(self.consume()[1])

                        inner, wrap, _ = self.parse_type()
                        field.type = wrap(field.type)
                        if isinstance(field.type, ArrayType):
                            # Array types in function parameters are treated as pointers
                            field.type = PointerType(field.type.target_type)

                        field.name = inner

                        fields.append(field)

                        if not self.accept("COMMA"):
                            self.expect("RPAREN", CompileError(f"Unclosed func args parenthesis", self.peek(-1).span))
                            break


        arr_stack = []
        while self.accept("LBRACKET"):
            if self.peek()[0] == "INT_LITERAL":
                arr_stack.append(int(self.consume()[1]))
            elif self.peek()[0] == "RBRACKET":
                arr_stack.append(None)
            else:
                raise CompileError(f"Non constant array size", self.peek().span)

            self.expect("RBRACKET", CompileError(f"Unclosed array length", self.peek(-1).span))

        func_stack = []
        while self.accept("LPAREN"):
            arg_types = []
            while self.peek()[0] != "RPAREN":
                base = PrimitiveType(self.consume()[1])
                _, wrap, _ = self.parse_type(False)
                arg_types.append(wrap(base))

                if self.peek()[0] != "COMMA":
                    break

            self.expect("RPAREN")

            func_stack.append(arg_types)



        def wrap(atom):
            for _ in range(ptr_count):
                atom = PointerType(atom)

            for arg_types in func_stack:
                atom = FunctionType(atom, arg_types)

            for s in reversed(arr_stack):
                atom = ArrayType(atom)
                atom.length = s

            if inner_wrap is not None:
                atom = inner_wrap(atom)

            return atom

        return name, wrap, fields

    def parse_type_cast(self) -> TypeCastNode:
        # Type cast
        self.expect("LPAREN")
        base = PrimitiveType(self.consume()[1])

        _, wrap, _ = self.parse_type(get_name=False)
        new_type = wrap(base)

        self.expect("RPAREN")

        if self.peek()[0] == "LPAREN":
            self.expect("LPAREN")
            expr = self.parse_expression()
            self.expect("RPAREN")
        else:
            expr = self.parse_operand()

        node = TypeCastNode()
        node.new_type = new_type
        node.expression = expr

        return node

    def has_inner_name(self, start: int = 0) -> bool:
        i = start

        # Check for this in {this} *something[]
        if self.peek(i)[0] != "IDENTIFIER":
            return False

        i += 1

        while self.peek(i)[0] not in ["COMMA", "RPAREN"]:
            if self.peek(i)[0] == "IDENTIFIER":
                return True

            i += 1

        return False

    def is_type_start(self, offset=0):
        kind, text = self.peek(offset)
        return kind == "IDENTIFIER" and text in self.type_names
