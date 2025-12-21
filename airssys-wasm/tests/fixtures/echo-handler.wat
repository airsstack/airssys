;; Echo Handler Module for testing payload parameter passing
;; Reads the first byte of the message and returns it as the result code.
;; This proves that the message payload is actually passed to WASM and readable.
;;
;; WASM-TASK-006 Phase 2 Task 2.2: New fixture for payload validation

(module
  (memory (export "memory") 1)
  
  ;; Handle-message that echoes first byte of message as result
  ;; Args: sender_ptr, sender_len, message_ptr, message_len
  ;; Returns: 
  ;;   - If message_len == 0: returns 255 (empty message indicator)
  ;;   - Otherwise: returns first byte of message (0-254 = success indicator)
  (func $handle_message (export "handle-message")
    (param $sender_ptr i32)
    (param $sender_len i32)
    (param $message_ptr i32)
    (param $message_len i32)
    (result i32)
    
    ;; Check if message is empty
    (if (result i32)
      (i32.eq (local.get $message_len) (i32.const 0))
      (then
        ;; Empty message - return 255 as indicator
        (i32.const 255)
      )
      (else
        ;; Read and return first byte of message from memory
        ;; This proves we can actually read the payload from WASM memory
        (i32.load8_u (local.get $message_ptr))
      )
    )
  )
  
  ;; Optional _start for initialization
  (func (export "_start"))
)
