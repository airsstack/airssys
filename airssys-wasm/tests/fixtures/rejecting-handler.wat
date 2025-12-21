;; Rejecting Handle-Message Component for error testing
;; Always rejects messages with error code 99

(component
  (core module $M
    (memory (export "memory") 1)
    
    (func $handle_msg (export "handle-message") (param i32 i32 i32) (result i32)
      i32.const 99
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
