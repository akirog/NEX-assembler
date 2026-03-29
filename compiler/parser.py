from .ast_nodes import *
from .frame_classes import *


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


unary_ops = [
    "!",
    "-"
]


class Parser:
    """Parses a list of tokens to an ast, and checks for syntax errors"""
    def __init__(self):
        self.position: int = 0
        self.tokens: list[tuple[str, str]] = []
        self.ast: ProgramNode = ProgramNode()

    def peek(self, offset: int = 0) -> tuple[str, str]:
        if self.position + offset >= len(self.tokens):
            return "EOF", ""

        return self.tokens[self.position + offset]


    def consume(self) -> tuple[str, str]:
        self.position += 1
        return self.tokens[self.position-1]


    def expect(self, kind: str):
        if self.tokens[self.position][0] != kind:
            raise SyntaxError(f"Expected {kind}, but found {self.tokens[self.position]}")

        self.position += 1

    def parse_program(self):
        while self.position < len(self.tokens):
            self.ast.body.nodes.append(self.parse_statement())


    def parse_statement(self) -> AstNode:
        node: AstNode

        if self.peek()[0] == "IDENTIFIER":
            # Variable decl, func decl or variable assignment
            if self.peek(1)[0] == "IDENTIFIER":
                # Variable decl or function decl
                if (
                        self.peek(2)[0] == "EQUALS" or
                        self.peek(2)[0] == "SEMICOLON" or
                        self.peek(2)[0] == "LBRACKET" or
                        self.peek(2)[0] == "RBRACKET"
                ):
                    # Variable decl
                    node = self.parse_variable_decl()

                elif self.peek(2)[0] == "LPAREN":
                    # Func decl
                    node = self.parse_function_declaration()

                else:
                    raise SyntaxError(f"Couldn't parse token: {self.peek()}")

            elif (self.peek(1)[0] == "EQUALS" or
                    (self.peek(1)[1] in operations and self.peek(2)[0] == "EQUALS") or
                    self.peek(1)[0] == "LBRACKET"
            ):
                # Variable assignment
                node = self.parse_variable_assignment()

            elif self.peek(1)[0] == "LPAREN":
                # Function call
                node = self.parse_function_call()

            elif self.peek(1)[0] == "DOT":
                # Member assignment is also handled by variable assignment function
                node = self.parse_variable_assignment()

            else:
                raise SyntaxError(f"Couldn't parse token: {self.peek()}")

        elif self.peek()[0] == "STAR":
            # Dereference, parse as variable assignment
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

        elif self.peek()[0] == "CONTINUE":
            node = ContinueNode()

        elif self.peek()[0] == "ASM_BLOCK":
            node = AssemblyBlockNode()
            node.assembly = self.consume()[1].split('\n')
            self.expect("SEMICOLON")

        else:
            raise SyntaxError(f"Couldn't parse token: {self.peek()}")

        return node


    def parse_for(self) -> AstNode:
        node = ForNode()
        self.expect("FOR")

        self.expect("LPAREN")
        node.init_expr = self.parse_variable_decl()
        self.expect("SEMICOLON")
        node.condition = self.parse_expression()
        self.expect("SEMICOLON")
        node.update_expr = self.parse_variable_assignment()
        self.expect("RPAREN")

        self.expect("LBRACE")
        node.body = self.parse_body()
        self.expect("RBRACE")
        return node


    def parse_while(self) -> AstNode:
        node = WhileNode()
        self.expect("WHILE")

        self.expect("LPAREN")
        node.condition = self.parse_expression()
        self.expect("RPAREN")

        self.expect("LBRACE")
        node.body = self.parse_body()
        self.expect("RBRACE")
        return node


    def parse_return(self) -> AstNode:
        node = ReturnNode()
        self.expect("RETURN")

        if self.peek()[0] != "SEMICOLON":
            node.ret_expr = self.parse_expression()

        self.expect("SEMICOLON")
        return node


    def parse_function_call(self) -> AstNode:
        node = FunctionCallNode()

        node.func_name = self.consume()[1]

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
        self.expect("SEMICOLON")

        return node

    def parse_struct_definition(self) -> AstNode:
        node = StructDeclNode()
        self.expect("STRUCT")

        node.name = self.consume()[1]

        self.expect("LBRACE")

        # Struct fields parsing is one of a kind so we just do it here
        while self.peek()[0] != "RBRACE":
            field = FieldNode()
            field.type = PrimitiveType(self.consume()[1])

            while self.peek()[0] == "STAR":
                self.expect("STAR")
                field.type = PointerType(field.type)

            field.name = self.consume()[1]

            node.fields.append(field)
            self.expect("SEMICOLON")


        self.expect("RBRACE")
        self.expect("SEMICOLON")
        return node


    def parse_if(self) -> AstNode:
        node = IfNode()
        self.expect("IF")

        self.expect("LPAREN")
        node.condition = self.parse_expression()
        self.expect("RPAREN")

        self.expect("LBRACE")
        node.body = self.parse_body()
        self.expect("RBRACE")

        # Check for else or else if
        if self.peek()[0] != "ELSE":
            return node

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

        self.expect("LBRACE")
        else_node.body = self.parse_body()
        self.expect("RBRACE")

        node.else_node = else_node

        return node

    def parse_variable_assignment(self) -> AssignmentNode:
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

        elif self.peek()[0] == "AMPERSAND":
            # Address of
            self.expect("AMPERSAND")
            # Parse target here since address needs to be of a variable
            node = AddressOfNode()
            node.variable = self.parse_target()
            return node

        elif self.peek()[0] == "IDENTIFIER":
            variable = IdentifierNode(self.consume()[1])

            while self.peek()[0] == "DOT" or self.peek()[0] == "LBRACKET":
                if self.peek()[0] == "DOT":
                    # Member access
                    self.expect("DOT")
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
        else:
            raise SyntaxError("Cant parse target: {self.peek()}")

    def parse_member_access(self, variable: AstNode) -> AstNode:
        """Parses a member access like p.x.y recursively, using the variable from the previous call as variable"""
        member_access = MemberAccessNode()
        member_access.variable = variable
        member_access.member = self.consume()[1]

        if self.peek()[0] == "DOT":
            # Nested member access
            node = self.parse_member_access(variable)
            return node

        else:
            # If we reached the end of the nesting we return the member access
            return member_access



    def parse_variable_decl(self) -> VariableDeclNode:
        node = VariableDeclNode()

        node.type = PrimitiveType(self.consume()[1])

        # Get pointer depth
        while self.peek()[0] == "STAR":
            self.expect("STAR")
            node.type = PointerType(node.type)


        node.name = self.consume()[1]


        while self.peek()[0] == "LBRACKET":
            # Array, expect length of array
            self.expect("LBRACKET")

            node.type = PointerType(node.type)

            # If no length we just set length from array length
            node.type.target_array_length = 0
            if self.peek()[0] != "RBRACKET":
                node.type.target_array_length = int(self.consume()[1])

            self.expect("RBRACKET")


        if self.peek()[0] == "SEMICOLON":
            self.expect("SEMICOLON")
            return node


        self.expect("EQUALS")


        if isinstance(node.type, PointerType) and node.type.target_array_length is not None:
            # Parse array literal
            node.init_value = self.parse_array_literal()

            if not isinstance(node.init_value, ArrayLiteralNode):
                raise SyntaxError("Ayo bruh error!!!")

            if node.type.target_array_length == 0:
                node.type.target_array_length = node.init_value.length

        else:
            # Parse normal expression
            node.init_value = self.parse_expression()

        self.expect("SEMICOLON")

        return node

    def parse_array_literal(self) -> ArrayLiteralNode:
        """Parses an array literal like [0, 7+5, 2], also handles strings and turns them to array literals"""
        node = ArrayLiteralNode()
        if self.peek()[0] == "STRING_LITERAL":
            # String array declaration
            string = self.consume()[1]
            string = string.removesuffix('"').removeprefix('"') # Remove quotes

            for char in string:
                value = ValueNode(ord(char))
                value.type = PrimitiveType("char")
                node.elements.append(value)

            null_terminator = ValueNode(0)
            null_terminator.type = PrimitiveType("char")

            node.elements.append(null_terminator) # Add null terminator

            node.length = len(node.elements)

            return node

        # Normal array declaration
        self.expect("LBRACE")

        while self.peek()[0] != "RBRACE":
            if self.peek()[0] == "LBRACE":
                self.parse_array_literal()

            node.elements.append(self.parse_expression())

            if self.peek()[0] == "COMMA":
                self.expect("COMMA")
            else:
                break

        node.length = len(node.elements)

        self.expect("RBRACE")
        return node

    def parse_function_declaration(self) -> AstNode:
        node = FunctionDeclNode()

        node.type = PrimitiveType(self.consume()[1])

        # Parse pointer things here
        while self.peek()[0] == "STAR":
            self.expect("STAR")
            node.type = PointerType(node.type)

        node.name = self.consume()[1]

        self.expect("LPAREN")

        # Handle parameters
        while self.peek()[0] != "RPAREN":
            field = FieldNode()
            field.type = PrimitiveType(self.consume()[1])

            while self.peek()[0] == "STAR":
                self.expect("STAR")
                field.type = PointerType(field.type)

            field.name = self.consume()[1]

            # Check for array
            while self.peek()[0] == "LBRACKET":
                self.expect("LBRACKET")

                # Nonstatic arrays are parsed as pointers
                field.type = PointerType(field.type)

                self.expect("RBRACKET")

            node.args.append(field)

            if self.peek()[0] == "COMMA":
                self.expect("COMMA")
            else:
                break

        self.expect("RPAREN")
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
        # For now just assemble ast without pemdas
        left: AstNode

        if self.peek()[0] == "LPAREN":
            self.expect("LPAREN")
            left = self.parse_expression()
            self.expect("RPAREN")
        else:
            left = self.parse_primary_expression()


        if self.peek()[0] == "RPAREN":
            return left
        elif self.peek()[0] == "SEMICOLON":
            return left
        elif self.peek()[0] == "COMMA":
            return left
        elif self.peek()[0] == "RBRACKET":
            return left
        elif self.peek()[0] == "EQUALS":
            return left


        if self.peek()[1] in operations:
            operation = self.consume()[1]
        else:
            raise SyntaxError(f"Couldn't parse operation: {self.peek()[0]}")

        right: AstNode

        if self.peek()[0] == "LPAREN":
            self.expect("LPAREN")
            right = self.parse_expression()
            self.expect("RPAREN")
        else:
            right = self.parse_expression()

        node = BinaryOpNode()
        node.left = left
        node.right = right
        node.operation = operation
        return node



    def parse_primary_expression(self) -> AstNode:
        """Parses a primary expression like x, 5, or *x"""
        unary_op = None

        while self.peek()[1] in unary_ops:
            if unary_op is None:
                unary_op = UnaryOpNode()
                unary_op.operation = self.consume()[1]
            else:
                unary_op.right = UnaryOpNode()
                unary_op.right.operation = self.consume()[1]


        node: AstNode

        if self.peek()[0] == "IDENTIFIER":
            name = self.consume()[1]

            if self.peek()[0] == "LPAREN":
                # Function call
                node = FunctionCallNode()
                node.func_name = name

                self.expect("LPAREN")
                while self.peek()[0] != "RPAREN":
                    self.expect("COMMA")
                    node.args.append(self.parse_expression())

                self.expect("RPAREN")

            elif self.peek()[0] == "LBRACKET":
                # Array access
                node = IdentifierNode(name)

                self.expect("LBRACKET")
                array_node = IndexExpressionNode()
                array_node.base = node
                array_node.index = self.parse_expression()
                node = array_node
                self.expect("RBRACKET")

            else:
                # Normal variable
                node = IdentifierNode()
                node.name = name

        elif self.peek()[0] in builtin_type_literals:
            # Number
            node = ValueNode()
            if self.peek()[0] == "INT_LITERAL":
                node.value = int(self.consume()[1])
                node.type = PrimitiveType("int")
            elif self.peek()[0] == "CHAR_LITERAL":
                node.value = ord(self.consume()[1])
                node.type = PrimitiveType("char")
            else:
                raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]}")

        elif self.peek()[0] == "STAR":
            # Dereference
            node = DereferenceNode()
            self.expect("STAR")

            if self.peek()[0] != "LPAREN":
                # Just a primary expression
                node.address_expression = self.parse_primary_expression()
            else:
                # A whole expression
                self.expect("LPAREN")
                node.address_expression = self.parse_expression()
                self.expect("RPAREN")

        elif self.peek()[0] == "AMPERSAND":
            # Address of
            node = AddressOfNode()
            self.expect("AMPERSAND")

            node.variable = self.parse_primary_expression()

        else:
            raise SyntaxError(f"Couldn't parse primary expression: {self.peek()}")


        inner_unary: None | UnaryOpNode = unary_op

        while inner_unary is not None and inner_unary.right is not None and isinstance(inner_unary.right, UnaryOpNode):
            inner_unary = inner_unary.right

        if inner_unary is not None:
            inner_unary.right = node
            node = inner_unary

        return node

