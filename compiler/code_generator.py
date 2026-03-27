from .ast_nodes import *
from .frame_classes import *

alu_ops = {"add", "sub", "mul", "div", "mod", "and", "or", "xor", "shl", "shr"}
cmp_ops = {"e", "ne", "l", "g", "le", "ge"}

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
        self.output: list[str] = []
        self.global_vars: list[GlobalVariable] = []
        self.global_frame: Frame = Frame()
        self.current_frame: Frame = Frame()
        self.type_table: dict[str, TypeDefinition] = {}
        self.free_registers: list[str] = scratch_registers.copy()

        self.loop_counter: int = 0
        self.if_end_counter: int = 0
        self.if_else_counter: int = 0
        self.comparison_counter: int = 0

        self.if_end_stack: list[str] = []
        self.loop_end_stack: list[str] = []


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


    def get_var_type(self, node: AstNode) -> str:
        """Get the type of the member node recursively"""

        if isinstance(node, IdentifierNode):
            frame = self.current_frame.lookup_symbol(node.name)
            var = frame.symbol_table.lookup_symbol(node.name)

            return var.type.get_type()

        elif isinstance(node, IndexExpressionNode):
            return node.pointee_type.get_type()

        elif isinstance(node, AddressOfNode):
            return "int" # Address of is always int

        elif isinstance(node, DereferenceNode):
            return "int" # PLS FIX -------------------------------------------------------------------------------------------------<<<<

        elif not isinstance(node, MemberAccessNode):
            raise SyntaxError(f"Cant get type of: {node}")


        if isinstance(node.variable, IdentifierNode):
            frame = self.current_frame.lookup_symbol(node.variable.name)
            var = frame.symbol_table.lookup_symbol(node.variable.name)

            return var.type.get_type()

        elif isinstance(node.variable, MemberAccessNode):
            var_type = self.get_var_type(node.variable)
            var_info = self.type_table.get(var_type)

            member = var_info.fields.get(node.member)
            return member.type.get_type()

        else:
            raise SyntaxError(f"Member access can only be of member or variable: {node}")


    def get_address_of_var(self, node: AstNode) -> str:
        """Get the address of a variable into a register"""

        if isinstance(node, IdentifierNode) or isinstance(node, VariableDeclNode):
            symbol = node.symbol
            output = self.get_scratch_reg()

            if symbol.is_global:
                # Global variables are not stack relative
                self.output.append(f"mov lp, _data_base")
                self.output.append(f"add {output}, lp, {symbol.offset} ; Global var: {symbol.name}")

            else:
                # For local vars we just sub from bp
                self.output.append(f"sub {output}, bp, {symbol.offset} ; Local variable address: {node.name}")

            return output

        elif isinstance(node, IndexExpressionNode):
            base_reg = self.generate_expression(node.base)
            index_reg = self.generate_expression(node.index)

            self.output.append(f"mul {index_reg}, {index_reg}, {self.type_table.get(node.pointee_type.get_type()).size} ; Index offset as index * size")
            self.output.append(f"add {base_reg}, {base_reg}, {index_reg} ; Address access, base_location + index offset")
            self.free_scratch_reg(index_reg)
            return base_reg

        else:
            raise SyntaxError(f"Cannot get address of node type {type(node)}: {node}")


        if isinstance(node, MemberAccessNode):
            base_reg = self.get_address_of_var(node.variable)
            base_type = self.get_var_type(node.variable)
            offset = self.type_table.get(base_type).fields.get(node.member).offset

            # Base is in reg so just return that reg += offset
            self.output.append(f"add {base_reg}, {base_reg}, {offset} ; Address of member {node.variable} : {node.member}")

            return base_reg

        if isinstance(node, IdentifierNode):
            frame = self.current_frame.lookup_symbol(node.name)

        elif isinstance(node, VariableDeclNode):
            frame = self.current_frame.lookup_symbol(node.name)

        elif isinstance(node, DereferenceNode):
            # Something trying to get the address of a dereference node wants the address in the expression

            # Getting address of something usually leads to storing into that address, for memory dereferences we give the address so they automatically store there
            output_reg = self.generate_expression(node.address_expression)
            return output_reg




        var = frame.symbol_table.lookup_symbol(node.name)
        var_type = self.type_table.get(var.type.get_type())
        address = var.offset

        array_offset_reg: str | None = None

        if isinstance(node, IdentifierNode) and node.array_index:
            if not isinstance(var.type, PointerType) and not isinstance(var.type, ArrayType):
                raise SyntaxError("Cannot index non array variable")

            array_index_node = BinaryOpNode()
            array_index_node.left = node.array_index
            array_index_node.right = ValueNode(var_type.size)
            array_index_node.operation = "mul"

            array_offset_reg = self.generate_expression(array_index_node)

        output = self.get_scratch_reg()

        if frame.is_global:
            print("global")
            # Global variables are not stack relative
            self.output.append(f"mov lp, _data_base")
            self.output.append(f"add {output}, lp, {address} ; Global var, get data base + address")

        else:
            print("not global")
            # For local vars we just sub from bp
            self.output.append(f"sub {output}, bp, {address} ; Local variable address: {node.name}")


        if array_offset_reg and isinstance(var.type, PointerType):
            # Pointer indexing, load the location it stores then add index
            self.output.append(f"load {output}, [{output}]")
            self.output.append(f"add {output}, {output}, {array_offset_reg} ; Array access, base_location + index offset")

        elif array_offset_reg:
            # Actual array indexing
            self.output.append(f"add {output}, {output}, {array_offset_reg} ; Array access, base_location + index offset")
            self.free_scratch_reg(array_offset_reg)

        return output



    def generate(self):
        # Data section (globals)
        self.output.append(f"section .data:")
        self.output.append(f"_data_base:")
        self.generate_globals()

        # Consts section

        # Code section
        self.output.append(f"section .text:")
        self.output.append(f"_start:")

        # Set up stack and base pointer
        self.output.append(f"mov bp, 0x3FFFF")
        self.output.append(f"mov sp, 0x3FFFF")

        self.generate_body(self.ast.body)


    def generate_globals(self):
        for var in self.global_vars:
            for value in var.init_bytes:
                if var.is_relative:
                    self.output.append(f"db {value} + _data_base")
                    continue

                for i in range(var.size):
                    self.output.append(f"db {(value >> (8 * i)) & 0xFF}")


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
                self.output.append(f"\n; Function call to {node.func_name}")
                self.generate_function_call(node)

            elif isinstance(node, AssemblyBlockNode):
                self.output.append(f"\n; Assembly block:")
                self.output.extend(node.assembly)

            else:
                raise SyntaxError(f"Cannot generate code for node {type(node)}: {node}")

            self.free_all_scratch_regs()

    def generate_return(self, node: ReturnNode):
        """Generate a return node"""

        self.output.append(f"\n; Return")

        # If ret value, get return value
        if node.ret_expr is not None:
            output_reg = self.generate_expression(node.ret_expr)
            self.output.append(f"mov ra, {output_reg} ; Return value")

        # Generate normal stack thing
        self.output.append(f"mov sp, bp")
        self.output.append(f"pop bp")
        self.output.append(f"ret")

    def generate_break(self, node: BreakNode):
        end_label = self.if_end_stack[-1]

        self.output.append(f"jmp {end_label}")

    def generate_continue(self, node: ContinueNode):
        pass


    def generate_function_call(self, node: FunctionCallNode):
        """Generate a function call node"""

        for i, arg in enumerate(node.args):
            reg = self.generate_expression(arg)
            self.output.append(f"mov a{i}, {reg}")
            self.free_scratch_reg(reg)

        self.output.append(f"call {node.func_name}")




    def generate_function_declaration(self, node: FunctionDeclNode):
        """Generate a function declaration, including setting up the stack and body"""
        self.current_frame = node.body.frame

        # First we add label and set up stack
        self.output.append(f"\n{node.name}:   ; Function declaration")
        self.output.append(f";FUNCTION INIT:")
        # push bp, bp = sp, sp -= frame size
        self.output.append(f"push bp")
        self.output.append(f"mov bp, sp")
        self.output.append(f"sub sp, sp, {self.current_frame.get_total_size()}")

        # Move arguments into stack, semantic analyzer has given them addresses already
        self.output.append(f"\n;FUNCTION ARGUMENTS:")

        for i, arg in enumerate(node.args):
            reg = f"a{i}"

            dest_frame = self.current_frame.lookup_symbol(arg.name)
            var = dest_frame.symbol_table.lookup_symbol(arg.name)
            dest_offset = var.offset
            var_type = self.type_table.get(var.type.get_type())

            if var_type.size == 1 and not isinstance(var.type, PointerType):
                self.output.append(f"store byte [bp - {dest_offset}], {reg}")
            else:
                self.output.append(f"store [bp - {dest_offset}], {reg}")

        self.output.append(f"\n;FUNCTION BODY:")

        self.generate_body(node.body)

        self.output.append(f"")


    def generate_if(self, node: IfNode, end_label: str | None = None):

        self.output.append(f"\n; If statement")

        cond_result = self.generate_expression(node.condition)

        self.current_frame = node.body.frame

        local_end_label = self.get_if_end_label() if end_label is None else end_label
        if not end_label:
            self.if_end_stack.append(local_end_label)

        else_label = self.get_if_else_label()

        self.output.append(f"cmp {cond_result}, 0")
        # jnz true jz false
        # If condition is false jump to else
        self.output.append(f"jz {else_label}")

        # Otherwise our code body will run
        self.generate_body(node.body)

        self.output.append(f"{else_label}:")
        if node.else_node:
            self.generate_if(node.else_node, local_end_label)

        if end_label is None:
            self.output.append(f"{local_end_label}:")
            self.if_end_stack.pop()


    def generate_while(self, node: WhileNode):
        loop_start_label = self.get_loop_label()
        loop_end_label = self.get_loop_label()

        self.current_frame = node.body.frame

        self.loop_end_stack.append(loop_end_label)

        self.output.append(f"\n; While loop")

        # Start
        self.output.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        self.output.append(f"cmp {output}, 0")
        self.output.append(f"jz {loop_end_label}")

        # Body
        self.generate_body(node.body)

        # Jump to start
        self.output.append(f"jmp {loop_start_label}")

        # End label
        self.output.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()

        return

    def generate_for(self, node: ForNode):
        loop_start_label = self.get_loop_label()
        loop_end_label = self.get_loop_label()

        self.current_frame = node.body.frame    

        self.loop_end_stack.append(loop_end_label)

        self.output.append(f"\n; For loop")

        # Init condition
        self.generate_variable_declaration(node.init_expr)

        # Start label
        self.output.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        self.output.append(f"cmp {output}, 0")
        self.output.append(f"jz {loop_end_label}")

        # Run update expr
        self.generate_assignment(node.update_expr)

        # Body
        self.generate_body(node.body)

        # Jump to start
        self.output.append(f"jmp {loop_start_label}")

        # End label
        self.output.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()

        return

    def generate_assignment(self, node: AssignmentNode):
        reg = self.generate_expression(node.expression)
        address = self.get_address_of_var(node.target)

        var_type = self.type_table.get(self.get_var_type(node.target))

        if var_type.size == 1:
            self.output.append(f"store byte [{address}] {reg} ; Assignment of: {node.target} = {node.expression}")
        else:
            self.output.append(f"store [{address}] {reg} ; Assignment of: {node.target} = {node.expression}")

        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)


    def generate_variable_declaration(self, node: VariableDeclNode):
        if node.init_value is None:
            return

        if isinstance(node.type, PointerType):
            # Array declarations handled separately
            self.generate_array_declaration(node)
            return

        # Generate as assignment
        reg = self.generate_expression(node.init_value)
        address = self.get_address_of_var(node)

        var_type = self.type_table.get(node.type.get_type())

        if var_type.size == 1 and not isinstance(node.type, PointerType):
            self.output.append(f"store byte [{address}] {reg} ; Variable declaration with initial value: {node.name} = {node.init_value}")
        else:
            self.output.append(f"store [{address}] {reg} ; Variable declaration with initial value: {node.name} = {node.init_value}")

        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)

    def generate_array_declaration(self, node: VariableDeclNode):
        """Directly creates the array and places it in memory"""
        if not isinstance(node.type, PointerType) or node.type.target_array_length is None:
            raise SystemError("Normal variable declaration given to generate array declaration")

        if not isinstance(node.init_value, ArrayLiteralNode):
            raise SyntaxError(f"Normal variable declaration given to generate array declaration: {node}")

        var_type = self.type_table.get(node.type.get_type())

        index = 0
        while index < node.type.target_array_length:
            if index < node.init_value.length:
                # If index is out of range use first element, for stuff like int arr[10] = [0]
                value_reg = self.generate_expression(node.init_value.elements[0])
            else:
                value_reg = self.generate_expression(node.init_value.elements[index])

            offset = node.symbol.offset

            offset += index * self.type_table.get(node.type.get_type()).size

            if var_type.size == 1:
                self.output.append(f"store byte [bp - {offset}], {value_reg} ; array declaration: {node.name}[{index}]")
            else:
                self.output.append(f"store [bp - {offset}], {value_reg} ; array declaration: {node.name}[{index}]")

            self.free_scratch_reg(value_reg)

            index += 1



    def generate_expression(self, node: AstNode) -> str:
        """Generate the assembly for an expression and return the register with the result"""
        if not isinstance(node, BinaryOpNode):
            return self.generate_primary_expression(node)

        left_reg = self.generate_expression(node.left)
        right_reg = self.generate_expression(node.right)
        operation = node.operation

        output_reg = left_reg

        if operation in alu_ops:
            self.output.append(f"{operation} {output_reg}, {left_reg}, {right_reg} ; Expression: {node}")
        elif operation in cmp_ops:
            output_reg = self.get_scratch_reg()

            label = self.get_comparison_label()
            self.output.append(f"mov {output_reg}, 1")
            self.output.append(f"cmp {left_reg}, {right_reg} ; Expression: {node}")
            self.output.append(f"j{operation} {label}")
            self.output.append(f"mov {output_reg}, 0")
            self.output.append(f"{label}:")

            self.free_scratch_reg(left_reg)
        else:
            raise SyntaxError(f"Unknown operator {operation}")

        self.free_scratch_reg(right_reg)

        return output_reg

    def generate_primary_expression(self, node: AstNode) -> str:
        """Generate the assembly for a primary expression like a number or dereference"""

        if isinstance(node, ValueNode):
            output_reg = self.get_scratch_reg()
            self.output.append(f"mov {output_reg} {node.value} ; Primary number: {node.value}")
            return output_reg

        elif isinstance(node, IndexExpressionNode):
            reg = self.get_address_of_var(node)
            self.output.append(f"load {reg}, [{reg}]")

            if self.type_table.get(node.pointee_type.get_type()).size == 1:
                #self.output.append(f"and {reg}, {reg}, 255 ; Single byte load, and with 0xFF")
                pass
            return reg

        elif isinstance(node, DereferenceNode):
            address_reg = self.generate_expression(node.address_expression)

            output_reg = self.get_scratch_reg()
            self.output.append(f"load {output_reg}, [{address_reg}] ; Dereference: *{node.address_expression}")

            if self.type_table.get(node.pointee_type.get_type()).size == 1:
                #self.output.append(f"and {output_reg}, {output_reg}, 255 ; Single byte load, and with 0xFF")
                pass
            self.free_scratch_reg(address_reg)
            return output_reg

        elif isinstance(node, AddressOfNode):
            output_reg = self.get_address_of_var(node.variable)
            return output_reg

        elif isinstance(node, IdentifierNode):
            output_reg = self.get_scratch_reg()
            address_reg = self.get_address_of_var(node)

            frame = self.current_frame.lookup_symbol(node.name)
            var = frame.symbol_table.lookup_symbol(node.name)

            self.output.append(f"load {output_reg}, [{address_reg}] ; Primary Identifier: {node.name}")

            if self.type_table.get(node.type.get_type()).size == 1:
                #self.output.append(f"and {output_reg}, {output_reg}, 255 ; Single byte load, and with 0xFF")
                pass

            self.free_scratch_reg(address_reg)
            return output_reg

        elif isinstance(node, FunctionCallNode):
            self.generate_function_call(node)
            output_reg = self.get_scratch_reg()
            self.output.append(f"mov {output_reg}, ra ; Function call return value")
            return output_reg

        else:
            raise SyntaxError(f"Cannot parse primary expression: {node}")