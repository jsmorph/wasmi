# Snapshot and Restore Functionality

Wasmi provides the ability to save the state of a running WebAssembly instance to a file and later restore it to continue execution. This is useful for:

- Saving the state of a long-running computation to resume later
- Creating checkpoints in a WebAssembly application
- Migrating WebAssembly instances between different processes or machines
- Debugging WebAssembly applications by examining their state at different points

## API Overview

The snapshot and restore functionality is provided through the `StoreSnapshot` trait, which is implemented for `Store<T>`:

```rust
pub trait StoreSnapshot {
    fn snapshot_to_file(&self, path: &str) -> Result<(), SnapshotError>;
    fn snapshot_to_writer<W: Write>(&self, writer: &mut W) -> Result<(), SnapshotError>;
    fn restore_from_file(&mut self, path: &str, module: &Module) -> Result<Instance, SnapshotError>;
    fn restore_from_reader<R: Read>(
        &mut self,
        reader: &mut R,
        module: &Module,
    ) -> Result<Instance, SnapshotError>;
}
```

## Basic Usage

Here's a simple example of how to use the snapshot and restore functionality:

```rust
use wasmi::{Engine, Module, Store, StoreSnapshot, Instance};

// Create a WebAssembly module
let engine = Engine::default();
let module = Module::new(&engine, wasm_bytes).unwrap();

// Create a store and instantiate the module
let mut store = Store::new(&engine, ());
let instance = Instance::new(&mut store, &module, &[]).unwrap();

// ... Run some WebAssembly code ...

// Save the state to a snapshot file
store.snapshot_to_file("snapshot.bin").unwrap();

// ... Run more WebAssembly code ...

// Create a new store and restore from the snapshot
let mut new_store = Store::new(&engine, ());
let restored_instance = new_store.restore_from_file("snapshot.bin", &module).unwrap();

// Continue execution with the restored instance
```

## Error Handling

The snapshot and restore functions return a `Result<T, SnapshotError>` where `SnapshotError` can be one of:

- `Io`: An I/O error occurred
- `InvalidSnapshot`: The snapshot file is invalid or corrupted
- `IncompatibleVersion`: The snapshot version is not compatible with this version of wasmi
- `IncompatibleModule`: The module in the snapshot is not compatible with the provided module

## Implementation Details

The snapshot format includes:

1. Magic bytes to identify a wasmi snapshot file
2. Version number to ensure compatibility
3. Serialized state of the WebAssembly instance, including:
   - Memory contents
   - Global variable values
   - Table contents
   - Function states
   - Execution state (stack, locals, etc.)

## Limitations

- Snapshots are tied to a specific module. You must provide the same module when restoring a snapshot.
- Snapshots may not be compatible between different versions of wasmi.
- Host functions and external references may require special handling.

## Complete Example

For a complete example, see the [snapshot example](../crates/wasmi/examples/snapshot.rs) in the wasmi repository.
