;; Minimal WebAssembly Component Model component for testing
;; Returns a constant value from a simple function

(component
  (core module $m
    (func (export "hello") (result i32)
      i32.const 42
    )
  )
  
  (core instance $i (instantiate $m))
  
  (func (export "hello") (result s32)
    (canon lift (core func $i "hello"))
  )
)
