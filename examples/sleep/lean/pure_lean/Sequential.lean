import Std.Internal.Async.Timer
import Std.Time

open Std.Internal.IO.Async (Async sleep)
open Std.Time

def secondsDurationRange := [0:5]

def waitThenPrint (x : Nat) : Async Unit := do
  sleep (Millisecond.Offset.ofSeconds (Second.Offset.ofNat x))
  IO.println s!"Counted {x} seconds"

def main : IO Unit := do
  let action := secondsDurationRange.forM waitThenPrint
  Async.block action
  pure ()
