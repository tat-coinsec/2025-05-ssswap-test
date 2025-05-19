# S..S..Swap
## Prize Pool TO BE FILLED OUT BY CYFRIN
Total Pool -

H/M -

Low -

Starts:

Ends:

nSLOC: 

Complexity Score: 

## About the Project

The protocol is aimed to be a minimalistic AMM on Solana. A template, if you want to call it that way.
Users can fork upon the project and integrate additional own logic, that's why I kept it simple.
It provides the absolute barebones of AMM functionality:

- Pool Creation
- Liquidity Operations
- Swap Exact In and Exact Out
- It has Slippage Protection
- It has LP Fee Collection

You will realize that commonly used functions like "preview swap" and "get price" are missing, which is intended for the current state.
I might decide to add those at a later point.

The audit is supposed to focus on the code validation of what is written and invariants I believe are important are documented within the natspec of the functions in lib.rs.

## Actors

The Protocol is permissionless and has no admin functionality, therefor no trusted roles are integrated.
Other than that we hopefully have Liquidity Providers and Swappers, called Users.

## Compatibilities

  Blockchains:
      - Solana
  Tokens:
      - SPL and Token22, NO native SOL

## Setup
This is a standard Anchor project, using LiteSVM for testing

Build
anchor build

Test


## Known Issues
None.