from .ast_nodes import *
from .frame_classes import *

class SemanticAnalyzer:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {
            # Static types
            "int": TypeDefinition("int", 4),
            "char": TypeDefinition("char", 1),
            "bool": TypeDefinition("bool", 1),
            "void": TypeDefinition("void", 0),
        }

        self.global_frame: Frame = Frame()

        self.global_vars: list[GlobalVariable] = []


    def analyze(self):
        self.build_type_table()
        self.separate_declarations()
        self.build_scope_stack()
        self.check_semantics()


    def build_type_table(self):
        """Collect structs and add them to the type_table"""
        for node in self.ast.body.nodes:
            if not isinstance(node, StructDeclNode):
                continue

            new_type = TypeDefinition()
            new_type.name = node.name

            offset = 0
            for field in node.fields:
                new_type.fields[field.name] = TypeField(name=field.name, type=field.type, offset=offset)
                new_type.size += self.type_table[field.type.get_type()].size
                offset += self.type_table[field.type.get_type()].size

            self.type_table[node.name] = new_type


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
                            var.init_array.append(self.get_static_value(node.init_value.elements[0]))
                        else:
                            var.init_array.append(self.get_static_value(node.init_value.elements[i]))


                self.global_vars.append(var)

            elif isinstance(node, FunctionDeclNode):
                ast.body.nodes.append(node)

            else:
                raise SyntaxError(f"Only struct and variable declarations are supported in global scope: {node}")

        self.ast = ast

    def get_static_value(self, node: AstNode):
        """Gets the static value of a node, only usable with stuff like number node or a foldable binary op node"""

        if isinstance(node, NumberNode):
            return node.value


    def build_scope_stack(self):
        self.global_frame = self.build_body_frame(self.ast.body)
        self.global_frame.is_global = True
        for child in self.global_frame.children:
            child.parent = self.global_frame


    def build_body_frame(self, body: BodyNode, base_offset: int = 0) -> Frame:
        frame: Frame = Frame()

        offset: int = base_offset

        # First add variables
        for node in body.nodes:
            if isinstance(node, VariableDeclNode):
                symbol = Symbol()

                if isinstance(node.type, ArrayType):
                    # Array
                    offset += self.type_table[node.type.get_type()].size * node.type.length
                    symbol.array_length = node.type.length
                else:
                    # Normal variable
                    offset += self.type_table[node.type.get_type()].size

                symbol.name = node.name
                symbol.type = node.type
                symbol.offset = offset


                frame.symbol_table.declare_symbol(symbol)

        frame.size = offset

        # Then add inner scopes now that we know our frame size
        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                # First get parameters of function, add size of those to func frame offset so parameters get space
                params_size = 0
                for i in range(len(node.args)):
                    params_size += self.type_table[node.args[i].type.get_type()].size

                func_frame = self.build_body_frame(node.body, params_size)
                func_frame.name = node.name
                node.body.parent = frame

                # Add params as actual variables in function symbol table
                param_offset = 0
                for i in range(len(node.args)):
                    param_offset += self.type_table[node.args[i].type.get_type()].size
                    func_frame.symbol_table.declare_symbol(Symbol(node.args[i].name, node.args[i].type, param_offset))


                frame.children.append(func_frame)

            elif isinstance(node, IfNode):
                if_frame = self.build_body_frame(node.body, offset)
                if_frame.name = "if frame"
                if_frame.parent = frame
                frame.children.append(if_frame)

            elif isinstance(node, WhileNode):
                while_frame = self.build_body_frame(node.body, offset)
                while_frame.name = "while frame"
                while_frame.parent = frame
                frame.children.append(while_frame)

            elif isinstance(node, ForNode):
                # For needs special treatment, it builds frame at offset + size so we can insert its init expr at offset
                for_frame = self.build_body_frame(node.body, offset+self.type_table[node.init_expr.type.get_type()].size)
                for_frame.symbol_table.declare_symbol(Symbol(node.init_expr.name, node.init_expr.type, offset))
                for_frame.name = "for frame"
                for_frame.parent = frame
                frame.children.append(for_frame)

        body.frame = frame
        return frame



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

