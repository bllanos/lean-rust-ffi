module

public import Std.Time

namespace AsyncEffect

namespace Time

opaque DurationPointed : NonemptyType
public def Duration : Type := DurationPointed.type
public instance instNonemptyDuration : Nonempty Duration := DurationPointed.property

namespace Duration

@[extern "async_effect_ffi_duration_as_secs"]
public opaque asSecondsNumber (d : @& Duration) : UInt64

public def asSeconds (d : Duration) : Std.Time.Second.Offset :=
  Std.Time.Second.Offset.ofNat d.asSecondsNumber.toNat

@[extern "async_effect_ffi_duration_from_secs"]
public opaque fromSeconds (s : UInt64) : Duration

@[extern "async_effect_ffi_duration_as_millis"]
opaque asMillisecondsNumber (d : @& Duration) : Nat

public def asMilliseconds (d : Duration) : Std.Time.Millisecond.Offset :=
  Std.Time.Millisecond.Offset.ofNat d.asMillisecondsNumber

@[extern "async_effect_ffi_duration_subtract"]
public opaque subtract (x y : @& Duration) : Duration

public def zero : Duration := fromSeconds 0

end Duration

public instance instSubDuration : Sub Duration where
  sub := Duration.subtract

public instance instInhabitedDuration : Inhabited Duration where
  default := Duration.zero

end Time

end AsyncEffect
