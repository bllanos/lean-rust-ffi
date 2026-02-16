import AsyncEffect

@[export concurrent_main]
def concurrentMain : IO Unit := do
  let past ← AsyncEffect.IO.monotonicNow
  let now ← AsyncEffect.IO.monotonicNow
  let elapsed := now - past
  AsyncEffect.IO.println
    s!"(Concurrent) Elapsed time: {elapsed.asMilliseconds} milliseconds"
