;; Rejecting Handle-Message Module for error testing
;; Always rejects messages with error code 99
;;
;; This is a core WASM module (not Component Model) for use with
;; wasmtime's Module loader. The handle-message function takes four i32
;; parameters (sender_ptr, sender_len, message_ptr, message_len) and 
;; returns i32 (0 = success, non-zero = error).
;;
;; WASM-TASK-006 Phase 2 Task 2.2: Updated to accept parameters

(module
  (memory (export "memory") 1)
  
  ;; Handle-message that always returns error code 99
  ;; Args: sender_ptr, sender_len, message_ptr, message_len
  ;; Returns: 99 (rejection error)
  (func $handle_message (export "handle-message")
    (param $sender_ptr i32)
    (param $sender_len i32)
    (param $message_ptr i32)
    (param $message_len i32)
    (result i32)
    
    ;; Always reject with error code 99
    i32.const 99
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
