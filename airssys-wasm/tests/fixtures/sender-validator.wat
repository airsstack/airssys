;; Sender Validator Module for testing sender parameter passing
;; Validates that sender_len > 0, returns error if sender is empty
;;
;; This fixture proves that sender metadata is actually passed to WASM.
;;
;; WASM-TASK-006 Phase 2 Task 2.2: New fixture for parameter validation

(module
  (memory (export "memory") 1)
  
  ;; Handle-message that validates sender is present
  ;; Args: sender_ptr, sender_len, message_ptr, message_len
  ;; Returns: 0 if sender_len > 0, 1 if sender is empty
  (func $handle_message (export "handle-message")
    (param $sender_ptr i32)
    (param $sender_len i32)
    (param $message_ptr i32)
    (param $message_len i32)
    (result i32)
    
    ;; Check if sender_len > 0
    (if (result i32)
      (i32.gt_s (local.get $sender_len) (i32.const 0))
      (then
        ;; Sender is valid - return success
        (i32.const 0)
      )
      (else
        ;; Sender is empty - return error code 1 (invalid sender)
        (i32.const 1)
      )
    )
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
