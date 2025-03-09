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
/// 4. Returns a pair containing the result of the 'handle' function and the last input to 'waeli'
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
/// * `values` - An array of integers that can be used by the 'waeli' function
///
/// # Returns
///
/// A tuple containing:
/// - The result of calling the 'handle' function with input 10, or -1 if execution was halted
/// - The last input passed to 'waeli' if execution was halted, or 0 if execution completed normally
fn continuation(wat: &str, values: &[i32]) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    // Create a new engine and store
    let engine = Engine::default();
    // Use a struct to hold our state
    struct HostState {
        values: Vec<i32>,
        call_count: usize,
        last_input: i32,
        execution_halted: bool,
    }
    let host_state = HostState {
        values: values.to_vec(),
        call_count: 0,
        last_input: 0,
        execution_halted: false,
    };
    let mut store = Store::new(&engine, host_state);
    let mut linker = Linker::new(&engine);

    // Define the 'waeli' host function
    // It takes an int as input and returns either:
    // - The value at the nth index of the values array if it exists (where n is the call count)
    // - Halts execution if no value exists at that index
    let waeli = Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> Result<i32, wasmi::Error> {
        // Store the last input
        caller.data_mut().last_input = input;
        
        if input <= 0 {
            return Ok(0);
        }
        
        // Get the current call count and increment it
        let call_count = caller.data().call_count;
        caller.data_mut().call_count += 1;
        
        // Check if we have a value at the current index
        if call_count < caller.data().values.len() {
            let result = caller.data().values[call_count];
            println!("  Host function waeli({}) => {} (call #{}, from array)", input, result, call_count + 1);
            Ok(result)
        } else {
            // Halt execution instead of generating a random number
            println!("  Host function waeli({}) => HALT (call #{}, no value in array)", input, call_count + 1);
            caller.data_mut().execution_halted = true;
            // Return a trap to halt execution
            Err(wasmi::Error::new("Execution halted: no value in array for this call"))
        }
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
    
    // Call the handle function and check if execution was halted
    let result = match handle.call(&mut store, input) {
        Ok(result) => {
            // Execution completed normally
            (result, 0)
        },
        Err(_) => {
            // Execution was halted by the waeli function
            if store.data().execution_halted {
                println!("  Execution halted by waeli function");
                (-1, store.data().last_input)
            } else {
                // Some other error occurred
                return Err("Unexpected error during execution".into());
            }
        }
    };
    
    Ok(result)
}

/// Runs the WebAssembly module until completion by dynamically generating values
/// for the 'waeli' host function as needed.
///
/// This function:
/// 1. Starts with an empty array of values
/// 2. Calls 'continuation' with the current array
/// 3. If execution halts, generates a random value and adds it to the array
/// 4. Repeats until execution completes normally
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
///
/// # Returns
///
/// The final result of the WebAssembly module execution
fn run(wat: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut values = Vec::new();
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        println!("\nIteration #{}", iteration);
        println!("Current values array: {:?}", values);
        
        // Call continuation with the current values
        let (result, waeli_input) = continuation(wat, &values)?;
        
        if result != -1 {
            // Execution completed normally
            println!("Execution completed with result: {}", result);
            return Ok(result);
        }
        
        // Execution was halted, generate a random value
        println!("Execution halted with waeli_input: {}", waeli_input);
        let mut rng = rand::thread_rng();
        let random_value = rng.gen_range(0..=waeli_input);
        println!("Generated random value: {} (in range [0, {}])", random_value, waeli_input);
        
        // Add the random value to the array for the next iteration
        values.push(random_value);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the WebAssembly module from the WAT file
    println!("Loading WebAssembly module...");
    let wat = std::fs::read_to_string("module.wat")?;
    
    // Run the module until completion
    println!("Running module until completion...");
    let result = run(&wat)?;
    
    // Display the final result
    println!("\nFinal result = {}", result);

    Ok(())
}
