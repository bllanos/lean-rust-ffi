import AsyncEffect

import Sleeper

open AsyncEffect.IO (EAsyncIO)

open Sleeper (blockAndPrint sleepAndPrint sleepAndPrintError)

@[export short_circuit_main]
def shortCircuitMain : IO Unit := do
  AsyncEffect.IO.println "Short-circuiting sleep operations on errors"

  let action1 := sleepAndPrint 1
  let action2 := sleepAndPrint 2
  let action3 := sleepAndPrint 3

  let action1Error := sleepAndPrintError 1
  let action2Error := sleepAndPrintError 2

  AsyncEffect.IO.println "\nPairs of actions:"

  AsyncEffect.IO.println "\nError first, shorter sleep first"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrently action1Error action2)
  )

  AsyncEffect.IO.println "\nError first, shorter sleep second"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrently action2Error action1)
  )

  AsyncEffect.IO.println "\nError second, shorter sleep first"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrently action1 action2Error)
  )

  AsyncEffect.IO.println "\nError second, shorter sleep second"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrently action2 action1Error)
  )

  AsyncEffect.IO.println "\nArrays of action with the error in the middle"

  AsyncEffect.IO.println "\nAscending sleep durations"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrentlyAll #[action1, action2Error, action3])
  )

  AsyncEffect.IO.println "\nDescending sleep durations"
  blockAndPrint (do
    let _ ← (EAsyncIO.concurrentlyAll #[action3, action2Error, action1])
  )
