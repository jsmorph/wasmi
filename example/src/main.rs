//! Wasmi Example: Host Function Integration
//!
//! This example demonstrates:
//! 1. Defining a host function that can be called from WebAssembly
//! 2. Loading and executing a WebAssembly module
//! 3. Calling an exported function from the WebAssembly module

use rand::Rng;
use wasmi::{
    Caller, Engine, Extern, Func, Linker, Module, Store
};

/// Executes a WebAssembly module with the 'waeli' host function.
///
/// This function:
/// 1. Sets up the wasmi environment with the 'waeli' host function
/// 2. Loads and instantiates the provided WAT module
/// 3. Calls the 'handle' function exported by the module with input 10
/// 4. Returns the result of the 'handle' function
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
/// * `values` - An array of integers that can be used by the 'waeli' function
///
/// # Returns
///
/// The result of calling the 'handle' function with input 10
fn continuation(wat: &str, values: &[i32]) -> Result<i32, Box<dyn std::error::Error>> {
    // Create a new engine and store
    let engine = Engine::default();
    // Use a struct to hold our state
    struct HostState {
        values: Vec<i32>,
        call_count: usize,
    }
    let host_state = HostState {
        values: values.to_vec(),
        call_count: 0,
    };
    let mut store = Store::new(&engine, host_state);
    let mut linker = Linker::new(&engine);

    // Define the 'waeli' host function
    // It takes an int as input and returns either:
    // - The value at the nth index of the values array if it exists (where n is the call count)
    // - A random int in the range [0, input] if no value exists at that index
    let waeli = Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> i32 {
        if input <= 0 {
            return 0;
        }
        
        // Get the current call count and increment it
        let call_count = caller.data().call_count;
        caller.data_mut().call_count += 1;
        
        // Check if we have a value at the current index
        let (result, source) = if call_count < caller.data().values.len() {
            (caller.data().values[call_count], "from array")
        } else {
            // Fall back to random number if no value exists
            let mut rng = rand::thread_rng();
            (rng.gen_range(0..=input), "randomly generated")
        };
        
        println!("  Host function waeli({}) => {} (call #{}, {})", input, result, call_count + 1, source);
        result
    });

    // Register the host function in the linker
    linker.define("env", "waeli", waeli)?;

    // Load the WebAssembly module from the WAT string
    let module = Module::new(&engine, wat)?;

    // Instantiate the module
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

    // Get the exported 'handle' function
    let handle = instance
        .get_export(&store, "handle")
        .and_then(Extern::into_func)
        .ok_or("Failed to find 'handle' function export")?
        .typed::<i32, i32>(&store)?;

    // Call the 'handle' function with input 10
    let input = 10;
    println!("  Initial acc = {}", input);
    let result = handle.call(&mut store, input)?;
    
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the WebAssembly module from the WAT file
    println!("Loading WebAssembly module...");
    let wat = std::fs::read_to_string("module.wat")?;
    
    // Define some values for the waeli function to use
    let values = [5]; // Only the first call will use this value, the second call will generate a random number
    
    // Call the continuation function with the WAT content and values
    println!("Instantiating module and executing...");
    let result = continuation(&wat, &values)?;
    
    // Display the result
    println!("Final result = {}", result);

    Ok(())
}
