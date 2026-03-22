
class TypeField:
    def __init__(self, name="", type="", offset=0, array_length=None):
        self.name: str = name
        self.type: str = type
        self.offset: int = offset
        self.array_length: int | None = array_length

class TypeDefinition:
    def __init__(self, name="", size=0, fields=None):
        if fields is None:
            fields = {}
        self.name: str = name
        self.size: int = size
        self.fields: dict[str, TypeField] = fields



class SymbolDefinition:
    def __init__(self, name="", type="", offset=0, array_length=None):
        self.name: str = name
        self.type: str = type
        self.offset: int = offset
        self.array_length: int | None = array_length


class SymbolTable:
    def __init__(self):
        self.symbols: dict[str, SymbolDefinition] = {}

    def declare_symbol(self, symbol: SymbolDefinition):
        if symbol.name in self.symbols:
            raise NameError(f"Symbol {symbol.name} already exists, cannot redefine.")

        self.symbols[symbol.name] = symbol

    def lookup_symbol(self, symbol: str) -> SymbolDefinition | None:
        if symbol not in self.symbols:
            return None
        return self.symbols[symbol]


class GlobalVariable:
    def __init__(self):
        self.name: str = ""
        self.type: str = ""
        self.init_value: int | None = 0
        self.is_array: bool = False
        self.init_array: list[int] = []


class Frame:
    def __init__(self):
        self.name: str = ""
        self.size: int = 0
        self.parent: Frame | None = None
        self.children: list[Frame] = []
        self.symbol_table: SymbolTable = SymbolTable()
        self.is_global = False

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