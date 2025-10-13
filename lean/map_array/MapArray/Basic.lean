namespace MapArray

structure MapOptions where
  addend : Int32
  multiplicand : Int32
deriving Repr

instance : ToString MapOptions where
  toString x := s!"{Repr.reprPrec x 0}"

-- It seems to be illegal to put an export annotation on the constructor's name
-- inside the structure definition, so define a separate function.
@[export mk_map_options]
def mkMapOptions : (addend : Int32) → (multiplicand : Int32) → MapOptions := MapOptions.mk

@[export map_options_to_string]
def mapOptionsToString : MapOptions → String := ToString.toString

@[export my_map]
def map (options : MapOptions) (arr : Array UInt8) : Array Int32 :=
  arr.map (fun x => (x.toInt8.toInt32 + options.addend) * options.multiplicand)

end MapArray
