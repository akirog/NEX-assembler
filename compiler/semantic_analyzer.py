from ast_nodes import *
from frame_classes import *

class TypeField:
    def __init__(self, name="", type="", offset=0):
        self.name: str = name
        self.type: str = type
        self.offset: int = offset

class TypeDefinition:
    def __init__(self, name="", size=0, fields=None):
        if fields is None:
            fields = {}
        self.name: str = name
        self.size: int = size
        self.fields: dict[str, TypeField] = fields

class SemanticAnalyzer:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.type_table: dict[str, TypeDefinition] = {
            # Static types
            "int": TypeDefinition("int", 4),
            "char": TypeDefinition("char", 1),
            "bool": TypeDefinition("bool", 1),
        }
        self.global_frame: Frame = Frame()


    def analyze(self):
        self.build_type_table()
        self.build_scope_stack()
        self.check_semantics()


    def build_type_table(self):
        for node in self.ast.body.nodes:
            if not isinstance(node, StructDeclNode):
                continue

            new_type = TypeDefinition()
            new_type.name = node.name

            offset = 0
            for field in node.fields:
                new_type.fields[field.name] = TypeField(name=field.name, type=field.type, offset=offset)
                new_type.size += self.type_table[field.type].size
                offset += self.type_table[field.type].size

            self.type_table[node.name] = new_type


    def build_scope_stack(self):
        self.global_frame = self.build_body_frame(self.ast.body)
        self.global_frame.is_global = True


    def build_body_frame(self, body: BodyNode, base_offset: int = 0) -> Frame:
        frame: Frame = Frame()

        offset: int = base_offset

        # First add variables
        for node in body.nodes:
            if isinstance(node, VariableDeclNode):
                symbol = SymbolDefinition()

                offset += self.type_table[node.type].size

                symbol.name = node.name
                symbol.type = node.type
                symbol.offset = offset

                frame.symbol_table.declare_symbol(symbol)

        frame.size = offset

        # Then add inner scopes now that we know our frame size
        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                # Func decl nodes get their own frame so no need to pass in offset
                func_frame = self.build_body_frame(node.body)
                func_frame.name = node.name
                node.body.parent = frame
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
                for_frame = self.build_body_frame(node.body, offset+self.type_table[node.init_expr.type].size)
                for_frame.symbol_table.declare_symbol(SymbolDefinition(node.init_expr.name, node.init_expr.type, offset))
                for_frame.name = "for frame"
                for_frame.parent = frame
                frame.children.append(for_frame)

        body.frame = frame
        return frame



    def check_semantics(self):
        pass