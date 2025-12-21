import Std.Internal.Async.Timer
import Std.Time

open Std.Internal.IO.Async (Async sleep)
open Std.Time

namespace Sleeper

def formatElapsedTime (startTime: Nat) (endTime: Nat) :
    Second.Offset × Millisecond.Offset :=
  let elapsedTime := Millisecond.Offset.ofNat (endTime - startTime)
  let elapsedTimeSeconds := elapsedTime.toSeconds
  let elapsedTimeRemainder := elapsedTime - elapsedTimeSeconds
  (elapsedTimeSeconds, elapsedTimeRemainder)

def sleepAndPrint (x : Nat) : Async Unit := do
  let startTime ← IO.monoMsNow
  sleep (Millisecond.Offset.ofSeconds (Second.Offset.ofNat x))
  let endTime ← IO.monoMsNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  IO.println s!"Called sleep for {x} seconds (actual sleep duration {elapsedTimeSeconds}.{elapsedTimeRemainder} seconds)"

def blockAndPrint (action: Async Unit) : IO Unit := do
  let startTime ← IO.monoMsNow
  Async.block action
  let endTime ← IO.monoMsNow
  let (elapsedTimeSeconds, elapsedTimeRemainder) := formatElapsedTime startTime endTime
  IO.println s!"Total duration {elapsedTimeSeconds}.{elapsedTimeRemainder} seconds"

end Sleeper
