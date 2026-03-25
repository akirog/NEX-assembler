from ..ast_nodes import *



class TypeResolver:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}

    def resolve_types(self):
        self.resolve_body_types(BodyNode())


    def resolve_body_types(self, body: BodyNode):
        """Set the types of variables, binary operations etc"""
        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                self.resolve_body_types(node.body)

            elif isinstance(node, IfNode):
                self.resolve_body_types(node.body)

            elif isinstance(node, WhileNode):
                self.resolve_body_types(node.body)

            elif isinstance(node, ForNode):
                self.resolve_body_types(node.body)

            elif isinstance(node, VariableDeclNode):
                if node.init_value:
                    self.get_type(node.init_value)

            elif isinstance(node, AssignmentNode):
                self.get_type(node.target)
                self.get_type(node.expression)

            elif isinstance(node, FunctionCallNode):
                self.get_type(node)

            else:
                print(f"did not match: {node}")



    def get_type(self, node: AstNode) -> TypeNode:
        """Sets the type of the node, returns the type it was set to"""

        if isinstance(node, IdentifierNode):
            print(node.symbol.type + "======================================================================================")
            node.type = node.symbol.type
            return node.symbol.type

        elif isinstance(node, IndexExpressionNode):
            node.type = self.get_type(node.base)
            print(node.type + " : " + node.base)
            return node.type

        elif isinstance(node, FunctionCallNode):
            for frame in self.global_frame.children:
                if not frame.return_type:
                    continue

                if frame.name == node.func_name:
                    node.type = frame.return_type
                    return node.type

            raise SyntaxError(f"Function call frame not found, could not assign return type")

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

            if left_type != right_type:
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

        else:
            raise SyntaxError(f"Unexpected node type {node}")