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
///
/// # Returns
///
/// The result of calling the 'handle' function with input 10
fn continuation(wat: &str) -> Result<i32, Box<dyn std::error::Error>> {
    // Create a new engine and store
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    // Define the 'waeli' host function
    // It takes an int as input and returns a random int in the range [0, input]
    let waeli = Func::wrap(&mut store, |_caller: Caller<()>, input: i32| -> i32 {
        if input <= 0 {
            return 0;
        }
        let mut rng = rand::thread_rng();
        let result = rng.gen_range(0..=input);
        println!("  Host function waeli({}) => {}", input, result);
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
    
    // Call the continuation function with the WAT content
    println!("Instantiating module and executing...");
    let result = continuation(&wat)?;
    
    // Display the result
    println!("Final result = {}", result);

    Ok(())
}
