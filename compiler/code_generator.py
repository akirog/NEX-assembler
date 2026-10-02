import math
from dataclasses import dataclass
from enum import Enum, auto
from typing import Callable

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

imm_compatible_ops = [
    "add",
    "sub",
    "or",
    "mul",
    "div",
    "shl",
    "shr"
]

commutative_ops = [
    "add",
    "mul",
    "and",
    "or",
    "xor",
]


comparisons_map = {
    "==": "eq",
    "!=": "ne",
    "<": "lt",
    ">": "gt",
    "<=": "le",
    ">=": "ge",
}

comparison_swaps = {
    "gt": "lt",
    "ge": "le"
}

comparison_inversions = {
    "eq": "ne",
    "ne": "eq",
    "lt": "ge",
    "le": "gt",
    "gt": "le",
    "ge": "lt"
}


logical_ops = [
    "&&",
    "||",
]


op_funcs: dict[str, Callable] = {
    # Arithmetic
    "+":  lambda a, b: a + b,
    "-":  lambda a, b: a - b,
    "*":  lambda a, b: a * b,
    "/":  lambda a, b: a / b,
    "%":  lambda a, b: a % b,

    # Bitwise
    "&":  lambda a, b: a & b,
    "|":  lambda a, b: a | b,
    "^":  lambda a, b: a ^ b,
    "<<": lambda a, b: a << b,
    ">>": lambda a, b: a >> b,

    # Comparisons
    "==": lambda a, b: a == b,
    "!=": lambda a, b: a != b,
    "<":  lambda a, b: a < b,
    ">":  lambda a, b: a > b,
    "<=": lambda a, b: a <= b,
    ">=": lambda a, b: a >= b,

    # Logical
    "&&": lambda a, b: int(bool(a)) and int(bool(b)),
    "||": lambda a, b: int(bool(a)) or int(bool(b)),
}


class OperandType(Enum):
    Register = auto()
    Immediate = auto()
    StackOffset = auto()
    DataLabel = auto()



@dataclass
class Operand:
    value: str | int = 0
    operand_type: OperandType = OperandType.Immediate

    invalid: bool = False


    def ensure_in_reg(self, codegen: CodeGenerator):
        if self.invalid:
            raise RuntimeError(f"Attempted to put invalidated operand into register: {self}")

        if self.operand_type == OperandType.Register:
            return self.value

        reg = codegen.get_scratch_reg()

        if self.operand_type == OperandType.Immediate:
            codegen.assembly.append(f"mov {reg}, {self.value}")

        elif self.operand_type == OperandType.StackOffset:
            codegen.assembly.append(f"subi {reg}, bp, {self.value}")

        elif self.operand_type == OperandType.DataLabel:
            codegen.assembly.append(f"addi {reg}, gp, {self.value}")

        self.value = reg
        self.operand_type = OperandType.Register

        return reg


    def free_if_reg(self, codegen: CodeGenerator):
        if self.invalid:
            return

        if self.operand_type == OperandType.Register:
            if not isinstance(self.value, str):
                raise RuntimeError(f"What?")
            codegen.free_scratch_reg(self.value)

            self.invalid = True

    def check_size(self, width: int) -> bool:
        if self.operand_type != OperandType.Immediate or not isinstance(self.value, int):
            raise RuntimeError(f"Attempted to ensure size of non imm value")

        # Return false if value doesnt fit
        if abs(self.value) - (1 if self.value < 0 else 0) > math.pow(2, (width - 1)):
            return False
        return True




scratch_registers = []
for i in range(18):
    scratch_registers.append(f"t{i}")


class RegisterCache:
    def __init__(self):
        self.cache_dict: dict[int, str] = {} # stack offset to register, register holds value of stack offset
        self.saved_cache: list[dict[int, str]] = []

    def clear_cache(self):
        self.cache_dict.clear()

    def cache_reg(self, reg: str, offset: int):
        self.cache_dict[offset] = reg

    def clear_reg(self, reg: str):
        keys_to_delete = [offset for offset, r in self.cache_dict.items() if r == reg]
        for offset in keys_to_delete:
            self.cache_dict.pop(offset)

    def clear_offset(self, offset: int):
        if offset in self.cache_dict:
            self.cache_dict.pop(offset)

    def get_reg(self, offset: int):
        if offset in self.cache_dict:
            return self.cache_dict[offset]
        raise RuntimeError(f"Attempted to get cached register {offset} wich is not cached")

    def save_cache(self):
        self.saved_cache.append(self.cache_dict.copy())

    def restore_cache(self, pop=True):
        if pop:
            self.cache_dict = self.saved_cache.pop()
        else:
            self.cache_dict = self.saved_cache[-1].copy()

    def check_cached(self, offset: int):
        if offset in self.cache_dict:
            return True
        return False


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

        self.reg_cache: RegisterCache = RegisterCache()

        self.loop_counter: int = 0
        self.if_end_counter: int = 0
        self.if_else_counter: int = 0
        self.literal_counter: int = 0

        self.if_end_stack: list[str] = []
        self.loop_end_stack: list[str] = []
        self.loop_start_stack: list[str] = []

    def cache_hit(self, offset: int) -> Operand:
        reg = self.reg_cache.get_reg(offset)
        if reg in self.free_registers:
            self.free_registers.remove(reg)
        return Operand(reg, OperandType.Register)

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
            raise RuntimeError("Out of scratch registers")

        active_cached_regs = set(self.reg_cache.cache_dict.values())

        for reg in self.free_registers:
            if reg not in active_cached_regs:
                self.free_registers.remove(reg)
                return reg

        reg = self.free_registers[0]
        self.free_registers.pop(0)

        self.reg_cache.clear_reg(reg)

        return reg

    def alloc_dest(self, *sources: Operand) -> str:
        """Free source operands, then return a fresh destination register."""
        for s in sources:
            s.free_if_reg(self)
        return self.get_scratch_reg()

    def free_scratch_reg(self, reg: str):
        assert(reg.startswith("t"))
        if reg not in self.free_registers:
            self.free_registers.append(reg)

    def free_all_scratch_regs(self):
        self.free_registers = scratch_registers.copy()


    def get_address_of_var(self, node: AstNode) -> Operand:
        """Get the address of a variable into a register"""
        operand: Operand = Operand()

        if isinstance(node, IdentifierNode) or isinstance(node, VariableDeclNode):
            symbol = node.symbol

            if symbol.is_global:
                # Global variables are not stack relative
                operand.operand_type = OperandType.DataLabel
                if symbol.label is None:
                    raise RuntimeError(f"global variable not given label {symbol}")

                operand.value = symbol.label


            else:
                # For local vars we just sub from bp
                operand.operand_type = OperandType.StackOffset
                operand.value = symbol.offset

        elif isinstance(node, IndexExpressionNode):
            base = self.generate_expression(node.base)
            index = self.generate_expression(node.index)
            element_size = self.type_table.get(node.pointee_type.get_type()).size
            base_reg = base.ensure_in_reg(self)

            if index.operand_type == OperandType.Immediate:
                dest = self.alloc_dest(base)
                self.assembly.append(f"addi {dest}, {base_reg}, {index.value * element_size} ; Address access, base_location + index offset")
            else:
                idx_reg = index.ensure_in_reg(self)

                if element_size != 1:
                    dest = self.alloc_dest(index)
                    self.assembly.append(f"muli {dest}, {idx_reg}, {element_size} ; Address access, base_location + index offset")
                    base.free_if_reg(self)
                    self.assembly.append(f"add {dest}, {base_reg}, {dest}")
                else:
                    dest = self.alloc_dest(base, index)
                    self.assembly.append(f"add {dest}, {base_reg}, {idx_reg}")

            operand.operand_type = OperandType.Register
            operand.value = dest

        elif isinstance(node, DereferenceNode):
            operand = self.generate_expression(node.address_expression)

        elif isinstance(node, MemberAccessNode):
            # Address = var + member offset
            var_addr = self.get_address_of_var(node.variable)
            member_offset = self.type_table.get(node.base_type.get_type()).fields[node.member].offset

            reg = var_addr.ensure_in_reg(self)
            var_addr.free_if_reg(self)
            operand.value = self.get_scratch_reg()
            self.assembly.append(f"addi {operand.value}, {reg}, {member_offset} ; Address access, base_location + member offset")

            operand.operand_type = OperandType.Register

        else:
            raise SyntaxError(f"Cannot get address of node type {type(node)}: {node}")

        return operand


    def generate(self):
        self.generate_body(self.ast.body)

        # Data section (globals)
        self.output.append(f"section .data")
        self.generate_globals()

        # Code section
        self.output.append(f"section .text")
        self.output.append(f"_start:")

        # Set gp to data stuff
        self.output.append(f"mov gp, __data_section")

        # Set up stack and base pointer
        self.output.append(f"mov bp, {0x10000 + self.base_address}")
        print(self.base_address)

        self.output.append(f"mov sp, {0x10000 + self.base_address}")
        self.output.append(f"jal _main")

        self.output.extend(self.assembly)


    def generate_globals(self):
        for var in self.data_section:
            if var.label is not None:
                self.output.append(f"{var.label}:")

            if var.target_label is not None:
                self.output.append(f"dw {var.target_label}")

            if len(var.init_bytes) == 0:
                continue

            if var.size == 4:
                self.output.append(f"dw ")
                for value in var.init_bytes:
                    self.output[-1] += f"{value}, "

            else:
                self.output.append(f"db ")
                for value in var.init_bytes:
                    for j in range(var.size):
                        self.output[-1] += f"{(value >> (8 * j)) & 0xFF}, "

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
                self.reg_cache.clear_cache()

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

                for _ in range(node.type.length):
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
            str_data.label = f"{node.name}"

            if isinstance(node.type, PointerType):
                # If pointer type we also want to create the pointer
                data = GlobalData()
                data.target_label = f"{node.name}__char_arr"

                self.data_section.append(data)

            elif isinstance(node.type, ArrayType):
                if node.type.length is None:
                    raise RuntimeError(f"Global variable array type missing initializer length")

                for j in range(node.type.length):
                    char = node.init_value.literal[j] if j < len(node.init_value.literal) else '\0'
                    str_data.init_bytes.append(ord(char))

            else:
                for char in node.init_value.literal:
                    str_data.init_bytes.append(ord(char))


            self.data_section.append(str_data)

        elif isinstance(node.init_value, ArrayLiteralNode):
            if not isinstance(node.type, ArrayType) or node.type.length is None:
                raise RuntimeError(f"Global variable array type missing initializer length")

            for j in range(node.type.length):
                value_node = ValueNode(0) if len(node.init_value.elements) <= j else node.init_value.elements[j]
                if not isinstance(value_node, ValueNode):
                    raise RuntimeError(f"Cannot declare global variable of whatever this is: {node}")

                data.init_bytes.append(value_node.value)

            self.data_section.append(data)

        elif isinstance(node.init_value, ValueNode):
            data.init_bytes.append(node.init_value.value)

            self.data_section.append(data)

        else:
            raise Warning(f"Warning: Didn't implement global generation for this type yet: {node}")


    def generate_return(self, node: ReturnNode):
        """Generate a return node"""

        self.assembly.append(f"\n; Return")

        if node.func_frame.name == "main":
            # Main return is syscall 60, so ret value in a0, and at as 60
            if node.ret_type is None or not isinstance(node.ret_type, PrimitiveType) or node.ret_type.get_type() != "int":
                raise SyntaxError(f"Main function must return int")

            if node.ret_expr is None:
                raise SyntaxError(f"Main function must return int")

            ret_expr = self.generate_expression(node.ret_expr)
            if ret_expr.operand_type not in [OperandType.Register, OperandType.Immediate]:
                ret_expr.ensure_in_reg(self)

            self.assembly.append(f"mov v0, {ret_expr.value} ; Return value")
            ret_expr.free_if_reg(self)

            self.assembly.append(f"mov a0, 60")
            self.assembly.append(f"mov at, 0x80")

            self.assembly.append(f"trigint at")
            return

        else:
            # If ret value, get return value
            if node.ret_expr is not None:
                ret_expr = self.generate_expression(node.ret_expr)
                if ret_expr.operand_type not in [OperandType.Register, OperandType.Immediate]:
                    ret_expr.ensure_in_reg(self)

                self.assembly.append(f"mov v0, {ret_expr.value} ; Return value")

            # Generate normal stack thing
            self.assembly.append(f"mov sp, bp")
            self.assembly.append(f"load bp, [bp]")
            self.assembly.append(f"addi sp, sp, 4")
            self.assembly.append(f"load ra, [sp]")
            self.assembly.append(f"addi sp, sp, 4")
            self.assembly.append(f"jr ra")




    def generate_break(self, node: BreakNode):
        end_label = self.loop_end_stack[-1]

        self.assembly.append(f"j {end_label}")

    def generate_continue(self, node: ContinueNode):
        start_label = self.loop_start_stack[-1]

        self.assembly.append(f"j {start_label} ; continue")


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


                    self.assembly.append(f"addi v0, zero, {size} ; Built-in sizeof function, sizeof {var.name}")

                case _:
                    raise NameError(f"Unknown built-in function: {node.func_frame.name}")

            return

        # Normal function
        for j, arg in enumerate(node.args):
            arg_expr = self.generate_expression(arg)
            if arg_expr.operand_type not in [OperandType.Register, OperandType.Immediate]:
                arg_expr.ensure_in_reg(self)

            self.assembly.append(f"mov a{j}, {arg_expr.value}")
            arg_expr.free_if_reg(self)

        if node.func_name is not None:
            self.assembly.append(f"jal _{node.func_name}")
        else:
            if node.func is None:
                raise RuntimeError(f"Func node name is none: {node}")

            address_expr = self.generate_expression(node.func)
            if address_expr.operand_type == OperandType.Register:
                self.assembly.append(f"jrl {address_expr.value}")
            elif address_expr.operand_type == OperandType.Immediate:
                self.assembly.append(f"jal {address_expr.value}")
            else:
                raise RuntimeError(f"Address expression for function address not imm or reg.")

        self.reg_cache.clear_cache()




    def generate_function_declaration(self, node: FunctionDeclNode):
        """Generate a function declaration, including setting up the stack and body"""
        if node.body.frame is None:
            raise RuntimeError(f"Function body frame is none: {node}")

        self.current_frame = node.body.frame


        # First we add label and set up stack
        self.assembly.append(f"\n_{node.name}:   ; Function declaration")
        self.assembly.append(f";FUNCTION INIT:")

        # push bp, bp = sp, sp -= frame size
        self.assembly.append(f"store [sp - 4], ra")
        self.assembly.append(f"store [sp - 8], bp")
        self.assembly.append(f"subi bp, sp, 8")
        self.assembly.append(f"subi sp, sp, {self.current_frame.get_total_size() + 8}")

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

            b = "b" if var_type.size == 1 and not isinstance(var.type, PointerType) else ""

            self.assembly.append(f"store{b} [bp - {dest_offset}], {reg}")

        self.assembly.append(f"\n;FUNCTION BODY:")
        self.generate_body(node.body)
        self.reg_cache.clear_cache()
        self.assembly.append(f"")


    def generate_if(self, node: IfNode, end_label: str | None = None):

        if end_label is None:
            self.assembly.append(f"\n; If statement")
            self.reg_cache.save_cache()
        else:
            self.assembly.append(f"\n; Else statement")
            self.reg_cache.restore_cache(pop=False)

        if node.body.frame is None:
            raise RuntimeError(f"Function body frame is none: {node}")

        self.current_frame = node.body.frame

        local_end_label = self.get_if_end_label() if end_label is None else end_label
        if not end_label:
            self.if_end_stack.append(local_end_label)

        else_label = self.get_if_else_label()

        # bnz true bz false
        # If condition is false jump to else
        if isinstance(node.condition, BinaryOpNode) and node.condition.operation in comparisons_map:
            operation = comparisons_map[node.condition.operation]
            print(f"HERE OP: {operation}")
            operation = comparison_inversions[operation]

            left_expr = self.generate_expression(node.condition.left)
            right_expr = self.generate_expression(node.condition.right)

            left_expr.ensure_in_reg(self)
            right_expr.ensure_in_reg(self)
            self.assembly.append(f"b{operation} {left_expr.value}, {right_expr.value}, {else_label}")
            left_expr.free_if_reg(self)
            right_expr.free_if_reg(self)

        else:
            cond_result = self.generate_expression(node.condition)

            cond_result.ensure_in_reg(self)
            self.assembly.append(f"beq {cond_result.value}, zero, {else_label}")
            cond_result.free_if_reg(self)

        # Otherwise our code body will run
        self.generate_body(node.body)

        # After body code we need to jump to end
        self.assembly.append(f"j {local_end_label}")

        self.assembly.append(f"{else_label}:")
        if node.else_node:
            self.generate_if(node.else_node, local_end_label)

        if end_label is None:
            self.assembly.append(f"{local_end_label}:")
            self.if_end_stack.pop()
            self.reg_cache.restore_cache()
            self.reg_cache.clear_cache()
            self.assembly.append(f"\n; If statement end\n")


    def generate_while(self, node: WhileNode):
        loop_start_label = self.get_loop_label() + "_start"
        loop_end_label = self.get_loop_label() + "_end"

        if node.body.frame is None:
            raise RuntimeError(f"Function body frame is none: {node}")

        self.current_frame = node.body.frame

        self.loop_end_stack.append(loop_end_label)
        self.loop_start_stack.append(loop_start_label)

        self.assembly.append(f"\n; While loop")
        self.reg_cache.clear_cache()

        # Start
        self.assembly.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        output.ensure_in_reg(self)
        self.assembly.append(f"beq {output.value}, zero, {loop_end_label}")
        output.free_if_reg(self)

        # Body
        self.generate_body(node.body)

        # Jump to start
        self.assembly.append(f"j {loop_start_label}")
        self.reg_cache.clear_cache()

        # End label
        self.assembly.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()
        self.loop_start_stack.pop()

        return

    def generate_for(self, node: ForNode):
        loop_start_label = self.get_loop_label()
        loop_end_label = self.get_loop_label()

        if node.body.frame is None:
            raise RuntimeError(f"Function body frame is none: {node}")

        self.current_frame = node.body.frame    

        self.loop_end_stack.append(loop_end_label)
        self.loop_start_stack.append(loop_start_label + "_update")

        self.assembly.append(f"\n; For loop")

        # Init condition
        self.generate_variable_declaration(node.init_expr)

        self.reg_cache.clear_cache()

        # Start label
        self.assembly.append(f"{loop_start_label}:")

        # Check condition
        output = self.generate_expression(node.condition)
        output.ensure_in_reg(self)
        self.assembly.append(f"beq {output.value}, zero, {loop_end_label}")
        output.free_if_reg(self)

        # Body
        self.generate_body(node.body)

        # Update label
        self.reg_cache.clear_cache()
        self.assembly.append(f"{loop_start_label}_update:")

        # Run update expr
        self.generate_assignment(node.update_expr)

        # Jump to start
        self.assembly.append(f"j {loop_start_label}")
        self.reg_cache.clear_cache()

        # End label
        self.assembly.append(f"{loop_end_label}:")

        self.loop_end_stack.pop()
        self.loop_start_stack.pop()

        return

    def generate_assignment(self, node: AssignmentNode):
        expr = self.generate_expression(node.expression)
        address_expr = self.get_address_of_var(node.target)

        var_type = self.type_table.get(node.type.get_type())
        if var_type is None:
            raise RuntimeError(f"var type is none: {node}")

        computed_addr = address_expr.operand_type == OperandType.Register

        address_asm = "bp - " + str(address_expr.value) if address_expr.operand_type == OperandType.StackOffset else address_expr.ensure_in_reg(self)
        b = "b" if var_type.size == 1 else ""

        expr.ensure_in_reg(self)
        self.assembly.append(f"store{b} [{address_asm}], {expr.value} ; Assignment of: {node.target} = {node.expression}")

        if address_expr.operand_type == OperandType.StackOffset and isinstance(address_expr.value, int) and isinstance(expr.value, str):
            if b == "b":
                self.assembly.append(f"addi at, zero, 0xFF")
                self.assembly.append(f"and {expr.value}, {expr.value}, at")
            self.reg_cache.cache_reg(expr.value, address_expr.value)
        elif computed_addr:
            self.reg_cache.clear_cache()

        expr.free_if_reg(self)
        address_expr.free_if_reg(self)


    def generate_variable_declaration(self, node: VariableDeclNode):
        if node.init_value is None:
            return

        value: Operand

        # Generate as assignment
        if isinstance(node.init_value, StringLiteralNode):
            if isinstance(node.type, PointerType):
                # These can be handled normally
                value = self.generate_expression(node.init_value)

            else:
                if isinstance(node.type, ArrayType) and node.type.length > len(node.init_value.literal):
                    node.init_value.literal += "\0" * (node.type.length - len(node.init_value.literal))

                value = self.generate_array_creation(node.init_value, node.symbol)
                value.free_if_reg(self)
                return

        elif isinstance(node.init_value, ArrayLiteralNode):
            if isinstance(node.init_value.type, PointerType):
                # These can be handled normally
                value = self.generate_expression(node.init_value)

            else:
                value = self.generate_array_creation(node.init_value, node.symbol)
                value.free_if_reg(self)
                return


        elif isinstance(node.init_value, StructInitNode):
            value = self.generate_struct_creation(node.init_value, node.symbol)
            value.free_if_reg(self)
            return


        else:
            value = self.generate_expression(node.init_value)
        address_expr = self.get_address_of_var(node)

        var_type = self.type_table[node.type.get_type()]

        address_asm = "bp - " + str(address_expr.value) if address_expr.operand_type == OperandType.StackOffset else address_expr.ensure_in_reg(self)
        b = "b" if var_type.size == 1 and not isinstance(node.type, PointerType) else ""

        value.ensure_in_reg(self)
        self.assembly.append(f"store{b} [{address_asm}], {value.value} ; Variable declaration with initial value: {node.name} = {node.init_value}")

        if address_expr.operand_type == OperandType.StackOffset:
            if b == "b":
                self.assembly.append(f"addi at, zero, 0xFF")
                self.assembly.append(f"and {value.value}, {value.value}, at")
            self.reg_cache.cache_reg(value.value, address_expr.value)
        elif address_expr.operand_type == OperandType.Register:
            self.reg_cache.clear_cache()

        value.free_if_reg(self)
        address_expr.free_if_reg(self)


    def generate_array_creation(self, node: AstNode, symbol: Symbol) -> Operand:
        operand: Operand = Operand()

        if isinstance(node, ArrayLiteralNode):
            element_size = self.type_table[node.type.dereference().get_type()].size
            offset = symbol.offset


            for expr in node.elements:
                value_expr = self.generate_expression(expr)

                b = "b" if element_size == 1 else ""

                self.assembly.append(f"store{b} [bp - {offset}], {value_expr.ensure_in_reg(self)}")

                # TODO: Test if uncommenting this line works when everything else works
                # self.reg_cache.cache_reg(value_expr.value, offset)

                value_expr.free_if_reg(self)
                offset -= element_size

            operand.operand_type = OperandType.StackOffset
            operand.value = symbol.offset

        elif isinstance(node, StringLiteralNode):
            offset = symbol.offset

            value_reg = self.get_scratch_reg()
            for char in node.literal:
                self.assembly.append(f"mov {value_reg}, {ord(char)}")
                self.assembly.append(f"storeb [bp - {offset}], {value_reg}")

                # TODO: Test if uncommenting this line works when everything else works
                # self.reg_cache.cache_reg(value_reg, offset)

                offset -= 1
            self.free_scratch_reg(value_reg)

            operand.operand_type = OperandType.StackOffset
            operand.value = symbol.offset

        else:
            raise SyntaxError(f"Non array expression given to generate_array_creation: {node}")

        return operand


    def generate_struct_creation(self, node: StructInitNode, symbol: Symbol) -> Operand:

        offset = symbol.offset

        if symbol.type is None:
            raise SyntaxError(f"Symbol not given type: {symbol}")
        type_name = symbol.type.get_type()

        var_type = self.type_table.get(type_name)


        if var_type is None:
            raise SyntaxError(f"Symbol type not in type table: {symbol}")

        for j, arg_type in enumerate(var_type.fields.values()):
            arg = node.args[j]
            expr = self.generate_expression(arg)

            arg_size = self.type_table[arg_type.type.get_type()].size

            extra = symbol.offset + var_type.size - (offset + arg_type.offset + 4)
            if extra > 0:
                # Load bytes << amount, >> amount, generate value, or value and bytes, store value
                expr_reg = expr.ensure_in_reg(self)
                extra_reg = self.get_scratch_reg()
                self.assembly.append(f"load {extra_reg}, [bp - {offset - arg_type.offset + extra}]")
                self.assembly.append(f"shri {extra_reg}, {extra_reg}, {extra}")
                merged = self.get_scratch_reg()
                self.assembly.append(f"or {merged}, {expr_reg}, {extra_reg}")
                self.free_scratch_reg(extra_reg)
                expr.free_if_reg(self)
                expr = Operand(merged, OperandType.Register)
                pass


            self.assembly.append(f"store [bp - {offset - arg_type.offset}], {expr.ensure_in_reg(self)}")
            expr.free_if_reg(self)


        operand = Operand()
        operand.operand_type = OperandType.StackOffset
        operand.value = symbol.offset
        return operand




    def generate_expression(self, node: AstNode) -> Operand:
        """Generate the assembly for an expression and return the register with the result"""
        if not isinstance(node, BinaryOpNode):
            return self.generate_primary_expression(node)

        left_expr = self.generate_expression(node.left)
        right_expr = self.generate_expression(node.right)
        operation = node.operation

        operand = Operand()

        if left_expr.operand_type == OperandType.Immediate and right_expr.operand_type == OperandType.Immediate:
            # Constant fold
            op_func = op_funcs[operation]
            operand.operand_type = OperandType.Immediate
            operand.value = op_func(left_expr.value, right_expr.value)
            return operand


        if operation in operations_map:
            operation = operations_map[operation]
            if operation in imm_compatible_ops:
                if right_expr.operand_type == OperandType.Immediate:
                    if not right_expr.check_size(16):
                        right_expr.ensure_in_reg(self)
                    else:
                        operation += "i"
                    left_expr.ensure_in_reg(self)

                elif left_expr.operand_type == OperandType.Immediate and operation in commutative_ops:
                    if not left_expr.check_size(16):
                        left_expr.ensure_in_reg(self)
                        right_expr.ensure_in_reg(self)
                    else:
                        operation += "i"
                        temp = left_expr
                        left_expr = right_expr
                        right_expr = temp
                        left_expr.ensure_in_reg(self)

                else:
                    left_expr.ensure_in_reg(self)
                    right_expr.ensure_in_reg(self)
            else:
                left_expr.ensure_in_reg(self)
                right_expr.ensure_in_reg(self)


            operand.operand_type = OperandType.Register
            left_expr.free_if_reg(self)
            right_expr.free_if_reg(self)
            operand.value = self.get_scratch_reg()

            self.assembly.append(f"{operation} {operand.value}, {left_expr.value}, {right_expr.value} ; Expression: {node}")

        elif operation in comparisons_map:
            operation = comparisons_map[operation]
            if operation in comparison_swaps:
                operation = comparison_swaps[operation]
                temp = left_expr
                left_expr = right_expr
                right_expr = temp

            left_expr.ensure_in_reg(self)
            right_expr.ensure_in_reg(self)

            operand.operand_type = OperandType.Register

            left_expr.free_if_reg(self)
            right_expr.free_if_reg(self)
            operand.value = self.get_scratch_reg()
            self.assembly.append(f"{operation} {operand.value}, {left_expr.value}, {right_expr.value}")




        elif operation in logical_ops:
            operand.operand_type = OperandType.Register

            l = left_expr.ensure_in_reg(self)
            r = right_expr.ensure_in_reg(self)
            left_expr.free_if_reg(self)
            right_expr.free_if_reg(self)
            operand.value = self.get_scratch_reg()
            # Just do alu or/and on the result of both expressions
            if operation == "||":
                self.assembly.append(f"or {operand.value}, {l}, {r} ; Expression: {node}")

            elif operation == "&&":
                self.assembly.append(f"and {operand.value}, {l}, {r} ; Expression: {node}")

        else:
            raise SyntaxError(f"Unknown operator {operation}")

        return operand

    def generate_primary_expression(self, node: AstNode) -> Operand:
        """Generate the assembly for a primary expression like a number or dereference"""
        operand = Operand()

        if isinstance(node, ValueNode):
            operand.operand_type = OperandType.Immediate
            operand.value = node.value

        elif isinstance(node, IndexExpressionNode):
            operand.operand_type = OperandType.Register
            address_expr = self.get_address_of_var(node)

            if address_expr.operand_type == OperandType.StackOffset:
                if self.reg_cache.check_cached(address_expr.value):
                    operand = self.cache_hit(address_expr.value)
                    return operand

            b = "b" if self.type_table.get(node.pointee_type.get_type()).size == 1 else ""

            if address_expr.operand_type == OperandType.Immediate:
                address_asm = f"{address_expr.value}"
            elif address_expr.operand_type == OperandType.StackOffset:
                address_asm = f"bp - {address_expr.value}"
            else:
                address_asm = f"{address_expr.ensure_in_reg(self)}"

            operand.value = self.get_scratch_reg()
            self.assembly.append(f"load{b} {operand.value}, [{address_asm}]")

            address_expr.free_if_reg(self)

        elif isinstance(node, DereferenceNode):
            address_expr = self.generate_expression(node.address_expression)

            if address_expr.operand_type == OperandType.StackOffset:
                if self.reg_cache.check_cached(address_expr.value):
                    operand = self.cache_hit(address_expr.value)
                    return operand

            b = "b" if self.type_table.get(node.pointee_type.get_type()).size == 1 else ""

            if address_expr.operand_type == OperandType.Immediate:
                address_asm = f"{address_expr.value}"
            elif address_expr.operand_type == OperandType.StackOffset:
                address_asm = f"bp - {address_expr.value}"
            else:
                address_asm = f"{address_expr.ensure_in_reg(self)}"

            operand.operand_type = OperandType.Register
            operand.value = self.get_scratch_reg()
            self.assembly.append(f"load{b} {operand.value}, [{address_asm}] ; Dereference: *{node.address_expression}")

            address_expr.free_if_reg(self)

        elif isinstance(node, AddressOfNode):
            operand = self.get_address_of_var(node.variable)

        elif isinstance(node, IdentifierNode):
            address_expr = self.get_address_of_var(node)

            # Arrays return their address when referenced, not their stored value
            if not isinstance(node.type, ArrayType):
                # Check cached value
                if address_expr.operand_type == OperandType.StackOffset:
                    if self.reg_cache.check_cached(address_expr.value):
                        operand = self.cache_hit(address_expr.value)
                        return operand

                b = "b" if self.type_table[node.type.get_type()].size == 1 else ""

                if address_expr.operand_type == OperandType.Immediate:
                    address_asm = f"{address_expr.value}"
                elif address_expr.operand_type == OperandType.StackOffset:
                    address_asm = f"bp - {address_expr.value}"
                else:
                    address_asm = f"{address_expr.ensure_in_reg(self)}"

                operand.operand_type = OperandType.Register
                operand.value = self.get_scratch_reg()

                self.assembly.append(f"load{b} {operand.value}, [{address_asm}] ; Primary Identifier: {node.name}")

                if address_expr.operand_type == OperandType.StackOffset:
                    self.reg_cache.cache_reg(operand.value, address_expr.value)

                address_expr.free_if_reg(self)

            else:
                return address_expr

        elif isinstance(node, FunctionCallNode):
            self.generate_function_call(node)
            operand.operand_type = OperandType.Register
            operand.value = self.get_scratch_reg()
            self.assembly.append(f"mov {operand.value}, v0 ; Function call return value")

        elif isinstance(node, UnaryOpNode):
            value_expr = self.generate_expression(node.right)

            # Constant folding
            if value_expr.operand_type == OperandType.Immediate:
                operand.operand_type = OperandType.Immediate
                if node.operation == "-":
                    operand.value = -value_expr.value
                elif node.operation == "!":
                    operand.value = 1 if value_expr.value == 0 else 0
                print("OPERAND:" + str(operand))
                return operand


            operand.operand_type = OperandType.Register
            value_reg = value_expr.ensure_in_reg(self)
            value_expr.free_if_reg(self)
            operand.value = self.get_scratch_reg()

            if node.operation == "!":
                # For not, we just xor first bit,
                self.assembly.append(f"xori {operand.value}, {value_reg}, 1 ; Not boolean")

            elif node.operation == "-":
                # This is just neg opcode
                self.assembly.append(f"neg {operand.value}, {value_reg}")

            else:
                raise SyntaxError(f"Unknown operator {node.operation}")

        elif isinstance(node, TypeCastNode):
            operand = self.generate_expression(node.expression)

        elif isinstance(node, ArrayLiteralNode):
            # Handled only as pointers to data since array wouldn't make sense
            if isinstance(node.type, PointerType):
                global_data = GlobalData()
                global_data.label = f"__data_ptr_{self.literal_counter}"
                self.literal_counter += 1
                global_data.size = self.type_table[node.type.dereference().get_type()].size

                for element in node.elements:
                    value_expr = self.generate_expression(element)

                    if value_expr.operand_type != OperandType.Immediate or not isinstance(value_expr.value, int):
                        raise SyntaxError(f"Cannot initialize array buffer with non constant value: {element}")

                    global_data.init_bytes.append(value_expr.value)

                self.data_section.append(global_data)

                operand.operand_type = OperandType.DataLabel

                if global_data.label is None:
                    raise RuntimeError(f"wtf")
                operand.value = global_data.label

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

                operand.operand_type = OperandType.DataLabel

                if global_data.label is None:
                    raise RuntimeError(f"wtf")
                operand.value = global_data.label

            else:
                raise SyntaxError(f"Cannot generate inline array of this type: {node}")

        elif isinstance(node, MemberAccessNode):
            if not isinstance(node.variable, IdentifierNode):
                raise SyntaxError(f"Cannot get member of non identifier: {node}")

            address_expr = self.get_address_of_var(node)
            operand.operand_type = OperandType.Register

            reg = address_expr.ensure_in_reg(self)
            address_expr.free_if_reg(self)
            operand.value = self.get_scratch_reg()

            self.assembly.append(f"load {operand.value}, [{reg}] ; Member access]")

        else:
            raise SyntaxError(f"Cannot parse primary expression: {node}")

        return operand