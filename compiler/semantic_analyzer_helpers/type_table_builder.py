from ..ast_nodes import *
from ..frame_classes import *

class TypeTableBuilder:
    def __init__(self):
        self.type_table: dict[str, TypeDefinition] = {
            # Static types
            "int": TypeDefinition("int", 4),
            "uint8": TypeDefinition("uint8", 1),
            "char": TypeDefinition("char", 1),
            "bool": TypeDefinition("bool", 1),
            "void": TypeDefinition("void", 0),
        }

        self.ast: ProgramNode = ProgramNode()


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