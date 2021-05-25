(module
  (type $type0 (func (result i32)))
  (table 0 anyfunc)
  (memory 1)
  (export "memory" memory)
  (export "main" $func0)
  (func $func0 (result i32)
    (local $var0 i32) 
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local $var0
    i32.const 0
    i32.store offset=12
    get_local $var0
    i32.const 5
    i32.store offset=8
    get_local $var0
    i32.const 4
    i32.store offset=4
    get_local $var0
    i32.load offset=8
    i32.const 4
    i32.add
  )
)