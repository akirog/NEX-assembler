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
            self.resolve_node_types(node.init_expr)
            self.get_type(node.condition)
            self.resolve_node_types(node.update_expr)
            self.resolve_body_types(node.body)

        elif isinstance(node, VariableDeclNode):
            if node.init_value:
                if isinstance(node.init_value, StringLiteralNode):
                    node.init_value.type = node.type
                elif isinstance(node.init_value, ArrayLiteralNode):
                    node.init_value.type = node.type
                elif isinstance(node.init_value, StructInitNode):
                    node.init_value.type = node.type

                self.get_type(node.init_value)

        elif isinstance(node, AssignmentNode):
            target_type = self.get_type(node.target)
            value_type = self.get_type(node.expression)

            if target_type.get_type() != value_type.get_type():
                raise SyntaxError(f"Cannot perform assignment of {node.target} to {node.expression}")

            node.type = target_type

        elif isinstance(node, FunctionCallNode):
            self.get_type(node)

        elif isinstance(node, ReturnNode):
            if node.ret_expr is not None:
                node.ret_type = self.get_type(node.ret_expr)
                if node.ret_type.get_type() != node.func_frame.return_type.get_type():
                    raise SyntaxError(f"Return type does not match function return type")

            else:
                # Ret expr is none
                if not node.func_frame.is_interrupt and node.func_frame.return_type.get_type() != "void":
                    print(f"WARNING: function has no return expression even tho function has return type")

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
            # Resolve arguments
            for arg in node.args:
                self.get_type(arg)

            if isinstance(node.func, IdentifierNode) and node.func.name == node.func_name:
                node.func.type = PrimitiveType("void")
            else:
                self.get_type(node.func)

            # Resolve return type
            node.ret_type = node.func_frame.return_type
            return node.ret_type

        elif isinstance(node, MemberAccessNode):
            base_type = self.get_type(node.variable)
            if not isinstance(base_type, PrimitiveType):
                raise SyntaxError(f"Unexpected type {base_type} for variable {node.variable}")

            node.base_type = base_type
            fields = self.type_table.get(base_type.type).fields
            if node.member not in fields:
                raise SyntaxError(f"Cannot get field: {node.member} from type {base_type.type}, it does not contain this field")

            node.member_type = fields[node.member].type
            return node.member_type


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
                raise SyntaxError(f"Cannot dereference type {type} from node {node}")

            type = type.target

            node.pointee_type = type
            return type

        elif isinstance(node, ValueNode):
            return node.type

        elif isinstance(node, ArrayLiteralNode):

            if len(node.elements) == 0:
                raise SyntaxError(f"Cannot declare array with empty elements")

            first_type = self.get_type(node.elements[0])

            for element in node.elements[1:]:
                this_type = self.get_type(element)
                if this_type.get_type() != first_type.get_type():
                    raise SyntaxError(f"Cannot declare array with different types: {this_type} and {first_type}")

            node.type = PointerType(first_type)

            return first_type

        elif isinstance(node, StructInitNode):
            self_fields: dict[str, TypeField] = self.type_table.get(node.type.get_type()).fields

            if len(self_fields) != len(node.args):
                raise SyntaxError(f"Missing fields in struct initiation")

            for i, arg in enumerate(node.args):
                if isinstance(arg, StructInitNode):
                    arg.type = PrimitiveType(list(self_fields.keys())[i])

                self.get_type(arg)

            return node.type

        elif isinstance(node, StringLiteralNode):
            node.type = PointerType(PrimitiveType("char"))
            return PointerType(PrimitiveType("char"))


        elif isinstance(node, UnaryOpNode):
            node.type = self.get_type(node.right)
            return node.type

        elif isinstance(node, AddressOfNode):
            return self.get_type(node.variable)

        elif isinstance(node, TypeCastNode):
            self.get_type(node.expression)
            return node.new_type

        else:
            raise SyntaxError(f"Unexpected node type {node}")