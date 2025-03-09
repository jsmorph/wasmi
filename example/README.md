# Wasmi Example: Host Function Integration

This example demonstrates how to use the Wasmi WebAssembly interpreter to:

1. Define a host function that can be called from WebAssembly
2. Load and execute a WebAssembly module
3. Call an exported function from the WebAssembly module

## Overview

The example consists of:

- A host environment (Rust) that defines a function named `waeli`
- A WebAssembly module that imports the `waeli` function and exports a function named `handle`

### The `waeli` Host Function

The `waeli` host function:
- Takes an integer as input
- Returns a random integer in the range [0, input]

### The WebAssembly Module

The WebAssembly module exports a function named `handle` that:
1. Initializes a local variable `acc` to be the integer input to `handle`
2. Calls `waeli(acc)` and checks if the result is even:
   - If even, adds the result to `acc`
   - If odd, subtracts the result from `acc`
3. Calls `waeli(acc)` again and checks if the result is even:
   - If even, adds the result to `acc`
   - If odd, subtracts the result from `acc`
4. Returns the final value of `acc`

## Running the Example

```bash
cargo run
```

## Example Output

The output shows:
- The initial value of `acc`
- The result of each call to `waeli`
- The final result returned by the `handle` function

For example, with input 10:
```
Calling handle(10):
  Initial acc = 10
  Host function waeli(10) => 10  # First call to waeli returns 10 (even)
  # acc = 10 + 10 = 20
  Host function waeli(20) => 20  # Second call to waeli returns 20 (even)
  # acc = 20 + 20 = 40
  Final result = 40
```

## Files

- `src/main.rs`: The Rust host application that defines the `waeli` function and calls the WebAssembly module
- `module.wat`: The WebAssembly module in text format (WAT) that defines the `handle` function
