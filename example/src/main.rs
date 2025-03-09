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

/// Generates a random integer in the range [0, input].
///
/// This function encapsulates the random number generation logic used by the 'waeli' host function.
///
/// # Arguments
///
/// * `input` - The upper bound (inclusive) for the random number
///
/// # Returns
///
/// A random integer in the range [0, input], or 0 if input <= 0
fn waeli(input: i32) -> i32 {
    if input <= 0 {
        return 0;
    }
    
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=input)
}

/// Executes a WebAssembly module with the 'waeli' host function.
///
/// This function:
/// 1. Sets up the wasmi environment with the 'waeli' host function
/// 2. Loads and instantiates the provided WAT module
/// 3. Calls the 'handle' function exported by the module with the specified initial value
/// 4. Returns a pair containing the result of the 'handle' function and the last input to 'waeli'
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
/// * `values` - An array of integers that can be used by the 'waeli' function
/// * `initial_value` - The initial value to pass to the 'handle' function
/// * `normal` - If true, the 'waeli' host function always uses the 'waeli' Rust function
///              and execution always completes normally. If false, the array and halting
///              behavior is used.
///
/// # Returns
///
/// A tuple containing:
/// - The result of calling the 'handle' function, or None if execution was halted
/// - The last input passed to 'waeli' if execution was halted, or 0 if execution completed normally
fn continuation(wat: &str, values: &[i32], initial_value: i32, normal: bool) -> Result<(Option<i32>, i32), Box<dyn std::error::Error>> {
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

    // Define the 'waeli' host function based on the 'normal' flag
    let waeli = if normal {
        // In 'normal' mode, always use the 'waeli' Rust function
        Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> i32 {
            // Store the last input
            caller.data_mut().last_input = input;
            
            // Call the 'waeli' Rust function
            let result = waeli(input);
            println!("  Host function waeli({}) => {} (normal mode)", input, result);
            result
        })
    } else {
        // In 'continuation' mode, use values from the array or halt execution
        Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> Result<i32, wasmi::Error> {
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
        })
    };

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

    // Call the 'handle' function with the specified initial value
    println!("  Initial acc = {}", initial_value);
    
    // Call the handle function and check if execution was halted
    let result = match handle.call(&mut store, initial_value) {
        Ok(result) => {
            // Execution completed normally
            (Some(result), 0)
        },
        Err(_) => {
            // Execution was halted by the waeli function
            if store.data().execution_halted {
                println!("  Execution halted by waeli function");
                (None, store.data().last_input)
            } else {
                // Some other error occurred
                return Err("Unexpected error during execution".into());
            }
        }
    };
    
    Ok(result)
}

/// Represents an element in the execution trace
#[derive(Debug)]
struct TraceElement {
    /// The value generated by the previous waeli call (None for the first iteration)
    previous_waeli_output: Option<i32>,
    /// The output from the continuation function (result, waeli_input)
    continuation_output: (Option<i32>, i32),
}

/// Runs the WebAssembly module until completion by dynamically generating values
/// for the 'waeli' host function as needed.
///
/// This function:
/// 1. Starts with an empty array of values
/// 2. Calls 'continuation' with the current array
/// 3. If execution halts, generates a random value and adds it to the array
/// 4. Repeats until execution completes normally
/// 5. Collects a trace of the execution
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
/// * `initial_value` - The initial value to pass to the 'handle' function
/// * `normal` - If true, runs in 'normal' mode where the 'waeli' host function
///              always uses the 'waeli' Rust function. If false, runs in 'continuation' mode.
///
/// # Returns
///
/// A tuple containing:
/// - The final result of the WebAssembly module execution
/// - A trace of the execution, with each element containing the previous waeli output
///   and the continuation output
fn run(wat: &str, initial_value: i32, normal: bool) -> Result<(i32, Vec<TraceElement>), Box<dyn std::error::Error>> {
    let mut values = Vec::new();
    let mut iteration = 0;
    let mut trace = Vec::new();
    let mut previous_waeli_output: Option<i32> = None;
    
    loop {
        iteration += 1;
        println!("\nIteration #{}", iteration);
        println!("Current values array: {:?}", values);
        
        // Call continuation with the current values, initial value, and mode
        let continuation_output = continuation(wat, &values, initial_value, normal)?;
        let (result, waeli_input) = continuation_output;
        
        // Add to the trace
        trace.push(TraceElement {
            previous_waeli_output,
            continuation_output,
        });
        
        if let Some(result) = result {
            // Execution completed normally
            println!("Execution completed with result: {}", result);
            return Ok((result, trace));
        }
        
        // Execution was halted, generate a random value using the waeli function
        println!("Execution halted with waeli_input: {}", waeli_input);
        let random_value = waeli(waeli_input);
        println!("Generated random value: {} (in range [0, {}])", random_value, waeli_input);
        
        // Store this random value for the trace
        previous_waeli_output = Some(random_value);
        
        // Add the random value to the array for the next iteration
        values.push(random_value);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Get the initial value from command line or use default (10)
    let initial_value = if args.len() > 1 {
        match args[1].parse::<i32>() {
            Ok(val) => {
                println!("Using initial value {} from command line", val);
                val
            },
            Err(_) => {
                println!("Invalid initial value '{}', using default (10)", args[1]);
                10
            }
        }
    } else {
        println!("No initial value provided, using default (10)");
        10
    };
    
    // Check if we should run in normal mode
    let normal_mode = args.len() > 2 && args[2] == "normal";
    
    // Load the WebAssembly module from the WAT file
    println!("Loading WebAssembly module...");
    let wat = std::fs::read_to_string("module.wat")?;
    
    // Run the module until completion
    if normal_mode {
        println!("Running module in NORMAL mode with initial value {}...", initial_value);
    } else {
        println!("Running module in CONTINUATION mode with initial value {}...", initial_value);
    }
    let (result, trace) = run(&wat, initial_value, normal_mode)?;
    
    // Display the final result
    println!("\nFinal result = {}", result);
    
    // Display the trace
    println!("\nExecution trace:");
    for (i, element) in trace.iter().enumerate() {
        let prev_output = match element.previous_waeli_output {
            Some(val) => val.to_string(),
            None => "None".to_string(),
        };
        let (cont_result, cont_input) = &element.continuation_output;
        let result_str = match cont_result {
            Some(val) => val.to_string(),
            None => "None".to_string(),
        };
        println!("Trace[{}]: (previous_waeli_output: {}, continuation_output: ({}, {}))",
                 i, prev_output, result_str, cont_input);
    }

    Ok(())
}
