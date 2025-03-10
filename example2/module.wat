;; WebAssembly module for the wasmi example
;;
;; This module:
;; 1. Imports a host function 'waeli' that takes an int and returns a random int in range [0, input]
;; 2. Exports a function 'handle' that uses the host function to perform calculations
;;
;; The 'handle' function logic:
;; - Initialize acc = input
;; - Call waeli(acc) and check if result is even:
;;   - If even: acc += result
;;   - If odd: acc -= result
;; - Call waeli(acc) again and check if result is even:
;;   - If even: acc += result
;;   - If odd: acc -= result
;; - If acc is odd, make a final call to waeli(acc) and add the result to acc
;; - Return acc

(module
  ;; Import the host function 'waeli' from the 'env' namespace
  (import "env" "waeli" (func $waeli (param i32) (result i32)))
  
  ;; Export the 'handle' function that takes an i32 and returns an i32
  (func $handle (export "handle") (param i32) (result i32)
    ;; Initialize local variables
    (local $acc i32)
    (local $waeli_result i32)
    
    ;; Initialize 'acc' to be the int input to 'handle'
    (local.set $acc (local.get 0))
    
    ;; Call waeli(acc)
    (local.set $waeli_result (call $waeli (local.get $acc)))
    
    ;; If the result is even, add it to acc. Otherwise subtract it from acc.
    (if (i32.eqz (i32.and (local.get $waeli_result) (i32.const 1)))
      (then
        ;; Even case: acc += waeli_result
        (local.set $acc (i32.add (local.get $acc) (local.get $waeli_result)))
      )
      (else
        ;; Odd case: acc -= waeli_result
        (local.set $acc (i32.sub (local.get $acc) (local.get $waeli_result)))
      )
    )
    
    ;; Call waeli(acc) again
    (local.set $waeli_result (call $waeli (local.get $acc)))
    
    ;; If the result is even, add it to acc. Otherwise subtract it from acc.
    (if (i32.eqz (i32.and (local.get $waeli_result) (i32.const 1)))
      (then
        ;; Even case: acc += waeli_result
        (local.set $acc (i32.add (local.get $acc) (local.get $waeli_result)))
      )
      (else
        ;; Odd case: acc -= waeli_result
        (local.set $acc (i32.sub (local.get $acc) (local.get $waeli_result)))
      )
    )
    
    ;; Make a final call to waeli only if acc is odd
    (if (i32.and (local.get $acc) (i32.const 1))
      (then
        ;; Acc is odd, call waeli(acc)
        (local.set $waeli_result (call $waeli (local.get $acc)))
        ;; Add the result to acc
        (local.set $acc (i32.add (local.get $acc) (local.get $waeli_result)))
      )
    )
    
    ;; Return acc
    (local.get $acc)
  )
)
