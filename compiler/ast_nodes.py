from frame_classes import *


class AstNode:
    pass

# Container nodes
class ProgramNode(AstNode):
    def __init__(self):
        self.body: BodyNode = BodyNode()

    def __repr__(self):
        return repr(self.body)


class BodyNode(AstNode):
    def __init__(self):
        self.nodes: list[AstNode] = []
        self.frame: Frame | None = None

    def __repr__(self):
        return '\n'.join(repr(node) for node in self.nodes)

class IfNode(AstNode):
    def __init__(self):
        self.condition: AstNode = AstNode()
        self.body: BodyNode = BodyNode()
        self.else_node: IfNode | None = None

    def __repr__(self):
        return f"if ({self.condition}) {{\n{self.body}\n}}" + (f" else {{\n{self.else_node}\n}}" if self.else_node else "")

class WhileNode(AstNode):
    def __init__(self):
        self.condition: AstNode = AstNode()
        self.body: BodyNode = BodyNode()

    def __repr__(self):
        return f"while ({self.condition}) {{\n{self.body}\n}}"

class ForNode(AstNode):
    def __init__(self):
        self.init_expr: VariableDeclNode = VariableDeclNode()
        self.condition: AstNode = AstNode()
        self.update_expr: AssignmentNode = AssignmentNode()
        self.body: BodyNode = BodyNode()

# Declaration
class VariableDeclNode(AstNode):
    def __init__(self):
        self.type: str = ""
        self.name: str = ""
        self.init_value: AstNode | None = None

    def __repr__(self):
        return f"{self.type} {self.name}" + (f" = {self.init_value}" if self.init_value else "")

class FunctionDeclNode(AstNode):
    def __init__(self):
        self.name: str = ""
        self.type: str = ""
        self.args: list[FieldNode] = []
        self.body: BodyNode = BodyNode()

    def __repr__(self):
        return f"{self.type} {self.name}({self.args}) {{\n{self.body}\n}}"

class StructDeclNode(AstNode):
    def __init__(self):
        self.name: str = ""
        self.fields: list[FieldNode] = []

    def __repr__(self):
        return f"struct {self.name} {{\n{'\n'.join(repr(field) for field in self.fields)}\n}}"

class AssignmentNode(AstNode):
    def __init__(self):
        self.target: AstNode = AstNode()
        self.expression: AstNode = AstNode()

    def __repr__(self):
        return f"{self.target} = {self.expression}"

class NumberNode(AstNode):
    def __init__(self):
        self.value: int = 0

    def __repr__(self):
        return f"{self.value}"

class IdentifierNode(AstNode):
    def __init__(self, name: str = ""):
        self.name = name

    def __repr__(self):
        return f"{self.name}"

class MemberAccessNode(AstNode):
    def __init__(self):
        self.variable: AstNode = AstNode()
        self.member: str = ""

    def __repr__(self):
        return f"{self.variable}.{self.member}"


class FunctionCallNode(AstNode):
    def __init__(self):
        self.func_name: str = ""
        self.args: list[AstNode] = []

    def __repr__(self):
        return f"{self.func_name}({', '.join(repr(arg) for arg in self.args)})"

# Operations
class BinaryOpNode(AstNode):
    def __init__(self):
        self.left: AstNode = AstNode()
        self.right: AstNode = AstNode()
        self.operation: str = ""

    def __repr__(self):
        return f"({self.left} {self.operation} {self.right})"

class UnaryOpNode(AstNode):
    def __init__(self):
        self.right: AstNode = AstNode()
        self.operation: str = ""

    def __repr__(self):
        return f"({self.operation}{self.right})"



# Memory stuff
class DereferenceNode(AstNode):
    def __init__(self):
        self.address_expression: AstNode = AstNode()

    def __repr__(self):
        return f"(*{self.address_expression})"

class AddressOfNode(AstNode):
    def __init__(self):
        self.variable: AstNode = AstNode()

    def __repr__(self):
        return f"(&{self.variable})"


# Control flow
class ReturnNode(AstNode):
    def __init__(self):
        self.ret_expr: AstNode | None = None

    def __repr__(self):
        return f"return" + (f" {self.ret_expr}" if self.ret_expr else "")

class BreakNode(AstNode):
    def __init__(self):
        pass

    def __repr__(self):
        return f"break"

class ContinueNode(AstNode):
    def __init__(self):
        pass

    def __repr__(self):
        return f"continue"


# Misc
class FieldNode(AstNode):
    def __init__(self):
        self.type: str = ""
        self.name: str = ""

    def __repr__(self):
        return f"{self.name}: {self.type}"