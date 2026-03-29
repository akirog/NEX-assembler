from ..ast_nodes import *

class ScopeBuilder:
    def __init__(self):
        self.global_frame: Frame = Frame()

        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}


    def build_scope_stack(self):
        self.global_frame = self.build_body_frame(self.ast.body)
        self.global_frame.is_global = True
        self.global_frame.name = "Global"
        for child in self.global_frame.children:
            child.parent = self.global_frame


    def build_body_frame(self, body: BodyNode, base_offset: int = 0) -> Frame:
        frame: Frame = Frame()

        offset: int = base_offset

        # First add variables
        for node in body.nodes:
            if isinstance(node, VariableDeclNode):
                symbol = Symbol()
                symbol.name = node.name
                symbol.type = node.type

                if isinstance(node.type, PointerType):
                    # Pointers are always int
                    offset += self.type_table["int"].size
                else:
                    # Normal variable
                    offset += self.type_table[node.type.get_type()].size


                symbol.offset = offset


                if isinstance(node.type, PointerType) and node.type.target_array_length is not None:
                    # Space for the array
                    offset += self.type_table[node.type.dereference().get_type()].size * node.type.target_array_length

                frame.symbol_table.declare_symbol(symbol)
                node.symbol = symbol

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
                func_frame.return_type = node.type
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