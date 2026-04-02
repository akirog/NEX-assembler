from .ast_nodes import *
from .frame_classes import *

operations_map = {
    "+":  "add",
    "-":  "sub",
    "*":  "mul",
    "/":  "div",
    "%":  "mod",
    "&":  "and",
    "|":  "or",
    "^":  "xor",
    "<<": "shl",
    ">>": "shr",
}


comparisons_map = {
    "==": "je",
    "!=": "jne",
    "<": "jl",
    ">": "jg",
    "<=": "jle",
    ">=": "jge",
}

logical_ops = [
    "&&",
    "||",
]

scratch_registers = [
    "r6",
    "r7",
    "r8",
    "r9",
    "r10",
    "r11",
    "r12",
]

class CodeGenerator:
    def __init__(self):
        self.ast: ProgramNode = ProgramNode()
        self.assembly: list[str] = []
        self.data_section: list[GlobalData] = []
        self.output: list[str] = []
        self.global_frame: Frame = Frame()
        self.current_frame: Frame = Frame()
        self.type_table: dict[str, TypeDefinition] = {}
        self.free_registers: list[str] = scratch_registers.copy()
        self.base_address: int = 0

        self.loop_counter: int = 0
        self.if_end_counter: int = 0
        self.if_else_counter: int = 0
        self.comparison_counter: int = 0
        self.literal_counter: int = 0

        self.if_end_stack: list[str] = []
        self.loop_end_stack: list[str] = []
        self.loop_start_stack: list[str] = []


    def get_if_end_label(self):
        self.if_end_counter += 1
        return f"_if_end_{self.if_end_counter-1}"

    def get_if_else_label(self):
        self.if_else_counter += 1
        return f"_if_else_{self.if_else_counter-1}"

    def get_loop_label(self):
        self.loop_counter += 1
        return f"_loop_{self.loop_counter}"

    def get_comparison_label(self):
        self.comparison_counter += 1
        return f"_comparison_{self.comparison_counter-1}"

    def get_scratch_reg(self) -> str:
        if len(self.free_registers) == 0:
            raise SyntaxError("Out of scratch registers")
        reg = self.free_registers[0]
        self.free_registers.pop(0)
        return reg

    def free_scratch_reg(self, reg: str):
        if reg not in self.free_registers:
            self.free_registers.append(reg)

    def free_all_scratch_regs(self):
        self.free_registers = scratch_registers.copy()


    def get_address_of_var(self, node: AstNode) -> str:
        """Get the address of a variable into a register"""

        if isinstance(node, IdentifierNode) or isinstance(node, VariableDeclNode):
            symbol = node.symbol
            output = self.get_scratch_reg()

            if symbol.is_global:
                # Global variables are not stack relative
                self.assembly.append(f"mov {output}, {symbol.label} ; Global var: {symbol.name}")
                self.assembly.append(f"movh {output}, {symbol.label} ; Global var: {symbol.name}")

            else:
                # For local vars we just sub from bp
                self.assembly.append(f"sub {output}, bp, {symbol.offset} ; Local variable address: {node.name}")

            return output

        elif isinstance(node, IndexExpressionNode):
            base_reg = self.generate_expression(node.base)
            index_reg = self.generate_expression(node.index)

            self.assembly.append(f"mul {index_reg}, {index_reg}, {self.type_table.get(node.pointee_type.get_type()).size} ; Index offset as index * size")
            self.assembly.append(f"add {base_reg}, {index_reg}, {base_reg} ; Address access, base_location + index offset")
            self.free_scratch_reg(index_reg)
            return base_reg

        else:
            raise SyntaxError(f"Cannot get address of node type {type(node)}: {node}")



    def generate(self):
        self.generate_body(self.ast.body)

        # Data section (globals)
        self.output.append(f"section .data:")
        self.output.append(f"_data_base:")
        self.generate_globals()
        self.output.append(f"_data_end:")

        # Code section
        self.output.append(f"section .text:")
        self.output.append(f"_start:")

        # Set up stack and base pointer
        self.output.append(f"mov bp, {0x10000 + self.base_address}")
        self.output.append(f"movh bp, {0x10000 + self.base_address}")
        print(self.base_address)

        self.output.append(f"mov sp, {0x10000 + self.base_address}")
        self.output.append(f"movh sp, {0x10000 + self.base_address}")

        self.output.extend(self.assembly)


    def generate_globals(self):
        for var in self.data_section:
            if var.label is not None:
                self.output.append(f"{var.label}:")

            if var.target_label is not None:
                self.output.append(f"db 0 + {var.target_label}")

            if len(var.init_bytes) == 0:
                continue

            self.output.append(f"db ")
            for value in var.init_bytes:
                for i in range(var.size):
                    self.output[-1] += f"{(value >> (8 * i)) & 0xFF}, "

            self.output[-1] = self.output[-1].removesuffix(", ")


    def generate_body(self, body: BodyNode):
        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                self.generate_function_declaration(node)

            elif isinstance(node, IfNode):
                self.generate_if(node)

            elif isinstance(node, WhileNode):
                self.generate_while(node)

            elif isinstance(node, ForNode):
                self.generate_for(node)

            elif isinstance(node, VariableDeclNode):
                if node.symbol.is_global:
                    self.generate_global_var_declaration(node)
                else:
                    self.generate_variable_declaration(node)

            elif isinstance(node, AssignmentNode):
                self.generate_assignment(node)

            elif isinstance(node, ReturnNode):
                self.generate_return(node)

            elif isinstance(node, BreakNode):
                self.generate_break(node)

            elif isinstance(node, ContinueNode):
                self.generate_continue(node)

            elif isinstance(node, FunctionCallNode):
                self.assembly.append(f"\n; Function call to {node.func_name}")
                self.generate_function_call(node)

            elif isinstance(node, AssemblyBlockNode):
                self.assembly.append(f"\n; Assembly block:")
                self.assembly.extend(node.assembly)

            else:
                raise SyntaxError(f"Cannot generate code for node {type(node)}: {node}")

            self.free_all_scratch_regs()


    def generate_global_var_declaration(self, node: VariableDeclNode):
        """Generate a global variables declaration"""

        size = self.type_table[node.type.get_type()].size

        if node.init_value is None:
            var = GlobalData()
            var.size = size
            var.label = node.name

            if isinstance(node.type, PrimitiveType) or isinstance(node.type, PointerType):
                # Both of these only take up 1*size without declaration
                var.init_bytes.append(0)

            elif isinstance(node.type, ArrayType):
                # This one is length*size, so handle empty different
                if node.type.length is None:
                    raise RuntimeError(f"Global variable array type missing initializer length")

                for i in range(node.type.length):
                    var.init_bytes.append(0)

            else:
                raise SyntaxError(f"Cannot generate global variables declaration for node {node}, type error")

            self.data_section.append(var)
            return


        data = GlobalData()
        data.size = size
        data.label = node.name

        if isinstance(node.init_value, StringLiteralNode):
            str_data = GlobalData()
            str_data.size = 1

            if isinstance(node.type, PointerType):
                # If pointer type we also want to create the pointer
                data = GlobalData()
                data.target_label = f"{node.name}__char_arr"

                self.data_section.append(data)

                str_data.label = f"{node.name}__char_arr"

            for char in node.init_value.literal:
                str_data.init_bytes.append(ord(char))

            self.data_section.append(str_data)

        elif isinstance(node.init_value, ArrayLiteralNode):
            for i in range(node.init_value.length):
                value_node = node.init_value.elements[i]
                if not isinstance(value_node, ValueNode):
                    raise RuntimeError(f"Cannot declare global variable of whatever this is: {node}")

                data.init_bytes.append(value_node.value)

            self.data_section.append(data)

        elif isinstance(node.init_value, ValueNode):
            data.init_bytes.append(node.init_value.value)

            self.data_section.append(data)

        else:
            print(f"Warning: Didnt implement global generation for this type yet: {node}")


    def generate_return(self, node: ReturnNode):
        """Generate a return node"""

        self.assembly.append(f"\n; Return")

        if not node.func_frame.is_interrupt:
            # If ret value, get return value
            if node.ret_expr is not None:
                output_reg = self.generate_expression(node.ret_expr)
                self.assembly.append(f"mov ra, {output_reg} ; Return value")

            # Generate normal stack thing
            self.assembly.append(f"mov sp, bp")
            self.assembly.append(f"pop bp")
            self.assembly.append(f"ret")

        else:
            # Interrupt return
            # Redo all registers.
            for i in range(14):
                self.assembly.append(f"load r{i}, [bp - {i * 4 + 4}]")

            # Restore old stack
            self.assembly.append(f"mov sp, bp")
            self.assembly.append(f"pop bp")

            self.assembly.append(f"iret")



    def generate_break(self, node: BreakNode):
        end_label = self.loop_end_stack[-1]

        self.assembly.append(f"jmp {end_label}")

    def generate_continue(self, node: ContinueNode):
        start_label = self.loop_start_stack[-1]

        self.assembly.append(f"jmp {start_label}")


    def generate_function_call(self, node: FunctionCallNode):
        """Generate a function call node"""

        # A Builtin function
        if node.func_frame.is_builtin:
            match node.func_frame.name:
                case "sizeof":
                    if len(node.args) != 1:
                        raise SyntaxError(f"Incorrect use of sizeof function, correct usage:\n\tsizeof(<identifier>)")

                    var = node.args[0]

                    if not isinstance(var, IdentifierNode):
                        raise SyntaxError(f"Incorrect use of sizeof function, correct usage:\n\tsizeof(<identifier>)")

                    var_type = var.symbol.type
                    if isinstance(var_type, ArrayType):
                        size = var_type.length
                    else:
                        size = self.type_table.get(var_type.get_type()).size

                    if isinstance(var_type, ArrayType):
                        size *= var_type.length

                    self.assembly.append(f"mov ra, {size} ; Built-in sizeof function, sizeof {var.name}")

                case _:
                    raise NameError(f"Unknown built-in function: {node.func_frame.name}")

            return

        # Normal function
        for i, arg in enumerate(node.args):
            reg = self.generate_expression(arg)
            self.assembly.append(f"mov a{i}, {reg}")
            self.free_scratch_reg(reg)

        self.assembly.append(f"call {node.func_name}")




    def generate_function_declaration(self, node: FunctionDeclNode):
        """Generate a function declaration, including setting up the stack and body"""
        self.current_frame = node.body.frame


        if not node.body.frame.is_interrupt:
            # First we add label and set up stack
            self.assembly.append(f"\n{node.name}:   ; Function declaration")
            self.assembly.append(f";FUNCTION INIT:")
            # push bp, bp = sp, sp -= frame size
            self.assembly.append(f"push bp")
            self.assembly.append(f"mov bp, sp")
            self.assembly.append(f"sub sp, sp, {self.current_frame.get_total_size()}")

            # Move arguments into stack, semantic analyzer has given them addresses already
            self.assembly.append(f"\n;FUNCTION ARGUMENTS:")

            for i, arg in enumerate(node.args):
                reg = f"a{i}"

                dest_frame = self.current_frame.lookup_symbol(arg.name)
                var = dest_frame.symbol_table.lookup_symbol(arg.name)
                if var is None:
                    raise SyntaxError(f"Genuinely how did this happen.")

                dest_offset = var.offset
                var_type = self.type_table.get(var.type.get_type())

                if var_type.size == 1 and not isinstance(var.type, PointerType):
                    self.assembly.append(f"store byte [bp - {dest_offset}], {reg}")
                else:
                    self.assembly.append(f"store [bp - {dest_offset}], {reg}")

        else:
            # Interrupt handler
            self.assembly.append(f"\n{node.name}:   ; Interrupt handler")

            # Make stack space
            self.assembly.append(f"push bp")
            self.assembly.append(f"mov bp, sp")
            self.assembly.append(f"sub sp, sp, {self.current_frame.get_total_size()}")

            # Store all registers
            # (don't need to store sp and bp, but I already set it up and I don't wanna take it down)
            for i in range(14):
                self.assembly.append(f"store [bp - {i * 4 + 4}], r{i}")


            # For arguments, since nothing ever gets passed into this function, we place the addresses of our arguments in the arguments
            # registers, that way you can use assembly to place interrupt data easily into the argument memory addresses.
            for i, arg in enumerate(node.args):
                reg = f"a{i}"
                dest_frame = self.current_frame.lookup_symbol(arg.name)
                var = dest_frame.symbol_table.lookup_symbol(arg.name)
                if var is None:
                    raise SyntaxError(f"Genuinely how did this happen.")

                self.assembly.append(f"sub {reg}, bp, {var.offset}")


        self.assembly.append(f"\n;FUNCTION BODY:")

        self.generate_body(node.body)

        self.assembly.append(f"")


    def generate_if(self, node: IfNode, end_label: str | None = None):

        if end_label is None:
            self.assembly.append(f"\n; If statement")
        else:
            self.assembly.append(f"\n; Else statement")


        cond_result = self.generate_expression(node.condition)

        self.current_frame = node.body.frame

        local_end_label = self.get_if_end_label() if end_label is None else end_label
        if not end_label:
            self.if_end_stack.append(local_end_label)

        else_label = self.get_if_else_label()

        self.assembly.append(f"cmp {cond_result}, 0")
        # jnz true jz false
        # If condition is false jump to else
        self.assembly.append(f"jz {else_label}")

        # Otherwise our code body will run
        self.generate_body(node.body)

        # After body code we need to jump to end
        self.assembly.append(f"jmp {local_end_label}")

        self.assembly.append(f"{else_label}:")
        if node.else_node:
            self.generate_if(node.else_node, local_end_label)

        if end_label is None:
            self.assembly.append(f"{local_end_label}:")
            self.if_end_stack.pop()
            self.assembly.append(f"\n; If statement end\n")


    def generate_while(self, node: WhileNode):
        loop_start_label = self.get_loop_label() + "_start"
        loop_end_label = self.get_loop_label() + "_end"

        self.current_frame = node.body.frame

        self.loop_end_stack.append(loop_end_label)
        self.loop_start_stack.append(loop_start_label)

        self.assembly.append(f"\n; While loop")

        # Start
        self.assembly.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        self.assembly.append(f"cmp {output}, 0")
        self.assembly.append(f"jz {loop_end_label}")

        # Body
        self.generate_body(node.body)

        # Jump to start
        self.assembly.append(f"jmp {loop_start_label}")

        # End label
        self.assembly.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()
        self.loop_start_stack.pop()

        return

    def generate_for(self, node: ForNode):
        loop_start_label = self.get_loop_label()
        loop_end_label = self.get_loop_label()

        self.current_frame = node.body.frame    

        self.loop_end_stack.append(loop_end_label)
        self.loop_start_stack.append(loop_start_label)

        self.assembly.append(f"\n; For loop")

        # Init condition
        self.generate_variable_declaration(node.init_expr)

        # Start label
        self.assembly.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        self.assembly.append(f"cmp {output}, 0")
        self.assembly.append(f"jz {loop_end_label}")

        # Body
        self.generate_body(node.body)

        # Run update expr
        self.generate_assignment(node.update_expr)

        # Jump to start
        self.assembly.append(f"jmp {loop_start_label}")

        # End label
        self.assembly.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()
        self.loop_start_stack.pop()

        return

    def generate_assignment(self, node: AssignmentNode):
        reg = self.generate_expression(node.expression)
        address = self.get_address_of_var(node.target)

        var_type = self.type_table.get(node.type.get_type())

        if var_type.size == 1:
            self.assembly.append(f"store byte [{address}] {reg} ; Assignment of: {node.target} = {node.expression}")
        else:
            self.assembly.append(f"store [{address}] {reg} ; Assignment of: {node.target} = {node.expression}")

        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)


    def generate_variable_declaration(self, node: VariableDeclNode):
        if node.init_value is None:
            return

        # Generate as assignment
        if isinstance(node.init_value, StringLiteralNode):
            if isinstance(node.init_value.type, PointerType):
                # These can be handled normally
                reg = self.generate_expression(node.init_value)

            else:
                reg = self.generate_array_creation(node.init_value, node.symbol)


        elif isinstance(node.init_value, ArrayLiteralNode):
            if isinstance(node.init_value.type, PointerType):
                # These can be handled normally
                reg = self.generate_expression(node.init_value)

            else:
                reg = self.generate_array_creation(node.init_value, node.symbol)

        else:
            reg = self.generate_expression(node.init_value)
        address = self.get_address_of_var(node)

        var_type = self.type_table[node.type.get_type()]

        if var_type.size == 1 and not isinstance(node.type, PointerType):
            self.assembly.append(f"store byte [{address}] {reg} ; Variable declaration with initial value: {node.name} = {node.init_value}")
        else:
            self.assembly.append(f"store [{address}] {reg} ; Variable declaration with initial value: {node.name} = {node.init_value}")

        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)


    def generate_array_creation(self, node: AstNode, symbol: Symbol) -> str:

        if isinstance(node, ArrayLiteralNode):
            element_size = self.type_table[node.type.dereference().get_type()].size
            offset = symbol.offset


            for expr in node.elements:
                value_reg = self.generate_expression(expr)

                if element_size == 1:
                    self.assembly.append(f"store byte [bp - {offset}], {value_reg}")
                else:
                    self.assembly.append(f"store [bp - {offset}], {value_reg}")

                self.free_scratch_reg(value_reg)
                offset -= element_size

            addr_reg = self.get_scratch_reg()
            self.assembly.append(f"sub {addr_reg}, bp, {symbol.offset}")
            return addr_reg

        elif isinstance(node, StringLiteralNode):
            offset = symbol.offset

            for char in node.literal:
                value_reg = self.get_scratch_reg()

                self.assembly.append(f"mov {value_reg}, {char}")
                self.assembly.append(f"store byte [bp - {offset}], {value_reg}")

                self.free_scratch_reg(value_reg)
                offset -= 1

            addr_reg = self.get_scratch_reg()
            self.assembly.append(f"sub {addr_reg}, bp, {symbol.offset}")
            return addr_reg


        else:
            raise SyntaxError(f"Non array expression given to generate_array_creation: {node}")



    def generate_expression(self, node: AstNode) -> str:
        """Generate the assembly for an expression and return the register with the result"""
        if not isinstance(node, BinaryOpNode):
            return self.generate_primary_expression(node)

        left_reg = self.generate_expression(node.left)
        right_reg = self.generate_expression(node.right)
        operation = node.operation

        output_reg = left_reg

        if operation in operations_map:
            self.assembly.append(f"{operations_map[operation]} {output_reg}, {left_reg}, {right_reg} ; Expression: {node}")

        elif operation in comparisons_map:
            output_reg = self.get_scratch_reg()

            label = self.get_comparison_label()
            self.assembly.append(f"mov {output_reg}, 1")
            self.assembly.append(f"cmp {left_reg}, {right_reg} ; Expression: {node}")
            self.assembly.append(f"{comparisons_map[operation]} {label}")
            self.assembly.append(f"mov {output_reg}, 0")
            self.assembly.append(f"{label}:")

            self.free_scratch_reg(left_reg)

        elif operation in logical_ops:
            # Just do alu or/and on the result of both expressions
            if operation == "||":
                self.assembly.append(f"or {left_reg}, {left_reg}, {right_reg} ; Expression: {node}")

            elif operation == "&&":
                self.assembly.append(f"and {left_reg}, {left_reg}, {right_reg} ; Expression: {node}")

        else:
            raise SyntaxError(f"Unknown operator {operation}")

        self.free_scratch_reg(right_reg)

        return output_reg

    def generate_primary_expression(self, node: AstNode) -> str:
        """Generate the assembly for a primary expression like a number or dereference"""

        if isinstance(node, ValueNode):
            output_reg = self.get_scratch_reg()
            self.assembly.append(f"mov {output_reg} {node.value} ; Primary number: {node}")

            if node.value > 0x1FFFF:
                # More than mov imm can do
                self.assembly.append(f"movh {output_reg}, {node.value} ; Primary number: {node}")

            return output_reg

        elif isinstance(node, IndexExpressionNode):
            reg = self.get_address_of_var(node)
            self.assembly.append(f"load {reg}, [{reg}]")

            if self.type_table.get(node.pointee_type.get_type()).size == 1:
                self.assembly.append(f"and {reg}, {reg}, 255 ; Single byte load, and with 0xFF")

            return reg

        elif isinstance(node, DereferenceNode):
            address_reg = self.generate_expression(node.address_expression)

            output_reg = self.get_scratch_reg()
            self.assembly.append(f"load {output_reg}, [{address_reg}] ; Dereference: *{node.address_expression}")

            if self.type_table.get(node.pointee_type.get_type()).size == 1:
                self.assembly.append(f"and {output_reg}, {output_reg}, 255 ; Single byte load, and with 0xFF")

            self.free_scratch_reg(address_reg)
            return output_reg

        elif isinstance(node, AddressOfNode):
            output_reg = self.get_address_of_var(node.variable)
            return output_reg

        elif isinstance(node, IdentifierNode):
            address_reg = self.get_address_of_var(node)

            frame = self.current_frame.lookup_symbol(node.name)

            # Arrays return their address when referenced, not their stored value
            if not isinstance(node.type, ArrayType):
                self.assembly.append(f"load {address_reg}, [{address_reg}] ; Primary Identifier: {node.name}")


            if not isinstance(node.type, ArrayType) and self.type_table[node.type.get_type()].size == 1:
                self.assembly.append(f"and {address_reg}, {address_reg}, 255 ; Single byte load, and with 0xFF")

            return address_reg

        elif isinstance(node, FunctionCallNode):
            self.generate_function_call(node)
            output_reg = self.get_scratch_reg()
            self.assembly.append(f"mov {output_reg}, ra ; Function call return value")
            return output_reg

        elif isinstance(node, UnaryOpNode):
            output_reg = self.generate_expression(node.right)
            if node.operation == "!":
                # For negating, we just xor first bit,
                self.assembly.append(f"xor {output_reg}, {output_reg}, 1 ; Negating boolean")

            elif node.operation == "-":
                # This is just neg opcode
                self.assembly.append(f"neg {output_reg}, {output_reg}")

            else:
                raise SyntaxError(f"Unknown operator {node.operation}")

            return output_reg

        elif isinstance(node, TypeCastNode):
            output_reg = self.generate_expression(node.expression)
            return output_reg

        elif isinstance(node, ArrayLiteralNode):
            # Handled only as pointers to char since array wouldn't make sense
            if isinstance(node.type, PointerType):
                global_data = GlobalData()
                global_data.label = f"__data_ptr_{self.literal_counter}"
                self.literal_counter += 1
                global_data.size = self.type_table[node.type.dereference().get_type()].size

                for element in node.elements:
                    if not isinstance(element, ValueNode):
                        raise SyntaxError(f"Cannot initialize array buffer with non constant value: {element}")

                    global_data.init_bytes.append(element.value)

                self.data_section.append(global_data)

                output_reg = self.get_scratch_reg()
                self.assembly.append(f"mov {output_reg}, {global_data.label} ; Array literal pointer")
                self.assembly.append(f"movh {output_reg}, {global_data.label} ; Array literal pointer")
                return output_reg

            else:
                raise SyntaxError(f"Cannot generate inline array of this type: {node}")


        elif isinstance(node, StringLiteralNode):
            # Handled only as pointers to char since array wouldn't make sense
            if isinstance(node.type, PointerType):
                global_data = GlobalData()
                global_data.label = f"__data_ptr_{self.literal_counter}"
                self.literal_counter += 1
                global_data.size = self.type_table[node.type.dereference().get_type()].size

                for char in node.literal:
                    global_data.init_bytes.append(ord(char))

                self.data_section.append(global_data)

                output_reg = self.get_scratch_reg()
                self.assembly.append(f"mov {output_reg}, {global_data.label} ; Array literal pointer")
                self.assembly.append(f"movh {output_reg}, {global_data.label} ; Array literal pointer")
                return output_reg

            else:
                raise SyntaxError(f"Cannot generate inline array of this type: {node}")


        else:
            raise SyntaxError(f"Cannot parse primary expression: {node}")