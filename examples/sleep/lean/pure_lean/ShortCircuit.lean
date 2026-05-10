import Std.Internal.Async

import Sleeper

open Std.Internal.IO.Async (Async)

open Sleeper (blockAndPrint sleepAndPrint sleepAndPrintError)

def main : IO Unit := do
  IO.println "Short-circuiting sleep operations on errors"

  let action1 := sleepAndPrint 1
  let action2 := sleepAndPrint 2
  let action3 := sleepAndPrint 3

  let action1Error := sleepAndPrintError 1
  let action2Error := sleepAndPrintError 2

  IO.println "\nPairs of actions:"

  IO.println "\nError first, shorter sleep first"
  blockAndPrint (do
    let _ ← (Async.concurrently action1Error action2)
  )

  IO.println "\nError first, shorter sleep second"
  blockAndPrint (do
    let _ ← (Async.concurrently action2Error action1)
  )

  IO.println "\nError second, shorter sleep first"
  blockAndPrint (do
    let _ ← (Async.concurrently action1 action2Error)
  )

  IO.println "\nError second, shorter sleep second"
  blockAndPrint (do
    let _ ← (Async.concurrently action2 action1Error)
  )

  IO.println "\nArrays of action with the error in the middle"

  IO.println "\nAscending sleep durations"
  blockAndPrint (do
    let _ ← (Async.concurrentlyAll #[action1, action2Error, action3])
  )

  IO.println "\nDescending sleep durations"
  blockAndPrint (do
    let _ ← (Async.concurrentlyAll #[action3, action2Error, action1])
  )
