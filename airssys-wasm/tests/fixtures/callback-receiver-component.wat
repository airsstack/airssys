;; Component Model fixture with handle-callback export
;; Used for testing WasmEngine::call_handle_callback()
;;
;; Exports:
;;   handle-callback: func(request_id: string, result: list<u8>) -> s32
;;
;; WASM-TASK-006 Phase 3 Task 3.2: Response Routing and Callbacks

(component
  (core module $M
    ;; Memory with 2 pages (128KB) to ensure enough space
    (memory (export "memory") 2)
    
    ;; Heap pointer starts at 4KB to leave room for initial memory
    (global $heap_ptr (mut i32) (i32.const 4096))
    
    ;; Storage for test verification
    (global $last_request_id_len (mut i32) (i32.const 0))
    (global $last_result_len (mut i32) (i32.const 0))
    (global $last_result_type (mut i32) (i32.const -1))
    (global $callback_count (mut i32) (i32.const 0))
    
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
    
    ;; Handle-callback implementation
    ;; Takes lowered parameters: request_id_ptr, request_id_len, result_ptr, result_len
    ;; Returns 0 for success
    (func $handle_callback (export "handle-callback")
      (param $request_id_ptr i32)
      (param $request_id_len i32)
      (param $result_ptr i32)
      (param $result_len i32)
      (result i32)
      
      ;; Store lengths for verification
      (global.set $last_request_id_len (local.get $request_id_len))
      (global.set $last_result_len (local.get $result_len))
      
      ;; Increment callback count
      (global.set $callback_count 
        (i32.add (global.get $callback_count) (i32.const 1)))
      
      ;; Determine result type from first byte if result_len > 0
      (if (i32.gt_s (local.get $result_len) (i32.const 0))
        (then
          ;; Read first byte as result type indicator
          ;; 0 = success, 1 = error
          (global.set $last_result_type
            (i32.load8_u (local.get $result_ptr)))
        )
        (else
          ;; Empty result - treat as success (0)
          (global.set $last_result_type (i32.const 0))
        )
      )
      
      ;; Return 0 for success
      (i32.const 0)
    )
  )
  
  ;; Instantiate the core module
  (core instance $m (instantiate $M))
  
  ;; Alias core exports for use in canon lift
  (alias core export $m "memory" (core memory $memory))
  (alias core export $m "cabi_realloc" (core func $realloc))
  
  ;; Lift handle-callback to Component Model signature
  ;; (string, list<u8>) -> s32
  (func (export "handle-callback")
    (param "request-id" string)
    (param "result" (list u8))
    (result s32)
    (canon lift (core func $m "handle-callback")
      (memory $memory)
      (realloc (func $realloc))
    )
  )
)
