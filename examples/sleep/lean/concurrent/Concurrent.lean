import AsyncEffect

import Sleeper

open AsyncEffect.IO (EAsyncIO)

open Sleeper (blockAndPrint sleepAndPrint)

def maximumSleepTimeSeconds : Nat := 6

def secondsDurationRange := [0:maximumSleepTimeSeconds]

@[export concurrent_main]
def concurrentMain : IO Unit := do
  AsyncEffect.IO.println "Concurrent sleep operations"

  let mut action := pure ()
  for i in secondsDurationRange do
    action := do
      let _ ← EAsyncIO.concurrently action (sleepAndPrint i)

  blockAndPrint action

  AsyncEffect.IO.println "Concurrent sleep operations concurrent with sequential sleep operations"

  let ascendingAction := secondsDurationRange.forM sleepAndPrint
  let descendingAction := secondsDurationRange.forM fun x =>
    (sleepAndPrint (maximumSleepTimeSeconds - x))
  let allAction := do
    let _ ← EAsyncIO.concurrentlyAll #[action, ascendingAction, descendingAction]
  blockAndPrint allAction
