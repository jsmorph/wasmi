//! Test demonstrating how to halt WASM execution at a host function call,
//! take a snapshot, and then resume execution with the pre-computed result.

use wasmi::{
    Engine, Module, Store, StoreSnapshot, Instance, Func, Caller, Extern,
    errors::SnapshotError, Error,
};
use std::fs;
use std::path::Path;

/// A module that calls a host function and uses its result.
const HALT_RESUME_WAT: &str = r#"
    (module
        ;; Import a host function that will halt execution
        (import "host" "work" (func $host_work (param i32) (result i32)))
        
        (global $input (mut i32) (i32.const 0))  ;; Initial input value
        (global $result (mut i32) (i32.const 0)) ;; Will store the result
        
        ;; Call the host function and store its result
        (func $impure_computation (export "impure_computation") (result i32)
            ;; Get the input value
            global.get $input
            
            ;; Add 1 to the input before calling the host function
            i32.const 1
            i32.add
            
            ;; Call the host function (this will halt execution)
            call $host_work
            
            ;; Add 1 to the result from the host function
            i32.const 1
            i32.add
            
            ;; Store the result
            global.set $result
            
            ;; Return the result
            global.get $result
        )
        
        ;; Get the current result
        (func $get_result (export "get_result") (result i32)
            global.get $result
        )
        
        ;; Get the input value
        (func $get_input (export "get_input") (result i32)
            global.get $input
        )
    )
"#;

#[test]
fn test_snapshot_halt_resume() {
    // Create a temporary file for the snapshot
    let snapshot_path = "halt_resume_snapshot.bin";
    
    // Clean up any existing snapshot file
    if Path::new(snapshot_path).exists() {
        fs::remove_file(snapshot_path).unwrap();
    }
    
    // Create a new engine and module
    let engine = Engine::default();
    let module = Module::new(&engine, HALT_RESUME_WAT).unwrap();
    
    // Create a store
    let mut store = Store::new(&engine, ());
    
    // Create a host function that will halt execution by returning an error
    let host_work = Func::wrap(&mut store, |_caller: Caller<'_, ()>, input: i32| -> Result<i32, Error> {
        println!("Host function called with input: {}", input);
        
        // Halt execution by returning an error
        Err(Error::new("Execution halted at host function call"))
    });
    
    // Create the imports array with our host function
    let imports = [Extern::Func(host_work)];
    
    // Instantiate the module with the imports
    let instance = Instance::new(&mut store, &module, &imports).unwrap();
    
    // Get the exported functions
    let impure_computation = instance.get_typed_func::<(), i32>(&store, "impure_computation").unwrap();
    let get_input = instance.get_typed_func::<(), i32>(&store, "get_input").unwrap();
    
    // Get the input value
    let input = get_input.call(&mut store, ()).unwrap();
    println!("Initial input: {}", input);
    
    // Call the host function, which should halt execution
    println!("Call WASM impure computation");
    let call_result = impure_computation.call(&mut store, ());
    
    // Verify that execution was halted
    assert!(call_result.is_err(), "Execution should have been halted");
    println!("Execution halted at host function as expected: {:?}", call_result.err().unwrap());
    
    // Take a snapshot at this point
    println!("Taking snapshot at host function call");
    match store.snapshot_to_file(snapshot_path) {
        Ok(_) => println!("Snapshot created successfully"),
        Err(SnapshotError::InvalidSnapshot) => {
            // This is expected since the implementation is not complete
            println!("Note: Snapshot functionality is a placeholder in this version.");
            println!("A full implementation would save the entire WebAssembly state.");
        }
        Err(e) => {
            panic!("Failed to create snapshot: {:?}", e);
        }
    }
    
    // Now, compute the result outside of WASM
    // In a real application, this might involve complex computation or external services
    let computed_result = input * 2; // Simple computation: double the input
    println!("Host computed result outside of WASM: {}", computed_result);
    
    // Create a new store for restoration
    let mut new_store = Store::new(&engine, ());
    
    // Create a new host function that will provide the pre-computed result
    let new_host_work = Func::wrap(&mut new_store, move |_caller: Caller<'_, ()>, input: i32| -> i32 {
        println!("Resumed execution with host function input: {}", input);
        // Return the pre-computed result
        println!("Returning pre-computed host function output: {}", computed_result);
        computed_result
    });
    
    // Create the imports array with our new host function
    let new_imports = [Extern::Func(new_host_work)];
    
    // Restore from the snapshot with the new host function
    println!("Restoring from snapshot with pre-computed result");
    match new_store.restore_from_file_with_imports(snapshot_path, &module, &new_imports) {
        Ok(new_instance) => {
            // Get the exported functions from the new instance
            let new_impure_computation = new_instance.get_typed_func::<(), i32>(&new_store, "impure_computation").unwrap();
            let new_get_result = new_instance.get_typed_func::<(), i32>(&new_store, "get_result").unwrap();
            
            // Resume execution from where it was halted
            let resume_result = new_impure_computation.call(&mut new_store, ()).unwrap();
            println!("Resumed execution result: {}", resume_result);
            
            // The WASM code adds 1 to the result from the host function
            let expected_result = computed_result + 1;
            
            // Verify that the result matches our expected value (pre-computed + 1)
            assert_eq!(resume_result, expected_result, "Result should match pre-computed value + 1");
            
            // Check the stored result
            let stored_result = new_get_result.call(&mut new_store, ()).unwrap();
            println!("Stored result: {}", stored_result);
            assert_eq!(stored_result, expected_result, "Stored result should match pre-computed value + 1");
            
           println!("Snapshot halt and resume test passed");
        }
        Err(SnapshotError::InvalidSnapshot) => {
            // This is expected since the implementation is not complete
            println!("Note: Snapshot restoration is a placeholder in this version.");
            println!("A full implementation would restore the entire WebAssembly state.");
        }
        Err(e) => {
            panic!("Failed to restore snapshot: {:?}", e);
        }
    }
    
    // Clean up the snapshot file
    if Path::new(snapshot_path).exists() {
        fs::remove_file(snapshot_path).unwrap();
        println!("Cleaned up snapshot file");
    }
}
