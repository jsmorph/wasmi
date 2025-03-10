use risc0_zkvm::guest::env;
use wasmi::{Caller, Engine, Extern, Func, Linker, Module, Store};

fn main() {
    // Read the WAT code and convert it to WASM
    let wat: String = env::read();
    let wasm = wat::parse_str(&wat).expect("Failed to parse WAT");
    
    // Read the initial value
    let initial_value: i32 = env::read();
    
    // Read the number of host function results
    let num_results: i32 = env::read();
    
    // Read all host function results
    let mut host_function_results = Vec::new();
    for _ in 0..num_results {
        host_function_results.push(env::read::<i32>());
    }
    
    // Create a new engine and store
    let engine = Engine::default();
    
    // Use a struct to hold our state
    struct HostState {
        last_input: i32,
        result_index: usize,
        host_function_results: Vec<i32>,
    }
    
    let host_state = HostState {
        last_input: 0,
        result_index: 0,
        host_function_results,
    };
    
    let mut store = Store::new(&engine, host_state);
    let mut linker = Linker::new(&engine);
    
    // Define the 'waeli' host function that will either return the provided result or halt execution
    let waeli = Func::wrap(&mut store, |mut caller: Caller<HostState>, input: i32| -> i32 {
        // Store the last input
        caller.data_mut().last_input = input;
        
        // Check if we have a result for this call
        let state = caller.data_mut();
        if state.result_index < state.host_function_results.len() {
            let result = state.host_function_results[state.result_index];
            state.result_index += 1;
            env::log(&format!("  Host function waeli({}) => {} (from previous run)", input, result));
            return result;
        }
        
        // Otherwise, halt execution
        env::log(&format!("  Host function waeli({}) called - halting execution", input));
        
        // Commit the input to the journal so the host can read it
        env::commit(&input);
        
        // Halt the execution by panicking
        // The host will need to provide the result when resuming
        panic!("HALT_FOR_HOST_FUNCTION");
    });
    
    // Register the host function in the linker
    linker.define("env", "waeli", waeli).expect("Failed to define host function");
    
    // Load the WebAssembly module
    let module = Module::new(&engine, &wasm[..]).expect("Failed to create module");
    
    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate")
        .start(&mut store)
        .expect("Failed to start");
    
    // Get the exported 'handle' function
    let handle = instance
        .get_export(&store, "handle")
        .and_then(Extern::into_func)
        .expect("Failed to find 'handle' function export")
        .typed::<i32, i32>(&store)
        .expect("Failed to type 'handle' function");
    
    // Call the 'handle' function with the initial value
    env::log(&format!("  Initial acc = {}", initial_value));
    
    let result = handle.call(&mut store, initial_value).expect("Failed to call 'handle'");
    
    env::log(&format!("  Final result = {}", result));
    
    // Commit the result to the journal
    env::commit(&result);
}
