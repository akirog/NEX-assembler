# NEX LANGUAGE

# Type system
## Primitive types
there are 5 built in primitive types:
`int` (4 bytes), `char` (1), `bool` (1), `uint8` (1), `void` (0).


## Type syntax
Pointers:
``*type``

Arrays:
``[type; length]``

functions:
``fn(arg0, arg1..) -> ret_type``

larger types can then be constructed:
``fn(*int, char) -> [int; 10]``

## Aliases
users can define custom type aliases with:
``
type new_type_name = type
``

## Structs
structs are defined with:
```
struct name {
    field0: type0,
    field1: type1,
};
```



# Declarations
Declarations are started with `let`, followed by their name, `:` and their type.
Examples:
```
let num: int = 10;
let msg: [char; 3];
struct Point { x: int, y: int };
let point: Point = { x: 10, y: 20 };
```

## Functions
Function declarations are started with `fn`, followed by their name, args, and optionally return type
Examples:
```
fn func1(my_arg: int) {
    do_stuff();
}

fn func2() -> char {
    return 'a';
}
```


# Expressions

## Type casts
type casts are performed with the `as` keyword.
e.g.
```
let ptr: *int = 10 as *int;

let num: int;
num = ptr as int;
```


# Enums
Enums are defined similarly to structs:
```
enum name {
    variant1,
    variant2(type1),
    variant3(type2, type3),
    variant4 { val1: type4, val2: type5 },
}
```

e.g.
```
enum MyEnum {
    number(int),
    character(char),
    more { my_num: int, my_char: char },
}
```


# Control flow
`if` / `else`, `while`, `for`, `break`, `continue`, `return`.

# Built-in functions
`sizeof(identifier)` - resolves to a compile-time `int` constant, the identifier's type size in bytes.

`__discriminant(enum_variant)` - resolves to a loadb of the enum variants first byte (the enum variant identifier), can be used to compare enum variants.


## Inline assembly
```
asm (
    "mov r0, some_label"
    "io r0, r0, r0, 1 ; comment"
);
```
Each line is passed through to the assembler almost verbatim (surrounding quotes stripped, trailing `//` comments stripped). Useful for anything the language doesn't expose yet (setting the interrupt vector, raw I/O ports, etc.) — see `kernel.nex` for examples.

## Preprocessor
See [COMPILER](#compiler) — `#include`, `#baseaddr`, `#define` are available inside `.nex` files too.