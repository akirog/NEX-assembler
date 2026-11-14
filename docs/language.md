# NEX LANGUAGE

# Type system
## Primitive types
Built in primitive types:\
`i8` (1 byte) `i32` (4 byte), \
`u8` (1 byte) `u32` (4 byte), \
`char` (1), `bool` (1), `void` (0).


## Type syntax
Pointers:
``*type``

Arrays:
``[type; length]``

Slices:
``[type]``
Slices are pointers that can be indexed

functions:
``fn(arg0, arg1..) -> ret_type``

larger types can then be constructed:
``fn(*i32, char) -> [i32; 10]``

# Array to slice coercion
An array variable can be implicitly coerced into a slice by taking its address with the `&` operator. The resulting slice points to the first element of the array and has the array's length. No elements are copied.

The element type of the array must be the same as the element type of the slice: a `[i32; 5]` can become a `[i32]`, but not a `[u8]`.

```
fn sum_of_numbers(numbers: [i32]) -> i32 {
    let total: i32 = 0;
    let index: u32 = 0;
    while index < numbers.len {
        total = total + numbers[index];
        index = index + 1;
    }
    return total;
}

fn main() {
    let array_of_numbers: [i32; 4] = [1, 2, 3, 4];
    let result: i32 = sum_of_numbers(&array_of_numbers);
}
```

Here `&array_of_numbers` has the type `*[i32; 4]`, which is implicitly coerced into `[i32]` because the function parameter expects a slice.

A slice has two members:
- `ptr`: a pointer to the first element (type `*element_type`)
- `len`: the number of elements (type `u32`)

Implicit coercion happens where a value is given to something with a known type: function call arguments, `let` initializers, struct field initializers and assignments.

```
let array_of_numbers: [i32; 4] = [1, 2, 3, 4];
let slice_of_numbers: [i32] = &array_of_numbers;
```

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
let num: i32 = 10;
let msg: [char; 3];
struct Point { x: i32, y: i32 };
let point: Point = { x: 10, y: 20 };
```

## Functions
Function declarations are started with `fn`, followed by their name, args, and optionally return type
Examples:
```
fn func1(my_arg: i32) {
    do_stuff();
}

fn func2() -> char {
    return 'a';
}
```


## Function arguments
Arguments are always passed **by value**: when a function is called, every argument is copied into the function's parameter. Changing a parameter inside the function never changes the variable that was passed in by the caller.

```
fn try_to_change(number_parameter: i32) {
    number_parameter = 99;
}

fn main() {
    let original_number: i32 = 5;
    try_to_change(original_number);
    // original_number is still 5
}
```

This applies to every type, including structs, arrays and slices. Passing a struct or an array copies all of its bytes, so passing a large value is slow and uses a lot of stack space. A slice only copies its pointer and length (8 bytes), not the elements it points to.

### Passing pointers instead of large values
To avoid copying a large value, pass a pointer to it instead. Only the pointer (4 bytes) is copied, and the function reads or writes the original value through the pointer.

```
struct Point { x: i32, y: i32 };

fn move_point_right(point_pointer: *Point) {
    (*point_pointer).x = (*point_pointer).x + 1;
}

fn main() {
    let player_position: Point = { x: 10, y: 20 };
    move_point_right(&player_position);
    // player_position.x is now 11
}
```

### Large values that a function produces
A function should not return a large value (for example a struct or an array) directly. Instead, the caller declares the variable that will hold the result and passes a pointer to it. The function writes the result through that pointer.

```
struct Point { x: i32, y: i32 };

fn create_point(destination_pointer: *Point, x_value: i32, y_value: i32) {
    (*destination_pointer).x = x_value;
    (*destination_pointer).y = y_value;
}

fn main() {
    let new_point: Point;
    create_point(&new_point, 10, 20);
    // new_point is now { x: 10, y: 20 }
}
```


# Expressions

## Type casts
type casts are performed with the `as` keyword.
e.g.
```
let ptr: *i32 = 10 as *i32;

let num: i32;
num = ptr as i32;
```

## Operations



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
    number(i32),
    character(char),
    more { my_num: i32, my_char: char },
}
```


# Control flow
`if` / `else`, `while`, `for`, `break`, `continue`, `return`.

# Built-in members
`identifier.__size` - resolves to a compile-time `u32` constant, the identifier's type size in bytes.

`enum_variant.__discriminant` - resolves to a loadb of the enum variants first byte (the enum variant identifier), can be used to compare enum variants.


## Inline assembly
```
asm (
    "mov r0, some_label"
    "io r0, r0, r0, 1 ; comment"
);
```
Each line is passed through to the assembler almost verbatim (surrounding quotes stripped, trailing `//` comments stripped). Useful for anything the language doesn't expose yet (setting the interrupt vector, raw I/O ports, etc.) — see `kernel.nex` for examples.

## Preprocessor
See [COMPILER](/README.md) — `#include`, `#baseaddr`, `#define` are available inside `.nex` files too.