;; Slow Handle-Message Module for timeout testing
;; Delays via busy loop before responding
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader. The delay loop consumes fuel, which
;; is used to enforce execution limits.

(module
  (memory (export "memory") 1)
  
  ;; Busy loop for delay (consumes fuel)
  (func $delay (param $n i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (loop $L
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $L (i32.lt_u (local.get $i) (local.get $n)))
    )
  )
  
  ;; Handle-message that delays then returns success
  (func $handle_message (export "handle-message") (result i32)
    ;; Large loop count to consume significant fuel
    (call $delay (i32.const 50000000))
    i32.const 0
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
