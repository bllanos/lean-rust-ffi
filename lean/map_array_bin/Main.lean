import MapArray

def main : IO Unit := do
  IO.println "Program start"

  let addend : Int32 := 2
  let multiplicand : Int32 := 3
  let mapOptions : MapArray.MapOptions := {addend := addend, multiplicand := multiplicand }
  IO.println s!"MapOptions instance: {mapOptions}"

  let arrSize := 6
  let mut arr : Array UInt8 := Array.emptyWithCapacity arrSize
  for i in [:arrSize] do
    assert! (i * 5) = (i.toUInt8 * 5).toNat
    arr := arr.push (i.toUInt8 * 5)
  IO.println s!"Input array: {arr}"

  let arrOut := MapArray.map mapOptions arr
  IO.println s!"Output array: {arrOut}"

  IO.println "Program end"
