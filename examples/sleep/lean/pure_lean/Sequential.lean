import Sleeper

open Sleeper (blockAndPrint formatElapsedTime sleepAndPrint)

def secondsDurationRange := [0:6]

def main : IO Unit := do
  IO.println "Sequential sleep operations"
  let action := secondsDurationRange.forM Sleeper.sleepAndPrint
  blockAndPrint action
