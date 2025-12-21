;; Slow Handle-Message Component for timeout testing
;; Delays ~500ms before responding

(component
  (core module $M
    (memory (export "memory") 1)
    
    (func $delay (param $n i32)
      (local $i i32)
      (local.set $i (i32.const 0))
      (loop $L
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br_if $L (i32.lt_u (local.get $i) (local.get $n)))
      )
    )
    
    (func $handle_msg (export "handle-message") (param i32 i32 i32) (result i32)
      (call $delay (i32.const 50000000))
      i32.const 0
    )
    
    (func (export "_start"))
  )
  
  (core instance $m (instantiate $M))
  
  (func (export "handle-message") (result u32)
    (canon lift (core func $m "handle-message"))
  )
  
  (func (export "_start")
    (canon lift (core func $m "_start"))
  )
)
