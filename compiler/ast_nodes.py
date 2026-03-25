from .frame_classes import *


class AstNode:
    pass

    def __repr__(self):
        return f"Undefined AST node"

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
        return f"if ({self.condition}) {{\n{self.body}\n}}" + (
            f" else {{\n{self.else_node}\n}}" if self.else_node else "")


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

    def __repr__(self):
        return f"for ({self.init_expr}; {self.condition}; {self.update_expr}) {{\n{self.body}\n}}"


# Declaration
class VariableDeclNode(AstNode):
    def __init__(self):
        self.type: TypeNode = TypeNode()
        self.name: str = ""
        self.init_value: AstNode | None = None

    def __repr__(self):
        return f"{self.type} {self.name}" + (f" = {self.init_value}" if self.init_value else "")


class FunctionDeclNode(AstNode):
    def __init__(self):
        self.name: str = ""
        self.type: TypeNode = TypeNode()
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


# Expressions
class ValueNode(AstNode):
    def __init__(self, value: int = 0):
        self.value: int = value
        self.type: TypeNode = TypeNode()

    def __repr__(self):
        return f"{self.value}"


class IdentifierNode(AstNode):
    def __init__(self, name: str = ""):
        self.name = name
        self.type: TypeNode = TypeNode()

    def __repr__(self):
        return f"{self.name}"


class IndexExpressionNode(AstNode):
    def __init__(self):
        self.base: AstNode = AstNode()
        self.index: AstNode = AstNode()
        self.type: TypeNode = TypeNode()


class MemberAccessNode(AstNode):
    def __init__(self):
        self.variable: AstNode = AstNode()
        self.member: str = ""
        self.member_type: TypeNode = TypeNode()

    def __repr__(self):
        return f"{self.variable}.{self.member}"


class FunctionCallNode(AstNode):
    def __init__(self):
        self.func_name: str = ""
        self.args: list[AstNode] = []
        self.ret_type: TypeNode = TypeNode()

    def __repr__(self):
        return f"{self.func_name}({', '.join(repr(arg) for arg in self.args)})"


class DereferenceNode(AstNode):
    def __init__(self):
        self.address_expression: AstNode = AstNode()
        self.pointee_type: TypeNode = TypeNode()

    def __repr__(self):
        return f"(*{self.address_expression})"

# Operations
class BinaryOpNode(AstNode):
    def __init__(self):
        self.left: AstNode = AstNode()
        self.right: AstNode = AstNode()
        self.operation: str = ""
        self.type: TypeNode = TypeNode()

    def __repr__(self):
        return f"({self.left} {self.operation} {self.right})"


class UnaryOpNode(AstNode):
    def __init__(self):
        self.right: AstNode = AstNode()
        self.operation: str = ""
        self.type: TypeNode = TypeNode()

    def __repr__(self):
        return f"({self.operation}{self.right})"


class AddressOfNode(AstNode):
    def __init__(self):
        self.variable: AstNode = AstNode()

    def __repr__(self):
        return f"(&{repr(self.variable)})"


# Control flow
class ReturnNode(AstNode):
    def __init__(self):
        self.ret_expr: AstNode | None = None
        self.ret_type: TypeNode = TypeNode()

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
        self.type: TypeNode = TypeNode()
        self.name: str = ""

    def __repr__(self):
        return f"{self.name}: {self.type}"


class ArrayLiteralNode(AstNode):
    def __init__(self):
        self.elements: list[AstNode] = []
        self.length: int = 0

    def __repr__(self):
        return f"[{', '.join(repr(element) for element in self.elements)}]"


class AssemblyBlockNode(AstNode):
    def __init__(self):
        self.assembly: list[str] = []

    def __repr__(self):
        return f"asm {{{'\n'.join(self.assembly)}}}"
