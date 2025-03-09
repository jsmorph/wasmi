# Wasmi Example: Advanced Host Function Integration

This example demonstrates advanced techniques for using the Wasmi WebAssembly interpreter, showcasing the interaction between host functions and WebAssembly modules.

## Overview

The example consists of:

- A host environment (Rust) that defines a function named `waeli`
- A WebAssembly module that imports the `waeli` function and exports a function named `handle`
- Multiple execution modes that demonstrate different ways to integrate host functions

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

## Execution Modes

This example supports two distinct execution modes:

### 1. Continuation Mode (Default)

In Continuation Mode, the execution proceeds through multiple iterations:

- The `waeli` host function uses values from an array based on the call count
- If no value exists at the current index, execution halts
- When execution halts, a random value is generated and added to the array
- This process repeats until enough values have been added to complete execution
- A trace of the execution is collected, showing the relationship between generated values and execution results

### 2. Normal Mode

In Normal Mode, execution completes in a single iteration:

- The `waeli` host function always uses the Rust `waeli` function to generate random numbers
- Execution never halts, and the final result is calculated immediately
- This mode simulates the original behavior but with enhanced tracing

## Key Components

### The `waeli` Rust Function

The `waeli` Rust function:
- Takes an integer as input
- Returns a random integer in the range [0, input]
- Encapsulates the random number generation logic

### The `continuation` Function

The `continuation` function:
- Sets up the Wasmi environment with the `waeli` host function
- Loads and instantiates the WebAssembly module
- Calls the `handle` function with a specified initial value
- Returns a tuple containing the result and additional information
- Supports both Continuation Mode and Normal Mode through a flag parameter

### The `run` Function

The `run` function:
- Starts with an empty array of values
- Calls `continuation` with the current array
- If execution halts, generates a random value and adds it to the array
- Repeats until execution completes normally
- Collects a trace of the execution

## Running the Example

### Basic Execution (Continuation Mode)

```bash
cargo run
```

### With Custom Initial Value

```bash
cargo run -- 20
```

### In Normal Mode

```bash
cargo run -- 10 normal
```

## Example Output

### Continuation Mode Output

```
Running module in CONTINUATION mode with initial value 10...

Iteration #1
Current values array: []
  Initial acc = 10
  Host function waeli(10) => HALT (call #1, no value in array)
  Execution halted by waeli function
Execution halted with waeli_input: 10
Generated random value: 3 (in range [0, 10])

Iteration #2
Current values array: [3]
  Initial acc = 10
  Host function waeli(10) => 3 (call #1, from array)
  Host function waeli(7) => HALT (call #2, no value in array)
  Execution halted by waeli function
Execution halted with waeli_input: 7
Generated random value: 2 (in range [0, 7])

Iteration #3
Current values array: [3, 2]
  Initial acc = 10
  Host function waeli(10) => 3 (call #1, from array)
  Host function waeli(7) => 2 (call #2, from array)
Execution completed with result: 9

Final result = 9

Execution trace:
Trace[0]: (previous_waeli_output: None, continuation_output: (-1, 10))
Trace[1]: (previous_waeli_output: 3, continuation_output: (-1, 7))
Trace[2]: (previous_waeli_output: 2, continuation_output: (9, 0))
```

### Normal Mode Output

```
Running module in NORMAL mode with initial value 10...

Iteration #1
Current values array: []
  Initial acc = 10
  Host function waeli(10) => 6 (normal mode)
  Host function waeli(4) => 2 (normal mode)
Execution completed with result: 6

Final result = 6

Execution trace:
Trace[0]: (previous_waeli_output: None, continuation_output: (6, 0))
```

## Files

- `src/main.rs`: The Rust host application that defines the `waeli` function and calls the WebAssembly module
- `module.wat`: The WebAssembly module in text format (WAT) that defines the `handle` function

## Computational Complexity

### Current Implementation Limitations

The current implementation of Continuation Mode has a significant computational inefficiency: it exhibits quadratic complexity in terms of the number of steps required to complete execution.

#### Why It's Quadratic

In Continuation Mode, when execution halts at step N because a value is missing:

1. A random value is generated and added to the array
2. The entire WebAssembly module is re-instantiated and execution starts from the beginning
3. All previous N-1 steps must be re-executed before reaching step N again
4. This process repeats for each new value needed

This means:
- For the first missing value: 1 step
- For the second missing value: 2 steps (1 previous + 1 new)
- For the third missing value: 3 steps (2 previous + 1 new)
- And so on...

The total number of steps is therefore 1 + 2 + 3 + ... + N, which is O(N²) - quadratic complexity.

### Potential Improvements

A more sophisticated implementation could achieve linear complexity by:

1. **State Serialization**: Capturing the complete WebAssembly execution state (stack, memory, locals, etc.) at the point of halting
2. **Resumable Execution**: Implementing a mechanism to restore this state and resume execution exactly where it left off
3. **Incremental Processing**: Only executing the new steps rather than re-executing from the beginning

This approach would require:
- Deep integration with the WebAssembly runtime to access internal state
- Serialization/deserialization of the complete execution context
- A more complex control flow mechanism

#### Benefits of a Resumable Implementation

- **Linear Complexity**: Each step would only be executed once, resulting in O(N) complexity
- **Efficiency**: Significantly faster for long-running computations or those requiring many values
- **Resource Conservation**: Less memory and CPU usage for complex WebAssembly modules

#### Challenges

- **Runtime Integration**: Requires low-level access to the WebAssembly runtime internals
- **State Management**: Complete state capture is complex and runtime-specific
- **Portability**: Different WebAssembly runtimes would require different implementations

The current implementation prioritizes simplicity and clarity over performance, making it suitable for educational purposes and small-scale applications. For production systems with performance requirements, a resumable execution model would be preferable.

## Key Concepts Demonstrated

1. **Host Function Integration**: Defining Rust functions that can be called from WebAssembly
2. **Execution Control**: Halting and resuming WebAssembly execution
3. **State Management**: Maintaining state between execution iterations
4. **Execution Tracing**: Collecting detailed information about the execution process

