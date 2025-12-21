;; Rejecting Handle-Message Module for error testing
;; Always rejects messages with error code 99
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader.

(module
  (memory (export "memory") 1)
  
  ;; Handle-message that always returns error code 99
  (func $handle_message (export "handle-message") (result i32)
    i32.const 99
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
