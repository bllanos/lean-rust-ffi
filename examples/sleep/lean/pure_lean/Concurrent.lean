import Std.Internal.Async

import Sleeper

open Std.Internal.IO.Async (Async)

open Sleeper (blockAndPrint sleepAndPrint)

def maximumSleepTimeSeconds : Nat := 6

def secondsDurationRange := [0:maximumSleepTimeSeconds]

def main : IO Unit := do
  IO.println "Concurrent sleep operations"

  let mut action := pure ()
  for i in secondsDurationRange do
    action := do
      let _ ← Async.concurrently action (sleepAndPrint i)

  blockAndPrint action

  IO.println "Concurrent sleep operations concurrent with sequential sleep operations"

  let ascendingAction := secondsDurationRange.forM sleepAndPrint
  let descendingAction := secondsDurationRange.forM fun x =>
    (sleepAndPrint (maximumSleepTimeSeconds - x))
  let allAction := do
    let _ ← Async.concurrentlyAll #[action, ascendingAction, descendingAction]
  blockAndPrint allAction
