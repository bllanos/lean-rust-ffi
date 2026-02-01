module

public import Std.Internal.Async
import Std.Internal.Async.Timer
import Std.Time

open Std.Internal.IO.Async (Async sleep)
open Std.Time

namespace Sleeper

def formatElapsedTime (startTimeMilliseconds: Nat) (endTimeMilliseconds: Nat) :
    Second.Offset × Millisecond.Offset :=
  let elapsedTime := Millisecond.Offset.ofNat (endTimeMilliseconds - startTimeMilliseconds)
  let elapsedTimeSeconds := elapsedTime.toSeconds
  let elapsedTimeRemainder := elapsedTime - elapsedTimeSeconds
  (elapsedTimeSeconds, elapsedTimeRemainder)

public def sleepAndPrint (sleepDurationSeconds : Nat) : Async Unit := do
  let startTime ← IO.monoMsNow
  sleep (Millisecond.Offset.ofSeconds (Second.Offset.ofNat sleepDurationSeconds))
  let endTime ← IO.monoMsNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  IO.println s!"Called sleep for {sleepDurationSeconds} seconds (actual sleep duration {elapsedTimeSeconds} seconds {elapsedTimeRemainder} milliseconds)"

public def blockAndPrint (action: Async Unit) : IO Unit := do
  let startTime ← IO.monoMsNow
  Async.block action
  let endTime ← IO.monoMsNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  IO.println s!"Total duration {elapsedTimeSeconds} seconds {elapsedTimeRemainder} milliseconds"

end Sleeper
