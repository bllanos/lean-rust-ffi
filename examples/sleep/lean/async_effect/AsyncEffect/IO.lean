module

public import AsyncEffect.Time

open AsyncEffect.Time (Instant)

namespace AsyncEffect

namespace IO

@[extern "async_effect_ffi_monotonic_now_immediate"]
public opaque monotonicNow : BaseIO Instant

@[extern "async_effect_ffi_println_immediate"]
public opaque printlnString (s : @& String) : IO Unit

/--
Converts `s` to a string using its `ToString α` instance, and prints it with a
trailing newline to standard output
-/
public def println [ToString α] (s : α) : IO Unit := do
  printlnString (toString s)

end IO

end AsyncEffect
