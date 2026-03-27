import ast

from ..ast_nodes import *

class NameResolver:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.global_frame: Frame = Frame()

        self.curr_frame: Frame = Frame()


    def resolve_all_names(self):
        self.resolve_body_names(self.ast.body)


    def resolve_body_names(self, body: BodyNode):
        self.curr_frame = body.frame

        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                self.resolve_body_names(node.body)

            elif isinstance(node, IfNode):
                self.resolve_expression_names(node.condition)
                self.resolve_body_names(node.body)
                self.resolve_body_names(node.else_node.body)

            elif isinstance(node, WhileNode):
                self.resolve_expression_names(node.condition)
                self.resolve_body_names(node.body)

            elif isinstance(node, ForNode):
                self.resolve_expression_names(node.init_expr)
                self.resolve_expression_names(node.condition)
                self.resolve_expression_names(node.update_expr)
                self.resolve_body_names(node.body)

            elif isinstance(node, VariableDeclNode):
                if node.init_value:
                    self.resolve_expression_names(node.init_value)

            elif isinstance(node, AssignmentNode):
                self.resolve_expression_names(node.target)
                self.resolve_expression_names(node.expression)

            elif isinstance(node, FunctionCallNode):
                self.resolve_expression_names(node)



    def resolve_expression_names(self, node: AstNode):
        """Recursively finds and sets variable symbols and function frames"""

        if isinstance(node, IdentifierNode):
            frame = self.curr_frame.lookup_symbol(node.name)
            symbol = frame.symbol_table.lookup_symbol(node.name)
            node.symbol = symbol

        elif isinstance(node, IndexExpressionNode):
            self.resolve_expression_names(node.base)
            self.resolve_expression_names(node.index)

        elif isinstance(node, FunctionCallNode):
            for frame in self.global_frame.children:
                if not frame.return_type:
                    continue

                if frame.name == node.func_name:
                    node.func_frame = frame

            if node.func_frame is None:
                raise NameError(f"Failed to find function: {node.func_name}  : {node}")

            for arg in node.args:
                self.resolve_expression_names(arg)

        elif isinstance(node, MemberAccessNode):
            self.resolve_expression_names(node.variable)

        elif isinstance(node, BinaryOpNode):
            self.resolve_expression_names(node.left)
            self.resolve_expression_names(node.right)

        elif isinstance(node, UnaryOpNode):
            self.resolve_expression_names(node.right)

        elif isinstance(node, DereferenceNode):
            self.resolve_expression_names(node.address_expression)

        elif isinstance(node, ValueNode):
            # These don't need to be handled but shouldn't crash
            pass

        else:
            raise SyntaxError(f"Unexpected node type {node}")