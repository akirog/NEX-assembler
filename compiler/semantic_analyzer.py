import ast

from ast_nodes import *


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
        pass

    def check_semantics(self):
        pass