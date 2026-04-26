module

public import AsyncEffect

open AsyncEffect (Sleep)
open AsyncEffect.Time (Duration Instant)
open AsyncEffect.IO (EAsyncIO println sleep)

namespace Sleeper

def formatElapsedTime (startTime: Instant) (endTime: Instant) :
    Std.Time.Second.Offset × Std.Time.Millisecond.Offset :=
  let elapsed := (endTime - startTime)
  let elapsedTimeSeconds := elapsed.asSeconds
  let elapsedTimeRemainder := elapsed - (Duration.fromSeconds elapsed.asSecondsNumber)
  (elapsedTimeSeconds, elapsedTimeRemainder.asMilliseconds)

public def sleepAndPrint (sleepDurationSeconds : Nat) : EAsyncIO IO.Error Unit := do
  let startTime ← AsyncEffect.IO.monotonicNow
  sleep (AsyncEffect.Sleep.fromSeconds sleepDurationSeconds.toUInt64)
  let endTime ← AsyncEffect.IO.monotonicNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  println s!"Called sleep for {sleepDurationSeconds} seconds (actual sleep duration {elapsedTimeSeconds} seconds {elapsedTimeRemainder} milliseconds)"

public def sleepAndPrintError (sleepDurationSeconds : Nat) : EAsyncIO IO.Error Unit := do
  sleepAndPrint sleepDurationSeconds
  println s!"Raising error after {sleepDurationSeconds} seconds sleep call"
  throw (IO.userError s!"Error after {sleepDurationSeconds} seconds sleep call")

public def blockAndPrint (action: EAsyncIO IO.Error Unit) : IO Unit := do
  let startTime ← AsyncEffect.IO.monotonicNow
  EAsyncIO.block action
  let endTime ← AsyncEffect.IO.monotonicNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  println s!"Total duration {elapsedTimeSeconds} seconds {elapsedTimeRemainder} milliseconds"

end Sleeper
