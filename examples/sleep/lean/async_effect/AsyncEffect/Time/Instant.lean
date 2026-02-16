module

public import AsyncEffect.Time.Duration

namespace AsyncEffect

namespace Time

opaque InstantPointed : NonemptyType
public def Instant : Type := InstantPointed.type
public instance instNonemptyInstant : Nonempty Instant := InstantPointed.property

namespace Instant

@[extern "async_effect_ffi_instant_subtract"]
public opaque subtract (x y : @& Instant) : Duration

end Instant

public instance instHSubInstant : HSub Instant Instant Duration where
  hSub := Instant.subtract

end Time

end AsyncEffect
