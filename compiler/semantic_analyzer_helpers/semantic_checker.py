from ..ast_nodes import *


class SemanticChecker:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}
        self.func_use: dict[str, bool] = {}


    def check_semantics(self):
        self.check_body_semantics(self.ast.body)


    def check_body_semantics(self, body: BodyNode):
        for node in body.nodes:
            self.check_statement_semantics(node)

        # Delete functions not used
        # This cant actually be used as of now sine the kernel has uncalled functions called in assembly, might try to find a fix for that later.


        # new_nodes = []
        # for node in body.nodes:
        #     if isinstance(node, FunctionDeclNode) and self.func_use[node.name] == False:
        #         continue
        #
        #     new_nodes.append(node)
        #
        # body.nodes = new_nodes


    def check_statement_semantics(self, node: AstNode):
        if isinstance(node, IfNode):
            if node.condition:
                self.check_expression_semantics(node.condition)

            self.check_body_semantics(node.body)
            if node.else_node:
                self.check_statement_semantics(node.else_node)

        elif isinstance(node, WhileNode):
            self.check_expression_semantics(node.condition)
            self.check_body_semantics(node.body)

        elif isinstance(node, ForNode):
            self.check_statement_semantics(node.init_expr)
            self.check_expression_semantics(node.condition)
            self.check_statement_semantics(node.update_expr)
            self.check_body_semantics(node.body)

        elif isinstance(node, FunctionDeclNode):
            self.check_body_semantics(node.body)
            self.func_use[node.name] = False if node.name != "main" else True
            if len(node.body.nodes) == 0 or not isinstance(node.body.nodes[-1], ReturnNode):
                node.body.nodes.append(ReturnNode())

        elif isinstance(node, VariableDeclNode):
            if node.init_value:
                self.check_expression_semantics(node.init_value)

        elif isinstance(node, AssignmentNode):
            self.check_expression_semantics(node.expression)

        elif isinstance(node, ReturnNode):
            if node.ret_expr:
                self.check_expression_semantics(node.ret_expr)

        elif isinstance(node, FunctionCallNode):
            for arg in node.args:
                self.check_expression_semantics(arg)

        elif isinstance(node, ContinueNode):
            pass

        elif isinstance(node, BreakNode):
            pass

        elif isinstance(node, AssemblyBlockNode):
            pass

        else:
            raise NotImplementedError(f"Not implemented semantic checking for statement {node} yet")

    def check_expression_semantics(self, node: AstNode):
        if isinstance(node, ValueNode):
            pass

        elif isinstance(node, BinaryOpNode):
            self.check_expression_semantics(node.left)
            self.check_expression_semantics(node.right)

            if (isinstance(node.left, IdentifierNode) and node.left.type.get_size(self.type_table) > 4 or
                isinstance(node.right, IdentifierNode) and node.right.type.get_size(self.type_table) > 4):
                raise NotImplementedError(f"Cannot add two variables of larger size than 4: {node.simple_repr()}")

        elif isinstance(node, UnaryOpNode):
            self.check_expression_semantics(node.right)

        elif isinstance(node, IdentifierNode):
            pass

        elif isinstance(node, IndexExpressionNode):
            pass

        elif isinstance(node, StringLiteralNode):
            pass

        elif isinstance(node, TypeCastNode):
            self.check_expression_semantics(node.expression)

        elif isinstance(node, FunctionCallNode):
            for arg in node.args:
                self.check_expression_semantics(arg)

            if node.func_name in self.func_use:
                self.func_use[node.func_name] = True
            elif node.func:
                self.check_expression_semantics(node.func)
            else:
                raise RuntimeError(f"No valid way to call function call target: {node}")

        elif isinstance(node, AddressOfNode):
            self.check_expression_semantics(node.variable)

        elif isinstance(node, ArrayLiteralNode):
            for element in node.elements:
                self.check_expression_semantics(element)

        elif isinstance(node, DereferenceNode):
            self.check_expression_semantics(node.address_expression)

        elif isinstance(node, StructInitNode):
            for arg in node.args:
                self.check_expression_semantics(arg)

        elif isinstance(node, MemberAccessNode):
            pass

        else:
            raise NotImplementedError(f"Not implemented semantic checking for expression {node} yet")