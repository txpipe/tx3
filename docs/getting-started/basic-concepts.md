# Basic Concepts

This guide introduces the fundamental concepts you need to understand to work with Tx3.

## UTxO Model

UTxO (Unspent Transaction Output) is the accounting model used by blockchains like Cardano. In this model:

- The blockchain state is represented by a set of unspent transaction outputs
- Each UTxO has:
  - An address (who owns it)
  - A value (what assets it contains)
  - Optional data (datum)
- Transactions consume UTxOs and create new ones
- Each UTxO can only be spent once

## Transaction Templates

A transaction template is a pattern that describes how to construct a valid transaction. It includes:

- Input requirements (which UTxOs to consume)
- Output specifications (what new UTxOs to create)
- Validation conditions
- Parameter bindings

Example:
```tx3
tx transfer(quantity: Int) {
    input source {
        from: Sender,
        min_amount: Ada(quantity),
    }
    
    output {
        to: Receiver,
        amount: Ada(quantity),
    }
}
```

## Parties

Parties are the participants in a transaction. They can be:

- Wallet addresses
- Smart contracts
- Script addresses

Example:
```tx3
party Sender;
party Receiver;
```

## Assets

Assets represent the values that can be transferred in transactions:

- Native tokens (like ADA)
- Custom tokens
- NFTs

Example:
```tx3
asset MyToken = "policy_id" "asset_name";
```

## Policies

Policies are onchain validation scripts that enforce rules for transactions:

- Time locks
- Multi-signature requirements
- Custom validation logic

Example:
```tx3
policy TimeLock = import("validators/vesting.ak");
```

## Data Types

Tx3 supports various data types:

- Basic types: `Int`, `Bool`, `Bytes`, `String`
- Custom types through records and variants
- Asset types for blockchain assets

Example:
```tx3
record State {
    lock_until: Int,
    owner: Bytes,
    beneficiary: Bytes,
}
```

## Transaction Resolution

When a transaction template is used:

1. Parameters are bound to values
2. Input UTxOs are selected based on criteria
3. Output values are computed
4. The final transaction is constructed
5. The transaction is validated

## Visualization

Tx3 provides three levels of visualization:

1. **Interaction Level (L1)**
   - High-level view of interactions
   - Focus on business logic
   - Abstract away technical details

2. **Transaction Level (L2)**
   - Detailed view of transactions
   - Shows inputs, outputs, validation
   - Includes script requirements

3. **Validator Level (L3)**
   - Complete UTxO graph
   - Full technical details
   - Transaction structure

## Next Steps

- [Quick Start Guide](quick-start.md) - Write your first Tx3 program
- [Language Guide](../language-guide/index.md) - Learn the Tx3 language in detail
- [Examples](../examples/index.md) - See more complex examples 