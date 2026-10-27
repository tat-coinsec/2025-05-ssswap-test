# S..S..Swap

Starts: May 29th, 2025 Noon UTC

Ends: June 05th, 2025 Noon UTC

nSLOC: 

[//]: # (contest-details-open)

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

[//]: # (contest-details-close)

[//]: # (scope-open)

## Scope

All Contracts within ```programs/amm/src/``` are in scope:

```
programs/amm/src/
├── instructions
    ├── liquidity_operations.rs
    ├── swap_operations.rs
    ├── transfer.rs
├── state
    ├── liquidity_pool.rs
├── lib.rs
```

## Compatibilities

  Blockchains:

      - Solana

  Tokens:

      - SPL and Token22, NO native SOL

[//]: # (scope-close)

[//]: # (getting-started-open)

## Setup

```
Build
anchor build
Test
```

[//]: # (getting-started-close)

[//]: # (known-issues-open)

## Known Issues
None.

[//]: # (known-issues-close)

## Running the tests

Run `anchor test` from the repository root.