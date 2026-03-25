from ..ast_nodes import *


class SemanticChecker:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}


    def check_semantics(self):
        self.check_body_semantics(self.ast.body)


    def check_body_semantics(self, node: BodyNode):
        for node in node.nodes:
            if isinstance(node, IfNode):
                self.check_body_semantics(node.body)

            elif isinstance(node, WhileNode):
                self.check_body_semantics(node.body)

            elif isinstance(node, ForNode):
                self.check_body_semantics(node.body)

            elif isinstance(node, FunctionDeclNode):
                self.check_body_semantics(node.body)
                if len(node.body.nodes) > 0 or not isinstance(node.body.nodes[-1], ReturnNode):
                    node.body.nodes.append(ReturnNode())