# Sway-Bytecode-Root-Solver
Computes the bytecode root of a Contract or Predicate with configurables in Sway.

# Overview

## Simple Contracts and Predicates

There is a simple contract and a simple predicate which each have a configurable value. This configurable value can be changed in the SDK for a particular case or instance. 

## Swapper Contract

The configurable swapper contract takes some bytecode and configurable values, inserts the configurable values into the bytecode, and then returns the resulting bytecode.

## Verifier Script

The verifier script takes some bytecode and configurables values and computes the bytecode root. 

1. For a contract, it will compare the computed bytecode root and bytecode root of the contract deployed on chain.
2. For a predicate, it will prepend the seed to the bytecode root and verify that the address passed as an argument matches the computed address for the predicate. 

## Tests

All tests are run in the tests folder.
