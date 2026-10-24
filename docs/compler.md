# Compiler

## Stack
The stack is set up like this:

BP + 12: Overflow arguments (optional)\
BP + 8: Old sp location\
BP + 4: Return addr\
BP + 0: Old bp\
...\
SP - 0: Top of stack\

## Functions
### Calling convention
Callee does:
```
[SP - 4] = ra   ; Store return address given by caller
[SP - 8] = BP   ; Store old bp
BP = SP - 8     ; Decrement SP
SP = BP - scope size    ; Give space for local vars
```

### Return convention
```
SP = BP + 8     ; Top of old stack
BP = [BP]       ; Old Bp
ra = [SP + 4]   ; Temp for holding return addr
jr ra           ; Return
```

### Arguments
Arguments are passed in registers a0 - a3.\
If there are more than 4 arguments or arguments don't fit in the registers,\
the caller will increment SP by overflow and place arguments at SP + 0 ... SP + overflow.\
The callee can then access the variables after setup with BP + 12 and up.

### Return values
Return values are passed in v0-v1.\
If the return value does not fit in these registers,\
the caller will have to have passed a pointer to the return value destination.\
The callee will then place the return value at the address \
pointed to by that return pointer.

