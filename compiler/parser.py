from ast_nodes import *

builtin_types = [
    "BOOL",
    "INT",
    "CHAR",
]

builtin_type_literals = [
    "BOOL_LITERAL",
    "INT_LITERAL",
    "CHAR_LITERAL",
]

operations_map = {
    "+":  "add",
    "-":  "sub",
    "*":  "mul",
    "/":  "div",
    "%":  "mod",
    "&":  "and",
    "|":  "or",
    "^":  "xor",
    "<<": "shl",
    ">>": "shr",
    "==": "eq",
    "!=": "neq",
    "<":  "lt",
    ">":  "gt",
    "<=": "lte",
    ">=": "gte",
    "&&": "logical_and",
    "||": "logical_or",
}

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
            raise SyntaxError(f"Expected {kind}, but found {self.tokens[self.position][1]}")

        self.position += 1

    def parse_program(self):
        while self.position < len(self.tokens):
            self.ast.body.nodes.append(self.parse_statement())


    def parse_statement(self) -> AstNode:
        token = self.peek()

        node = AstNode()

        if token[0] in builtin_types or token[0] == "IDENTIFIER":
            # Variable decl, func decl or variable assignment
            if self.peek(1)[0] == "IDENTIFIER":
                # Variable decl or function decl
                if self.peek(2)[0] == "EQUALS" or self.peek(2)[0] == "SEMICOLON":
                    # Variable decl
                    node = self.parse_variable_decl()

                elif self.peek(2)[0] == "LPAREN":
                    # Func decl
                    node = self.parse_function_declaration()

                else:
                    raise SyntaxError(f"Couldn't parse token: {token[1]}")

            elif self.peek(1)[0] == "EQUALS":
                # Variable assignment
                node = self.parse_variable_assignment()

            elif self.peek(1)[0] == "LPAREN":
                # Function call
                node = self.parse_function_call()

            else:
                raise SyntaxError(f"Couldn't parse token: {token[1]}")

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

        else:
            raise SyntaxError(f"Couldn't parse token: {token[1]}")

        return node


    def parse_for(self) -> AstNode:
        node = ForNode()
        self.expect("FOR")

        self.expect("LPAREN")
        node.init_expr = self.parse_variable_decl()
        self.expect("SEMICOLON")
        node.condition = self.parse_expression()
        self.expect("SEMICOLON")
        node.update_expr = self.parse_expression()
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

        node.function_name = self.consume()[1]

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
            field.name = self.consume()[1]
            self.expect("COLON")

            field.type = self.consume()[1]

            if self.peek()[0] == "COMMA":
                self.expect("COMMA")

            node.fields.append(field)

        self.expect("RBRACE")

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

        return node

    def parse_variable_assignment(self) -> AstNode:
        node = AssignmentNode()
        node.target = IdentifierNode(self.consume()[1])
        self.expect("EQUALS")
        node.expression = self.parse_expression()

        self.expect("SEMICOLON")

        return node



    def parse_variable_decl(self) -> VariableDeclNode:
        node = VariableDeclNode()

        node.type = self.consume()[1]

        node.name = self.consume()[1]

        if self.peek()[0] == "SEMICOLON":
            self.expect("SEMICOLON")
            return node

        self.expect("EQUALS")

        node.init_value = self.parse_expression()

        self.expect("SEMICOLON")

        return node


    def parse_function_declaration(self) -> AstNode:
        node = FunctionDeclNode()

        node.type = self.consume()[1]

        node.name = self.consume()[1]

        self.expect("LPAREN")

        # Handle parameters
        while self.peek()[0] != "RPAREN":
            field = FieldNode()
            field.type = self.consume()[1]

            field.name = self.consume()[1]

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
        left = AstNode()

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


        if self.peek()[1] in operations_map:
            operation = self.consume()[1]
        else:
            raise SyntaxError(f"Couldn't parse operation: {self.peek()[0]}")

        right = AstNode()

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
        if self.peek()[0] == "IDENTIFIER":
            # Var or function, just guess var for now
            node = IdentifierNode()
            node.name = self.consume()[1]
            return node

        elif self.peek()[0] in builtin_type_literals:
            # Number
            node = NumberNode()
            if self.peek()[0] == "INT_LITERAL":
                node.value = int(self.consume()[1])
            elif self.peek()[0] == "CHAR_LITERAL":
                node.value = ord(self.consume()[1])
            else:
                raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]}")
            return node

        elif self.peek()[0] == "STAR":
            # Dereference
            node = DereferenceNode()
            self.expect("STAR")

            if self.peek()[0] != "LPAREN":
                # Just a primary expression
                node.location = self.parse_primary_expression()
            else:
                # A whole expression
                self.expect("LPAREN")
                node.location = self.parse_expression()
                self.expect("RPAREN")

            return node

        else:
            raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]}")