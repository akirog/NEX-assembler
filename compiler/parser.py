from operator import truediv

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

            elif self.peek(1)[0] == "STAR":
                # Also variable declaration
                node = self.parse_variable_decl()

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
            self.expect("BREAK")
            self.expect("SEMICOLON")

        elif self.peek()[0] == "CONTINUE":
            node = ContinueNode()
            self.expect("CONTINUE")
            self.expect("SEMICOLON")

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
        node.condition = self.parse_expression()
        self.expect("SEMICOLON")
        node.update_expr = self.parse_variable_assignment(False)
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

        node.func = IdentifierNode(self.consume()[1])

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

            node.type = ArrayType(node.type)

            # If no length we just set length from array length
            if self.peek()[0] != "RBRACKET":
                node.type.length = eval(self.consume()[1])

            self.expect("RBRACKET")


        if self.peek()[0] == "SEMICOLON":
            self.expect("SEMICOLON")
            return node

        self.expect("EQUALS")

        # Parse normal expression
        node.init_value = self.parse_expression()

        if isinstance(node.type, ArrayType) and node.type.length is None:
            if not isinstance(node.init_value, StringLiteralNode):
                raise SyntaxError(f"Unable to get static length of array type: {node}")

            node.type.length = len(node.init_value.literal)

        self.expect("SEMICOLON")

        return node

    def parse_array_literal(self) -> ArrayLiteralNode:
        """Parses an array literal like { 0, 7, 2 }"""
        node = ArrayLiteralNode()
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

        if self.peek()[0] == "LBRACE":
            # Struct initiation
            self.expect("LBRACE")
            node = StructInitNode()

            while self.peek()[0] != "RBRACE":
                node.args.append(self.parse_expression())

                if self.peek()[0] == "COMMA":
                    self.expect("COMMA")
                else:
                    break

            self.expect("RBRACE")
            return node


        if self.peek()[1] in unary_ops:
            op = self.consume()[1]
            left = UnaryOpNode()
            left.operation = op

            if self.peek()[0] == "LPAREN":
                self.expect("LPAREN")
                left.right = self.parse_expression()
                self.expect("RPAREN")

            else:
                left.right = self.parse_primary_expression()

        elif self.peek()[0] == "STAR":
            self.expect("STAR")

            if self.peek()[0] == "LPAREN":
                self.expect("LPAREN")

                left: DereferenceNode = DereferenceNode()
                left.address_expression = self.parse_expression()

                self.expect("RPAREN")

            else:
                left: DereferenceNode = DereferenceNode()
                left.address_expression = self.parse_primary_expression()



        elif self.peek()[0] == "LPAREN":
            is_typecast = True

            if self.peek(1)[0] == "IDENTIFIER":
                i = 2
                while is_typecast:
                    if self.peek(i)[0] == "RPAREN":
                        break
                    elif self.peek(i)[0] != "STAR":
                        is_typecast = False

                    i += 1
            else:
                is_typecast = False

            if is_typecast:
                left = self.parse_type_cast()

            else:
                # Normal parenthesis expression
                self.expect("LPAREN")
                left = self.parse_expression()
                self.expect("RPAREN")
        else:
            left = self.parse_primary_expression()


        expression_enders = ["RPAREN", "SEMICOLON", "COMMA", "RBRACKET", "RBRACE", "EQUALS"]
        if self.peek()[0] in expression_enders:
            return left


        if self.peek()[1] in operations:
            operation = self.consume()[1]
        else:
            raise SyntaxError(f"Couldn't parse operation: {self.peek()}")

        right: AstNode

        if self.peek()[0] == "LPAREN":
            if self.peek(1)[0] == "IDENTIFIER" and self.peek(2)[0] == "RPAREN":
                right = self.parse_type_cast()

            else:
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
        """Parses a primary expression like x, 5"""
        unary_op = None

        # Consume unary op
        while self.peek()[1] in unary_ops:
            if unary_op is None:
                unary_op = UnaryOpNode()
                unary_op.operation = self.consume()[1]
            else:
                unary_op.right = UnaryOpNode()
                unary_op.right.operation = self.consume()[1]


        # Consume atom
        atom: AstNode
        if self.peek()[0] != "IDENTIFIER" and self.peek()[0] not in builtin_type_literals and self.peek()[0] != "AND":
            raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]} {self.peek(1)[1]}")

        if self.peek()[0] == "IDENTIFIER":
            atom = IdentifierNode(self.consume()[1])

        elif self.peek()[0] in builtin_type_literals:
            atom = ValueNode()
            if self.peek()[0] == "INT_LITERAL":
                atom.value = eval(self.consume()[1])
                atom.type = PrimitiveType("int")
            elif self.peek()[0] == "CHAR_LITERAL":
                atom.value = ord(self.consume()[1].strip("'"))
                atom.type = PrimitiveType("char")
            elif self.peek()[0] == "BOOL_LITERAL":
                atom.value = 1 if self.consume()[1] == "true" else 0
                atom.type = PrimitiveType("bool")
            elif self.peek()[0] == "STRING_LITERAL":
                atom = StringLiteralNode()
                atom.literal = self.consume()[1].strip('"') + "\0"

            else:
                raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]}")
        elif self.peek()[0] == "AND":
            self.expect("AND")
            var = IdentifierNode(self.consume()[1])
            atom = AddressOfNode()
            atom.variable = var

        else:
            raise SyntaxError(f"Couldn't parse primary expression: {self.peek()[0]}")



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

            else:
                break


        inner_unary: None | UnaryOpNode = unary_op

        while inner_unary is not None and inner_unary.right is not None and isinstance(inner_unary.right, UnaryOpNode):
            inner_unary = inner_unary.right

        if inner_unary is not None:
            inner_unary.right = atom
            atom = inner_unary

        return atom


    def parse_type_cast(self) -> TypeCastNode:
        # Type cast
        self.expect("LPAREN")
        new_type = PrimitiveType(self.consume()[1])

        while self.peek()[0] == "STAR":
            self.expect("STAR")
            new_type = PointerType(new_type)

        self.expect("RPAREN")

        if self.peek()[0] == "LPAREN":
            self.expect("LPAREN")
            expr = self.parse_expression()
            self.expect("RPAREN")
        else:
            expr = self.parse_primary_expression()

        right = TypeCastNode()
        right.new_type = new_type
        right.expression = expr

        return right