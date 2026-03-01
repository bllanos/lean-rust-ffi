module

public import AsyncEffect.Time.Duration

namespace AsyncEffect

open AsyncEffect.Time (Duration)

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
