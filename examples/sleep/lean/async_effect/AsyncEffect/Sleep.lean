module

public import AsyncEffect.Time.Duration

namespace AsyncEffect

open AsyncEffect.Time (Duration)

-- Lean representation of a sleep duration
--
-- The FFI representation is the same as `Duration` because the constructor has
-- a single field.
-- See <https://lean-lang.org/functional_programming_in_lean/Programming___-Proving___-and-Performance/Special-Types/#runtime-special-types>
public structure Sleep where
  private duration : Duration

namespace Sleep

public def from_duration (duration : Duration) : Sleep :=
  Sleep.mk duration

end Sleep

public instance instCoeDurationSleep : Coe Duration Sleep where
  coe := Sleep.from_duration

namespace Sleep

public def fromSeconds (seconds: UInt64) : Sleep :=
  Duration.fromSeconds seconds

public def fromMilliseconds (milliseconds: UInt64) : Sleep :=
  Duration.fromMilliseconds milliseconds

public def zero : Sleep := Duration.zero

end Sleep

public instance instInhabitedSleep : Inhabited Sleep where
  default := Sleep.zero

end AsyncEffect
