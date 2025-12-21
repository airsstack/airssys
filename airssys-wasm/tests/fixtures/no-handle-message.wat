;; Module without handle-message export for testing
;; Only has a hello function, no handle-message export
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader. Used to test error handling when
;; handle-message export is missing.

(module
  (memory (export "memory") 1)
  
  ;; Simple hello function that returns 42
  (func $hello (export "hello") (result i32)
    i32.const 42
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
