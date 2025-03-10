//! Wasmi Example: Host Function Integration
//!
//! This example demonstrates:
//! 1. Defining a host function that can be called from WebAssembly
//! 2. Loading and executing a WebAssembly module
//! 3. Calling an exported function from the WebAssembly module
//! 4. Using freeze/thaw functionality to suspend and resume execution

use rand::Rng;
use wasmi::{
    Caller, Engine, Error, Extern, Func, Linker, Module, ResumableCall, Store, Val
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

/// Represents an element in the execution trace
#[derive(Debug)]
struct TraceElement {
    /// The input to the waeli function
    waeli_input: i32,
    /// The random value generated
    random_value: i32,
}

/// Executes a WebAssembly module with the 'waeli' host function.
///
/// This function:
/// 1. Sets up the wasmi environment with the 'waeli' host function
/// 2. Loads and instantiates the provided WAT module
/// 3. Calls the 'handle' function exported by the module with the specified initial value
/// 4. For each call to the host function, suspends execution, computes a value, and resumes
///
/// # Arguments
///
/// * `wat` - The WebAssembly Text format content as a string
/// * `initial_value` - The initial value to pass to the 'handle' function
/// * `normal` - If true, the 'waeli' host function always uses the 'waeli' Rust function
///              and execution always completes normally. If false, the freeze/thaw behavior is used.
///
/// # Returns
///
/// A tuple containing:
/// - The final result of the WebAssembly module execution
/// - A trace of the execution with all waeli calls
fn run(wat: &str, initial_value: i32, normal: bool) -> Result<(i32, Vec<TraceElement>), Box<dyn std::error::Error>> {
    // Create a new engine and store
    let engine = Engine::default();
    
    // Create the host state to track the last input
    struct HostState {
        last_input: i32,
    }
    
    let host_state = HostState {
        last_input: 0,
    };
    
    let mut store = Store::new(&engine, host_state);
    let mut linker = Linker::new(&engine);
    
    // Collect trace of all waeli calls
    let mut trace = Vec::new();

    // Define the 'waeli' host function based on the 'normal' flag
    let waeli_func = if normal {
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
        // In 'freeze/thaw' mode, always suspend execution
        Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> Result<i32, Error> {
            // Store the last input
            caller.data_mut().last_input = input;
            
            if input <= 0 {
                println!("  Host function waeli({}) => 0 (input <= 0)", input);
                return Ok(0);
            }
            
            // Always suspend execution by returning an error
            println!("  Host function waeli({}) => SUSPEND (need random value)", input);
            Err(Error::new("Execution suspended: need random value"))
        })
    };

    // Register the host function in the linker
    linker.define("env", "waeli", waeli_func)?;

    // Load the WebAssembly module from the WAT string
    let module = Module::new(&engine, wat)?;

    // Instantiate the module
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

    // Get the exported 'handle' function
    let handle = instance
        .get_export(&store, "handle")
        .and_then(Extern::into_func)
        .ok_or("Failed to find 'handle' function export")?;
    
    // Call the 'handle' function with the specified initial value
    println!("  Initial acc = {}", initial_value);
    
    // Prepare input and output buffers
    let inputs = [Val::I32(initial_value)];
    let mut outputs = [Val::I32(0)];
    
    if normal {
        // In normal mode, just call the function directly
        let typed_handle = handle.typed::<i32, i32>(&store)?;
        let result = typed_handle.call(&mut store, initial_value)?;
        println!("  Execution completed with result: {}", result);
        return Ok((result, trace));
    }
    
    // In freeze/thaw mode, use call_resumable and handle each suspension
    let mut iteration = 0;
    
    // Start the execution
    let mut current_call = handle.call_resumable(&mut store, &inputs, &mut outputs)?;
    
    // Continue execution until completion
    loop {
        match current_call {
            ResumableCall::Finished => {
                // Execution completed normally
                let result = match outputs[0] {
                    Val::I32(val) => val,
                    _ => return Err("Unexpected result type".into()),
                };
                println!("  Execution completed with result: {}", result);
                return Ok((result, trace));
            },
            ResumableCall::Resumable(invocation) => {
                // Execution was suspended, generate a random value
                iteration += 1;
                println!("\nIteration #{}", iteration);
                
                // Get the input that was passed to the waeli function
                let waeli_input = store.data().last_input;
                println!("  Execution suspended with waeli_input: {}", waeli_input);
                
                // Generate a random value
                let random_value = waeli(waeli_input);
                println!("  Generated random value: {} (in range [0, {}])", random_value, waeli_input);
                
                // Add to trace
                trace.push(TraceElement {
                    waeli_input,
                    random_value,
                });
                
                // Prepare the value to resume with
                let resume_inputs = [Val::I32(random_value)];
                
                // Resume execution with the generated random value
                println!("  Resuming execution with value: {}", random_value);
                
                // Resume execution and get the next call state
                current_call = invocation.resume(&mut store, &resume_inputs, &mut outputs)?;
            }
        }
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
        println!("Running module in FREEZE/THAW mode with initial value {}...", initial_value);
    }
    let (result, trace) = run(&wat, initial_value, normal_mode)?;
    
    // Display the final result
    println!("\nFinal result = {}", result);
    
    // Display the trace
    println!("\nExecution trace:");
    for (i, element) in trace.iter().enumerate() {
        println!("Trace[{}]: waeli({}) => {}", i, element.waeli_input, element.random_value);
    }

    Ok(())
}
