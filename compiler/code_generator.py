from ast_nodes import *
from frame_classes import *

alu_ops = {"add", "sub", "mul", "div", "mod", "and", "or", "xor", "shl", "shr"}
cmp_ops = {"eq", "neq", "lt", "gt", "lte", "gte"}

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

            return var.type

        elif not isinstance(node, MemberAccessNode):
            raise SyntaxError(f"Member access can only be of member or variable: {node}")


        if isinstance(node.variable, IdentifierNode):
            frame = self.current_frame.lookup_symbol(node.variable.name)
            var = frame.symbol_table.lookup_symbol(node.variable.name)

            return var.type

        elif isinstance(node.variable, MemberAccessNode):
            var_type = self.get_var_type(node.variable)
            var_info = self.type_table.get(var_type)

            member = var_info.fields.get(node.member)
            return member.type

        else:
            raise SyntaxError(f"Member access can only be of member or variable: {node}")


    def get_address_of_var(self, node: AstNode) -> str:
        """Get the address of a variable into a register"""

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


        else:
            raise SyntaxError(f"Cannot get address of node type {type(node)}: {node}")

        address = frame.symbol_table.lookup_symbol(node.name).offset

        output = self.get_scratch_reg()

        if frame.is_global:
            # Global variables are not stack relative
            self.output.append(f"mov {output}, {address} ; Global var, gotta change this check")
        else:
            # For local vars we just sub from bp
            self.output.append(f"sub {output}, bp, {address} ; Local variable address: {node.name}")

        return output



    def generate(self):
        self.generate_body(self.ast.body)

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
                self.generate_function_call(node)


            self.free_all_scratch_regs()

    def generate_return(self, node: ReturnNode):
        """Generate a return node"""

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
            self.output.append(f"mov a{i}, reg")
            self.free_scratch_reg(reg)

        # Before call push sp
        self.output.append(f"push sp")
        self.output.append(f"call {node.func_name}")




    def generate_function_declaration(self, node: FunctionDeclNode):
        """Generate a function declaration, including setting up the stack and body"""
        self.current_frame = node.body.frame

        # First we add label and set up stack
        self.output.append(f"{node.name}:   ; Function declaration: {node.name}")

        # push bp, bp = sp, sp -= frame size
        self.output.append(f"push bp")
        self.output.append(f"mov bp, sp")
        self.output.append(f"sub sp, sp, {self.current_frame.get_total_size()}")

        self.output.append(f"\n;FUNCTION BODY:\n")

        self.generate_body(node.body)


    def generate_if(self, node: IfNode, end_label: str | None = None):
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
        self.output.append(f"store [{address}] {reg} ; Assignment of: {node.target} = {node.expression}")
        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)


    def generate_variable_declaration(self, node: VariableDeclNode):
        if node.init_value is None:
            return

        # Generate as assignment
        reg = self.generate_expression(node.init_value)
        address = self.get_address_of_var(node)
        self.output.append(
            f"store [{address}] {reg} ; Variable declaration with initial value: {node.name} = {node.init_value}")
        self.free_scratch_reg(reg)
        self.free_scratch_reg(address)


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
            label = self.get_comparison_label()
            self.output.append(f"mov {output_reg}, 1")
            self.output.append(f"cmp {left_reg}, {right_reg} ; Expression: {node}")
            self.output.append(f"{operation} {label}")
            self.output.append(f"mov {output_reg}, 0")
            self.output.append(f"{label}:")
        else:
            raise SyntaxError(f"Unknown operator {operation}")

        self.free_scratch_reg(left_reg)
        self.free_scratch_reg(right_reg)

        return output_reg

    def generate_primary_expression(self, node: AstNode) -> str:
        """Generate the assembly for a primary expression like a number or dereference"""

        if isinstance(node, NumberNode):
            output_reg = self.get_scratch_reg()
            self.output.append(f"mov {output_reg} {node.value} ; Primary number: {node.value}")
            return output_reg

        elif isinstance(node, DereferenceNode):
            address_reg = self.generate_expression(node.address_expression)

            output_reg = self.get_scratch_reg()
            self.output.append(f"load {output_reg}, [{address_reg}] ; Primary Dereference: *{node.address_expression}")
            self.free_scratch_reg(address_reg)
            return output_reg

        elif isinstance(node, AddressOfNode):
            output_reg = self.get_address_of_var(node.variable)
            return output_reg

        elif isinstance(node, IdentifierNode):
            output_reg = self.get_scratch_reg()
            address_reg = self.get_address_of_var(node)
            self.output.append(f"load {output_reg}, [{address_reg}] ; Primary Identifier: {node.name}")
            self.free_scratch_reg(address_reg)
            return output_reg

        else:
            raise SyntaxError(f"Cannot parse primary expression: {node}")