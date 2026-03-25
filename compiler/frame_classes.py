
class TypeField:
    def __init__(self, name="", type: TypeNode|None=None, offset=0):
        self.name: str = name
        self.type: TypeNode | None = type
        self.offset: int = offset

class TypeDefinition:
    def __init__(self, name="", size=0, fields=None):
        if fields is None:
            fields = {}
        self.name: str = name
        self.size: int = size
        self.fields: dict[str, TypeField] = fields


class Symbol:
    def __init__(self, name="", type: TypeNode|None=None, offset=0, array_length=None):
        self.name: str = name
        self.type: TypeNode | None = type
        self.offset: int = offset


class SymbolTable:
    def __init__(self):
        self.symbols: dict[str, Symbol] = {}

    def declare_symbol(self, symbol: Symbol):
        if symbol.name in self.symbols:
            raise NameError(f"Symbol {symbol.name} already exists, cannot redefine.")

        self.symbols[symbol.name] = symbol

    def lookup_symbol(self, symbol: str) -> Symbol | None:
        if symbol not in self.symbols:
            return None
        return self.symbols[symbol]


class GlobalVariable:
    def __init__(self):
        self.name: str = ""
        self.type: TypeNode | None = None
        self.init_value: int | None = None
        self.init_array: list[int] = []


class Frame:
    def __init__(self):
        self.name: str = ""
        self.size: int = 0
        self.parent: Frame | None = None
        self.children: list[Frame] = []
        self.symbol_table: SymbolTable = SymbolTable()
        self.is_global = False
        self.return_type: TypeNode | None = None

    def lookup_symbol(self, symbol: str) -> Frame:
        """Returns the first frame with an instance of this variable name"""
        if symbol not in self.symbol_table.symbols:
            if self.parent is None:
                raise NameError(f"Symbol {symbol} does not exist.")

            return self.parent.lookup_symbol(symbol)
        return self


    def get_total_size(self) -> int:
        """Get total size of this frame, meaning this size plus largest child's total size"""
        largest_child_size = 0
        for child in self.children:
            if child.get_total_size() > largest_child_size:
                largest_child_size = child.get_total_size()

        if len(self.children) == 0:
            return self.size

        return largest_child_size



class TypeNode:
    def get_type(self) -> str:
        raise NotImplementedError()

    def __repr__(self):
        return f"Missing type"

class PrimitiveType(TypeNode):
    def __init__(self, type_name: str = ""):
        self.type: str = type_name

    def __repr__(self):
        return self.type

    def get_type(self) -> str:
        return self.type


class PointerType(TypeNode):
    def __init__(self, target: TypeNode | None = None):
        self.target: TypeNode | None = target

    def __repr__(self):
        return f"Pointer<{self.target}>"

    def get_type(self) -> str:
        return self.target.get_type()


class ArrayType(TypeNode):
    def __init__(self, type_node: TypeNode | None = None, length: int | None = None):
        self.type: TypeNode | None = type_node
        self.length: int | None = length

    def __repr__(self):
        return f"Array<{self.type}, {self.length}>"

    def get_type(self) -> str:
        return self.type.get_type()