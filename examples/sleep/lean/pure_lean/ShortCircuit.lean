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

  try
    blockAndPrint (do
      let _ ← (Async.concurrently action1Error action2)
    )
  catch e =>
    IO.println e

  IO.println "\nError first, shorter sleep second"

  try
    blockAndPrint (do
      let _ ← (Async.concurrently action2Error action1)
    )
  catch e =>
    IO.println e

  IO.println "\nError second, shorter sleep first"

  try
    blockAndPrint (do
      let _ ← (Async.concurrently action1 action2Error)
    )
  catch e =>
    IO.println e

  IO.println "\nError second, shorter sleep second"

  try
    blockAndPrint (do
      let _ ← (Async.concurrently action2 action1Error)
    )
  catch e =>
    IO.println e

  IO.println "\nArrays of action with the error in the middle"

  IO.println "\nAscending sleep durations"

  try
    blockAndPrint (do
      let _ ← (Async.concurrentlyAll #[action1, action2Error, action3])
    )
  catch e =>
    IO.println e

  IO.println "\nDescending sleep durations"

  try
    blockAndPrint (do
      let _ ← (Async.concurrentlyAll #[action3, action2Error, action1])
    )
  catch e =>
    IO.println e
