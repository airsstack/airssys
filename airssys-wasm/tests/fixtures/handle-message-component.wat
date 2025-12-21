;; Component Model fixture with handle-message export
;; Used for testing WasmEngine::call_handle_message()
;;
;; Exports:
;;   handle-message: func(sender: string, message: list<u8>) -> result
;;
;; This uses a simplified signature (string instead of component-id record)
;; to match the initial call_handle_message() implementation.

(component
  (core module $M
    ;; Memory with 2 pages (128KB) to ensure enough space
    (memory (export "memory") 2)
    
    ;; Heap pointer starts at 4KB to leave room for initial memory
    (global $heap_ptr (mut i32) (i32.const 4096))
    
    ;; Realloc function required by Component Model for string/list handling
    ;; This is a simple bump allocator
    (func $realloc (export "cabi_realloc")
      (param $old_ptr i32)
      (param $old_size i32)
      (param $align i32)
      (param $new_size i32)
      (result i32)
      
      (local $ptr i32)
      (local $aligned_ptr i32)
      
      ;; If new_size is 0, just return 0
      (if (i32.eqz (local.get $new_size))
        (then (return (i32.const 0)))
      )
      
      ;; Get current heap pointer
      (local.set $ptr (global.get $heap_ptr))
      
      ;; Align the pointer (align must be power of 2)
      ;; aligned = (ptr + align - 1) & ~(align - 1)
      (local.set $aligned_ptr
        (i32.and
          (i32.add 
            (local.get $ptr) 
            (i32.sub (local.get $align) (i32.const 1))
          )
          (i32.xor 
            (i32.sub (local.get $align) (i32.const 1)) 
            (i32.const -1)
          )
        )
      )
      
      ;; Update heap pointer to after the allocation
      (global.set $heap_ptr 
        (i32.add (local.get $aligned_ptr) (local.get $new_size))
      )
      
      ;; Return the aligned pointer
      (local.get $aligned_ptr)
    )
    
    ;; Handle-message implementation
    ;; Takes lowered parameters: sender_ptr, sender_len, message_ptr, message_len
    ;; Returns 0 for success (Ok variant in result type)
    (func $handle_message (export "handle-message")
      (param $sender_ptr i32)
      (param $sender_len i32)
      (param $message_ptr i32)
      (param $message_len i32)
      (result i32)
      
      ;; Return 0 = Ok(())
      ;; In Component Model result encoding: 0 means success/Ok variant
      i32.const 0
    )
  )
  
  ;; Instantiate the core module
  (core instance $m (instantiate $M))
  
  ;; Alias core exports for use in canon lift
  (alias core export $m "memory" (core memory $memory))
  (alias core export $m "cabi_realloc" (core func $realloc))
  
  ;; Lift handle-message to Component Model signature
  ;; (string, list<u8>) -> result
  (func (export "handle-message")
    (param "sender" string)
    (param "message" (list u8))
    (result (result))
    (canon lift (core func $m "handle-message")
      (memory $memory)
      (realloc (func $realloc))
    )
  )
)
