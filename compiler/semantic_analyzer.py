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

        self.verbose: bool = False


    def analyze(self):
        print_width = 40
        # Type table builder:
        # Builds the type table, finding struct definitions and adding them as types
        type_table_builder = TypeTableBuilder()
        type_table_builder.ast = self.ast
        type_table_builder.build_type_table()
        self.type_table = type_table_builder.type_table


        if self.verbose:
            print(f"{"=" * print_width} TYPE TABLE {"=" * print_width}")
            # Print type table
            for var_type, value in self.type_table.items():
                print(f"{var_type}:")
                print(f"\tname: {value.name}")
                print(f"\tsize: {value.size}")
                print(f"\tfields:")
                for field_name, field in value.fields.items():
                    print(f"\t\t{field_name}: {field.type}, {field.offset}")
            print()
            print()


        # Separate declarations:
        # Remove struct declarations
        # Make sure only func and var decl are in global scope
        self.separate_declarations()


        # Scope builder:
        # Builds the frames of functions, giving addresses to variables
        scope_builder = ScopeBuilder()
        scope_builder.ast = self.ast
        scope_builder.type_table = self.type_table
        scope_builder.build_scope_stack()
        self.global_frame = scope_builder.global_frame


        if self.verbose:
            print(f"{"=" * print_width} GLOBAL FRAME {"=" * print_width}")
            print(self.global_frame.name + ":")
            print_frame(self.global_frame, 1)
            print()
            print()


        # Name resolver:
        # Finds all Identifier nodes, finds their symbol and assigns their symbol property to said symbol.
        # Finds all function calls and sets their frames.
        name_resolver = NameResolver()
        name_resolver.ast = self.ast
        name_resolver.global_frame = self.global_frame
        name_resolver.resolve_all_names()
        self.ast = name_resolver.ast


        # Type resolver:
        # Finds all assignments and declarations, and finds then sets the types of the expressions used in them
        type_resolver = TypeResolver()
        type_resolver.ast = self.ast
        type_resolver.type_table = self.type_table
        type_resolver.resolve_types()
        self.ast = type_resolver.ast

        if self.verbose:
            print(f"{"=" * print_width} TYPE SET FRAME {"=" * print_width}")
            print(self.ast)
            print()
            print()

        # Semantic checker:
        # Checks that the program is semantically correct, like variables being used after declaration etc
        semantic_checker = SemanticChecker()
        semantic_checker.ast = self.ast
        semantic_checker.type_table = self.type_table
        semantic_checker.check_semantics()


    def separate_declarations(self):
        """Filter out variable declarations and struct declarations from the ast"""

        ast = ProgramNode()

        for node in self.ast.body.nodes:
            if isinstance(node, StructDeclNode):
                # Struct declarations no longer needed
                continue

            elif isinstance(node, VariableDeclNode):
                ast.body.nodes.append(node)

            elif isinstance(node, FunctionDeclNode):
                ast.body.nodes.append(node)

            elif isinstance(node, AssemblyBlockNode):
                ast.body.nodes.append(node)

            else:
                raise SyntaxError(f"Only struct and variable declarations are supported in global scope: {node}")

        self.ast = ast



def print_frame(frame: Frame, indent: int = 0):
    for name, var in frame.symbol_table.symbols.items():
        if var.is_global:
            print(f"{"\t" * indent}{repr(var.type)} {name} label: {var.label}")
        else:
            print(f"{"\t"*indent}{repr(var.type)} {name} @{var.offset}")

    print()

    for frame in frame.children:
        print(f"{"\t"*indent}frame: {frame.name} with size: {frame.size}:")

        print_frame(frame, indent + 1)