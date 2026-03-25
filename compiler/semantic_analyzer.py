from .ast_nodes import *
from .frame_classes import *
from .semantic_analyzer_helpers.type_table_builder import *
from .semantic_analyzer_helpers.scope_builder import *
from .semantic_analyzer_helpers.name_resolver import *
from .semantic_analyzer_helpers.type_resolver import *
from .semantic_analyzer_helpers.semantic_checker import *


class SemanticAnalyzer:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}

        self.global_frame: Frame = Frame()
        self.global_vars: list[GlobalVariable] = []


    def analyze(self):
        # Type table builder:
        # Builds the type table, finding struct definitions and adding them as types
        type_table_builder = TypeTableBuilder()
        type_table_builder.ast = self.ast
        type_table_builder.build_type_table()
        self.type_table = type_table_builder.type_table


        # Separate declarations:
        # Separate global variables
        self.separate_declarations()


        # Scope builder:
        # Builds the frames of functions, giving addresses to variables
        scope_builder = ScopeBuilder()
        scope_builder.ast = self.ast
        scope_builder.type_table = self.type_table
        scope_builder.build_scope_stack()
        self.global_frame = scope_builder.global_frame


        # Name resolver:
        # Finds all Identifier nodes, finds their symbol and assigns their symbol property to said symbol.
        # Finds all function calls and sets their frames.
        name_resolver = NameResolver()
        name_resolver.ast = self.ast
        name_resolver.global_frame = self.global_frame
        name_resolver.resolve_names()
        self.ast = name_resolver.ast


        # Type resolver:
        # Finds all assignments and declarations, and finds then sets the types of the expressions used in them
        type_resolver = TypeResolver()
        type_resolver.ast = self.ast
        type_resolver.type_table = self.type_table
        type_resolver.resolve_types()
        self.ast = type_resolver.ast


        # Semantic checker:
        # Checks that the program is semantically correct, like variables being used after declaration etc
        self.check_semantics()


    def separate_declarations(self):
        """Filter out variable declarations and struct declarations from the ast"""

        ast = ProgramNode()

        for node in self.ast.body.nodes:
            if isinstance(node, StructDeclNode):
                # Struct declarations no longer needed
                continue

            elif isinstance(node, VariableDeclNode):
                # Handle global declaration
                var = GlobalVariable()
                var.name = node.name
                var.type = node.type

                if isinstance(node.type, ArrayType):
                    # Array decl
                    if not isinstance(node.init_value, ArrayLiteralNode):
                        raise SyntaxError(f"Only array literals are supported for global array declaration: {node}")

                    for i in range(node.type.length):
                        if i > len(node.init_value.elements):
                            value = node.init_value.elements[0]
                        else:
                            value = node.init_value.elements[i]

                        if not isinstance(value, ValueNode):
                            raise SyntaxError(f"Only constant values are supported for global variables: {node}")

                        var.init_array.append(value.value)

                elif isinstance(node.type, PrimitiveType):
                    # TODO
                    pass

                self.global_vars.append(var)

            elif isinstance(node, FunctionDeclNode):
                ast.body.nodes.append(node)

            else:
                raise SyntaxError(f"Only struct and variable declarations are supported in global scope: {node}")

        self.ast = ast


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