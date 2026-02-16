import AsyncEffect

@[export sequential_main]
def sequentialMain : IO Unit := do
  let past ← AsyncEffect.IO.monotonicNow
  let now ← AsyncEffect.IO.monotonicNow
  let elapsed := now - past
  AsyncEffect.IO.println
    s!"(Sequential) Elapsed time: {elapsed.asMilliseconds} milliseconds"
