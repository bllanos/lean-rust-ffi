import AsyncEffect

@[export concurrent_main]
def concurrentMain : IO Unit := do
  let now ← AsyncEffect.IO.monotonicNow
  AsyncEffect.IO.println s!"(Concurrent) Current time: {now}"
