class SymbolDefinition:
    def __init__(self, name="", type="", offset=0):
        self.name: str = name
        self.type: str = type
        self.offset: int = offset

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


class Frame:
    def __init__(self):
        self.is_global = False
        self.name: str = ""
        self.size: int = 0
        self.parent: Frame | None = None
        self.children: list[Frame] = []
        self.symbol_table: SymbolTable = SymbolTable()

    def lookup_symbol(self, symbol: str) -> Frame:
        """Returns the first frame with an instance of this variable name"""
        if symbol not in self.symbol_table.symbols:
            if self.parent is None:
                raise NameError(f"Symbol {symbol} does not exist.")

            return self.parent.lookup_symbol(symbol)
        return self