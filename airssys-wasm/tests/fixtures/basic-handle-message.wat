;; Basic Handle-Message Module for testing
;; Accepts messages and returns success (0 = success)
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader. The handle-message function takes no
;; parameters and returns i32 (0 = success, non-zero = error).

(module
  (memory (export "memory") 1)
  
  ;; Simple handle-message that always returns success (0)
  (func $handle_message (export "handle-message") (result i32)
    i32.const 0
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
