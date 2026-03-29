from asyncio.tools import NodeType

from ..ast_nodes import *



class TypeResolver:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}

    def resolve_types(self):
        self.resolve_body_types(self.ast.body)


    def resolve_body_types(self, body: BodyNode):
        """Set the types of variables, binary operations etc"""
        for node in body.nodes:
            self.resolve_node_types(node)


    def resolve_node_types(self, node: AstNode):
        if isinstance(node, FunctionDeclNode):
            self.resolve_body_types(node.body)

        elif isinstance(node, IfNode):
            self.resolve_body_types(node.body)
            self.get_type(node.condition)

            if node.else_node:
                self.resolve_node_types(node.else_node)

        elif isinstance(node, WhileNode):
            self.get_type(node.condition)
            self.resolve_body_types(node.body)

        elif isinstance(node, ForNode):
            self.resolve_body_types(node.body)

        elif isinstance(node, VariableDeclNode):
            if node.init_value:
                self.get_type(node.init_value)

        elif isinstance(node, AssignmentNode):
            target_type = self.get_type(node.target)
            value_type = self.get_type(node.expression)

            if target_type.get_type() != value_type.get_type():
                raise SyntaxError(f"Cannot perform assignment of {node.target} to {node.expression}")

            node.type = target_type

        elif isinstance(node, FunctionCallNode):
            for arg in node.args:
                self.get_type(arg)

            self.get_type(node)

        elif isinstance(node, AssemblyBlockNode):
            # Nothing to resolve, at least not yet
            pass

        else:
            print(f"Unexpected node type {node}")



    def get_type(self, node: AstNode) -> TypeNode:
        """Sets the type of the node, returns the type it was set to"""

        if isinstance(node, IdentifierNode):
            node.type = node.symbol.type
            return node.symbol.type

        elif isinstance(node, IndexExpressionNode):
            node.pointee_type = self.get_type(node.base).dereference()
            self.get_type(node.index)
            return node.pointee_type

        elif isinstance(node, FunctionCallNode):
            node.type = node.func_frame.return_type
            return node.type

        elif isinstance(node, MemberAccessNode):
            base_type = self.get_type(node.variable)
            if not isinstance(base_type, PrimitiveType):
                raise SyntaxError(f"Unexpected type {base_type} for variable {node.variable}")

            fields = self.type_table.get(base_type.type).fields
            if node.member not in fields:
                raise SyntaxError(f"Cannot get field: {node.member} from type {base_type.type}, it does not contain this field")

            node.type = fields[node.member].type
            return node.type


        elif isinstance(node, BinaryOpNode):
            left_type = self.get_type(node.left)
            right_type = self.get_type(node.right)

            if left_type.get_type != left_type.get_type:
                raise SyntaxError(f"Cannot use operation on variables of different types without casting: {left_type} and {right_type}")

            node.type = left_type
            return node.type

        elif isinstance(node, UnaryOpNode):
            type = self.get_type(node.right)

            node.type = type
            return node.type

        elif isinstance(node, DereferenceNode):
            type = self.get_type(node.address_expression)

            if not isinstance(type, PointerType):
                raise SyntaxError(f"Cannot dereference type {type}")

            type = type.target

            node.pointee_type = type
            return type

        elif isinstance(node, ValueNode):
            return node.type

        elif isinstance(node, UnaryOpNode):
            node.type = self.get_type(node.right)
            return node.type

        else:
            raise SyntaxError(f"Unexpected node type {node}")