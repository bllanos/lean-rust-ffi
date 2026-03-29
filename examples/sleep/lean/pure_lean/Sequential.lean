import Sleeper

open Sleeper (blockAndPrint sleepAndPrint)

def maximumSleepTimeSeconds : Nat := 6

def secondsDurationRange := [0:maximumSleepTimeSeconds]

def main : IO Unit := do
  IO.println "Sequential sleep operations"
  let ascendingAction := secondsDurationRange.forM sleepAndPrint
  let descendingAction := secondsDurationRange.forM fun x =>
    (sleepAndPrint (maximumSleepTimeSeconds - x))
  let action := ascendingAction >>= fun () => descendingAction
  blockAndPrint action
