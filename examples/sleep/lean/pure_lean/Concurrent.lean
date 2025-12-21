import Std.Internal.Async

import Sleeper

open Std.Internal.IO.Async (Async)

open Sleeper (blockAndPrint formatElapsedTime sleepAndPrint)

def secondsDurationRange := [0:6]

def main : IO Unit := do
  IO.println "Concurrent sleep operations"

  let mut action := pure ()
  for i in secondsDurationRange do
    action := do
      let _ ← Async.concurrently action (Sleeper.sleepAndPrint i)

  blockAndPrint action
