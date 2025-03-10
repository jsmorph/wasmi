# WASMI with RISC Zero zkVM Verification

This project extends the example3 WASMI project to incorporate RISC Zero zkVM verification for WebAssembly execution. It demonstrates how to:

1. Execute WebAssembly Text (WAT) code within the RISC Zero zkVM
2. Generate zero-knowledge proofs (receipts) for the execution
3. Verify these proofs to ensure the execution was performed correctly

## Project Structure

- `src/main.rs`: Host application that provides the `run_wat_in_zkvm` function
- `methods/guest/src/main.rs`: Guest code that runs inside the zkVM
- `methods/src/methods.rs`: Auto-generated file with method IDs

## How It Works

The `run_wat_in_zkvm` function takes a WebAssembly Text (WAT) string and an initial value, and:

1. Sends the WAT code and initial value to the zkVM guest
2. The guest parses the WAT, instantiates it with wasmi, and executes it
3. The guest returns the result, which is included in the zkVM receipt
4. The host verifies the receipt and returns both the result and the receipt

## Usage

### Basic Usage

```rust
use example3_zk::run_wat_in_zkvm;

fn main() {
    let wat = r#"
    (module
      (import "env" "waeli" (func $waeli (param i32) (result i32)))
      (func $handle (export "handle") (param i32) (result i32)
        (i32.add (local.get 0) (call $waeli (i32.const 10)))
      )
    )
    "#;
    
    let (result, receipt) = run_wat_in_zkvm(wat, 5).unwrap();
    
    println!("Result: {}", result);
    println!("Receipt verified: {}", receipt.verify(WAT_EXECUTOR_ID).is_ok());
}
```

### Extended Usage with example3 Integration

The `example3_extension` module provides functionality to extend the example3 code with zkVM verification:

```rust
use example3_zk::example3_extension::run_with_zk_verification;

fn main() {
    let wat = std::fs::read_to_string("module.wat").unwrap();
    let (result, trace) = run_with_zk_verification(&wat, 10).unwrap();
    
    println!("Final result: {}", result);
    println!("Execution trace with zkVM verification: {:?}", trace);
}
```

## Building and Running

```bash
# Run the basic example with an optional initial value
cargo build
cargo run -- [initial_value]

# Run the extended example3 integration
cargo run -- extension
```

## Testing

```bash
cargo test
```
