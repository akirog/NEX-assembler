from ..ast_nodes import *


# Built-in functions as their names and return types
built_in_function: dict[str, TypeNode] = {
    "sizeof": PrimitiveType("int"),
}



class ScopeBuilder:
    def __init__(self):
        self.global_frame: Frame = Frame()

        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {}


    def build_scope_stack(self):
        self.global_frame = self.build_global_frame(self.ast.body)




    def build_global_frame(self, body: BodyNode) -> Frame:
        """Builds a frame as the global frame"""
        frame = Frame()

        frame.is_global = True
        frame.name = "Global"

        # Set up builtin function placeholders
        for name, ret_type in built_in_function.items():
            builtin_frame = Frame()
            builtin_frame.name = name
            builtin_frame.return_type = ret_type
            builtin_frame.is_builtin = True

            frame.children.append(builtin_frame)




        for node in body.nodes:
            if isinstance(node, VariableDeclNode):
                symbol = Symbol()
                symbol.name = node.name
                symbol.type = node.type
                symbol.is_global = True

                symbol.label = node.name

                frame.symbol_table.declare_symbol(symbol)
                node.symbol = symbol


            elif isinstance(node, FunctionDeclNode):
                node_frames = self.get_node_frame(node, 0)

                if node_frames is None:
                    continue

                for node_frame in node_frames:
                    node_frame.parent = frame

                frame.children.extend(node_frames)


        body.frame = frame
        return frame



    def build_body_frame(self, body: BodyNode, base_offset: int = 0) -> Frame:
        frame: Frame = Frame()

        offset: int = base_offset

        # First add variables
        for local_node in body.nodes:
            if isinstance(local_node, ForNode):
                node = local_node.init_expr
            else:
                node = local_node


            if isinstance(node, VariableDeclNode):
                symbol = Symbol()
                symbol.name = node.name
                symbol.type = node.type

                if isinstance(node.type, PointerType):
                    # Pointers are always int
                    offset += self.type_table["int"].size

                elif isinstance(node.type, PrimitiveType):
                    # Normal variable
                    offset += self.type_table[node.type.get_type()].size

                elif isinstance(node.type, ArrayType):
                    # Space for the array
                    offset += self.type_table[node.type.get_type()].size * node.type.length

                else:
                    raise NotImplementedError(f"Type: {node.type} is not supported in scope builder")


                symbol.offset = offset

                frame.symbol_table.declare_symbol(symbol)
                node.symbol = symbol


        frame.size = offset

        # Then add inner scopes now that we know our frame size
        for node in body.nodes:
            node_frames = self.get_node_frame(node, offset)

            if node_frames is None:
                continue

            for node_frame in node_frames:
                node_frame.parent = frame

            frame.children.extend(node_frames)

        body.frame = frame
        return frame

    def get_node_frame(self, node: AstNode, offset: int) -> list[Frame] | None:
        if isinstance(node, FunctionDeclNode):
            # First get parameters of function, add size of those to func frame offset so parameters get space
            params_size = 0
            for i in range(len(node.args)):
                params_size += self.type_table[node.args[i].type.get_type()].size

            if node.type.get_type() == "interrupt":
                params_size += 64  # Space for all registers to be spilled, needed for interrupt handler

            func_frame = self.build_body_frame(node.body, params_size)
            func_frame.name = node.name

            if node.type.get_type() == "interrupt":
                func_frame.is_interrupt = True
            else:
                func_frame.return_type = node.type

            # Add params as actual variables in function symbol table
            param_offset = 0 if not func_frame.is_interrupt else 64
            for i in range(len(node.args)):
                param_offset += self.type_table[node.args[i].type.get_type()].size
                func_frame.symbol_table.declare_symbol(Symbol(node.args[i].name, node.args[i].type, param_offset))

            return [func_frame]

        elif isinstance(node, IfNode):
            if_frame = self.build_body_frame(node.body, offset)
            if_frame.name = "if frame"

            frames = [if_frame]

            if node.else_node is not None:
                frames.extend(self.get_node_frame(node.else_node, offset))

            return frames

        elif isinstance(node, WhileNode):
            while_frame = self.build_body_frame(node.body, offset)
            while_frame.name = "while frame"
            return [while_frame]

        elif isinstance(node, ForNode):
            # For needs special treatment, it builds frame at offset
            for_frame = self.build_body_frame(node.body, offset)
            for_frame.name = "for frame"
            return [for_frame]

        else:
            return None