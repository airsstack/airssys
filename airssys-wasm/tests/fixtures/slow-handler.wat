;; Slow Handle-Message Module for timeout testing
;; Delays via busy loop before responding
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader. The delay loop consumes fuel, which
;; is used to enforce execution limits.
;;
;; WASM-TASK-006 Phase 2 Task 2.2: Updated to accept parameters

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
  ;; Args: sender_ptr, sender_len, message_ptr, message_len
  ;; Returns: 0 on success (after delay)
  (func $handle_message (export "handle-message")
    (param $sender_ptr i32)
    (param $sender_len i32)
    (param $message_ptr i32)
    (param $message_len i32)
    (result i32)
    
    ;; Large loop count to consume significant fuel
    (call $delay (i32.const 50000000))
    i32.const 0
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
