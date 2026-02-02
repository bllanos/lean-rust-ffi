import AsyncEffect

@[export sequential_main]
def sequentialMain : IO Unit := do
  let now ← AsyncEffect.IO.monotonicNow
  -- TODO print using Rust
  IO.println s!"(Sequential) Current time: {now}"
