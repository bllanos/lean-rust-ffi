module

public import AsyncEffect.Sleep

namespace AsyncEffect

namespace IO

public opaque AsyncIOPointed (α : Type) : NonemptyType
@[expose]
public def AsyncIO (α : Type) : Type := (AsyncIOPointed α).type
public instance instNonemptyAsyncIO (α : Type) : Nonempty (AsyncIO α) :=
  (AsyncIOPointed α).property

namespace AsyncIO

@[extern "async_effect_ffi_async_io_pure"]
public opaque pure (a : α) : AsyncIO α

@[extern "async_effect_ffi_async_io_map"]
public opaque map (f : α → β) (self : AsyncIO α) : AsyncIO β

@[extern "async_effect_ffi_async_io_bind"]
public opaque bind (self : AsyncIO α) (f : α → AsyncIO β) : AsyncIO β

@[extern "async_effect_ffi_async_io_concurrently"]
public opaque concurrently (x :  AsyncIO α) (y :  AsyncIO β) : AsyncIO (α × β)

public def concurrentlyAll (xs : Array (AsyncIO α)) :  AsyncIO (Array α) :=
  xs.foldl (fun accumulator item =>
    (concurrently accumulator item).map (fun (arr, value) =>
      arr.push value
    )) (pure #[])

@[extern "async_effect_ffi_async_io_lift_base_io"]
public opaque lift (x :  BaseIO α) : AsyncIO α

@[extern "async_effect_ffi_async_io_block"]
public opaque block [Inhabited α] (self : AsyncIO α) : BaseIO α

end AsyncIO

public instance instFunctorAsyncIO : Functor AsyncIO where
  map := AsyncIO.map

public instance instMonadAsyncIO : Monad AsyncIO where
  pure := AsyncIO.pure
  bind := AsyncIO.bind

public instance instMonadLiftBaseIOAsyncIO : MonadLift BaseIO AsyncIO where
  monadLift := AsyncIO.lift

public instance instInhabitedAsyncIO [Inhabited α] : Inhabited (AsyncIO α) where
  default := pure default

@[expose]
public def EAsyncIO (ε : Type) (α : Type) := AsyncIO (Except ε α)

public instance instMonadLiftAsyncIOEAsyncIO : MonadLift AsyncIO (EAsyncIO ε) where
  monadLift x := x.map pure

namespace EAsyncIO

public def pure (a : α) : EAsyncIO ε α :=
  AsyncIO.pure (.ok a)

public def map (f : α → β) (self : EAsyncIO ε α) : EAsyncIO ε β :=
  AsyncIO.map (.map f) self

public def bind (self : EAsyncIO ε α) (f : α → EAsyncIO ε β) : EAsyncIO ε β :=
  AsyncIO.bind self fun
    | .ok a => f a
    | .error e => AsyncIO.pure (.error e)

public def lift (x : EIO ε α) : EAsyncIO ε α :=
  AsyncIO.lift x.toBaseIO

public def block [Inhabited ε] (self : EAsyncIO ε α) : EIO ε α := do
  let result ← self |> AsyncIO.block
  match result with
  | .ok a => return a
  | .error e => throw e

public def concurrently (x : EAsyncIO ε α) (y : EAsyncIO ε β) : EAsyncIO ε (α × β) :=
  ((AsyncIO.concurrently x y).map ((fun (resultX, resultY) =>
    resultX >>= fun valueX =>
      resultY >>= fun valueY =>
        return (valueX, valueY)
    )
  ))

public def concurrentlyAll (xs : Array (EAsyncIO ε α)) :  EAsyncIO ε (Array α) :=
  xs.foldl (fun accumulator item =>
    (concurrently accumulator item).map (fun (arr, value) =>
      arr.push value
    )) (pure #[])

end EAsyncIO

public instance instFunctorEAsyncIO : Functor (EAsyncIO ε) where
  map := EAsyncIO.map

public instance instMonadEAsyncIO : Monad (EAsyncIO ε) where
  pure := EAsyncIO.pure
  bind := EAsyncIO.bind

public instance instMonadLiftBaseIOEAsyncIO : MonadLift BaseIO (EAsyncIO ε) where
  monadLift x := AsyncIO.lift (x.map pure)

public instance instMonadLiftEIOEAsyncIO : MonadLift (EIO ε) (EAsyncIO ε) where
  monadLift := EAsyncIO.lift

public instance instInhabitedEAsyncIO [Inhabited α] [Inhabited ε] : Inhabited (EAsyncIO ε α) where
  default := AsyncIO.pure default

@[extern "async_effect_ffi_asyncio_from_sleep"]
public opaque sleep (sleep : Sleep) : AsyncIO Unit

public def sleepFromSeconds (seconds: UInt64) : AsyncIO Unit :=
  sleep (Sleep.fromSeconds seconds)

public def sleepFromMilliseconds (milliseconds: UInt64) : AsyncIO Unit :=
  sleep (Sleep.fromMilliseconds milliseconds)

end IO

end AsyncEffect
