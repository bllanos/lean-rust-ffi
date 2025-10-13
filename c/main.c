#include <stdio.h>
#include <lean/lean.h>

extern void lean_initialize_runtime_module();
// This would replace `lean_initialize_runtime_module()` if the code accesses the `Lean` package
// extern void lean_initialize();
extern void lean_io_mark_end_initialization();

extern lean_object* initialize_MapArray(uint8_t builtin, lean_object *w);
extern lean_object* mk_map_options(uint32_t addend, uint32_t multiplicand);
extern lean_object* map_options_to_string(lean_object*);
extern lean_object* my_map(lean_object *options, lean_object *arr);

int main() {
  printf("Program start\n");

  // Lean initialization
  // -------------------
  lean_initialize_runtime_module();

  // Lean module initialization
  // --------------------------
  lean_object* res;
  // Use same default as for Lean executables
  // See https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md#initialization
  uint8_t builtin = 1;
  res = initialize_MapArray(builtin, lean_io_mk_world());
  if (lean_io_result_is_ok(res)) {
      lean_dec(res);
  } else {
      lean_io_result_show_error(res);
      lean_dec(res);
      return 1;  // do not access Lean declarations if initialization failed
  }
  lean_io_mark_end_initialization();

  // Program logic
  // -------------

  uint32_t addend = 2;
  uint32_t multiplicand = 3;

  lean_object* map_options = mk_map_options(addend, multiplicand);
  // Avoid having `map_options_to_string()` destroy `map_options`
  lean_inc(map_options);
  lean_object* map_options_lean_str = map_options_to_string(map_options);
  // Strings in Lean are null-terminated
  char const * map_options_cstr = lean_string_cstr(map_options_lean_str);
  printf("MapOptions instance: %s\n", map_options_cstr);
  // This seems to be an alternative to `lean_dec()` that can be used when
  // the value is known not to be a scalar.
  lean_dec_ref(map_options_lean_str);

  size_t arr_size = 6;
  lean_object* arr = lean_alloc_array(arr_size, arr_size);
  lean_object ** arr_data = lean_array_cptr(arr);

  printf("Populating input array: [ ");
  for(size_t i = 0; i < arr_size; ++i) {
    assert(i * 5 < 256); // Ensure values fit in `uint8_t`
    uint8_t value = i * 5;
    // There are no functions for boxing `uint8_t` values specifically, so use
    // `lean_box_uint32()`
    *(arr_data + i) = lean_box_uint32(value);
    printf("%d, ", value);
  }
  printf("]\n");

  // Note: `my_map()` will call `lean_dec()` on all arguments.
  lean_object * arr_out = my_map(map_options, arr);
  arr_data = lean_array_cptr(arr_out);
  arr_size = lean_array_size(arr_out);

  printf("Output array: [ ");
  for(size_t i = 0; i < arr_size; ++i) {
    int32_t value = (int32_t) lean_unbox_uint32(*(arr_data + i));
    printf("%d, ", value);
  }
  printf("]\n");

  lean_dec_ref(arr_out);

  printf("Program end\n");
}
