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
        self.free_registers: list[str] = scratch_registers.copy()


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
            self.output.append(f"mov {output}, {address}")
        else:
            # For local vars we just sub from bp
            self.output.append(f"sub {output}, bp, {address}")

        return output



    def generate(self):
        self.generate_body(self.ast.body)

    def generate_body(self, body: BodyNode):
        self.current_frame = body.frame

        for node in body.nodes:
            if isinstance(node, FunctionDeclNode):
                self.generate_body(node.body)

            elif isinstance(node, IfNode):
                self.generate_body(node.body)

            elif isinstance(node, WhileNode):
                self.generate_body(node.body)

            elif isinstance(node, VariableDeclNode):
                if node.init_value is None:
                    continue

                # Generate as assignment
                reg = self.generate_expression(node.init_value)
                address = self.get_address_of_var(node)
                self.output.append(f"store [{address}] {reg}]")
                self.free_scratch_reg(reg)
                self.free_scratch_reg(address)

            elif isinstance(node, AssignmentNode):
                reg = self.generate_expression(node.expression)
                address = self.get_address_of_var(node.target)
                self.output.append(f"store [{address}] {reg}")
                self.free_scratch_reg(reg)
                self.free_scratch_reg(address)



            self.free_all_scratch_regs()

    def generate_expression(self, node: AstNode) -> str:
        """Generate the assembly for an expression and return the register with the result"""
        if not isinstance(node, BinaryOpNode):
            return self.generate_primary_expression(node)

        left_reg = self.generate_expression(node.left)
        right_reg = self.generate_expression(node.right)
        operation = node.operation

        output_reg = left_reg

        if operation in alu_ops:
            self.output.append(f"{operation} {output_reg}, {left_reg}, {right_reg}")
        elif operation in cmp_ops:
            raise SyntaxError("Cant use comparison operators yet, buy premium for 9.99$ for unlimited access")
        else:
            raise SyntaxError(f"Unknown operator {operation}")

        self.free_scratch_reg(left_reg)
        self.free_scratch_reg(right_reg)

        return output_reg

    def generate_primary_expression(self, node: AstNode) -> str:
        """Generate the assembly for a primary expression like a number or dereference"""

        if isinstance(node, NumberNode):
            output_reg = self.get_scratch_reg()
            self.output.append(f"mov {output_reg} {node.value}")
            return output_reg

        elif isinstance(node, DereferenceNode):
            address_reg = self.generate_expression(node.location)

            output_reg = self.get_scratch_reg()
            self.output.append(f"load {output_reg}, [{address_reg}]")
            self.free_scratch_reg(address_reg)
            return output_reg

        elif isinstance(node, AddressOfNode):
            output_reg = self.get_address_of_var(node.variable)
            return output_reg

        elif isinstance(node, IdentifierNode):
            output_reg = self.get_scratch_reg()
            address_reg = self.get_address_of_var(node)
            self.output.append(f"load {output_reg}, [{address_reg}]")
            self.free_scratch_reg(address_reg)
            return output_reg

        else:
            raise SyntaxError(f"Cannot parse primary expression: {node}")