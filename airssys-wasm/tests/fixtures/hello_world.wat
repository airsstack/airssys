;; Minimal WebAssembly Component for testing
;; Simple arithmetic component - no memory, no imports required

(component
  (core module $M
    ;; Export a simple function that returns 42
    (func $hello (export "hello") (result i32)
      i32.const 42
    )
  )
  
  ;; Instantiate the core module
  (core instance $m (instantiate $M))
  
  ;; Lift the core function to a component function
  ;;  () -> s32 signature (no parameters, returns signed 32-bit integer)
  (func (export "hello") (result s32)
    (canon lift (core func $m "hello"))
  )
)
