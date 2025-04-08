---
title: Basic Concepts
---

This guide introduces the fundamental concepts you need to understand to work with Tx3.

## UTxO Model

UTxO (Unspent Transaction Output) is the accounting model used by blockchains like Bitcoin & Cardano. In this model:

- The blockchain state is represented by a set of unspent transaction outputs
- Each UTxO has:
  - An address (who owns it)
  - A value (what assets it contains)
  - Optional data (datum)
- Transactions consume UTxOs and create new ones
- Each UTxO can only be spent once

## Tx3 Templates

A transaction template in Tx3 is a pattern that describes how to construct a valid transaction. It includes:

- Input requirements (which UTxOs to consume)
- Output specifications (what new UTxOs to create)
- Validation conditions
- Parameter bindings

Example:
```tx3
party Sender;
party Receiver;

tx transfer(quantity: Int) {
    input source {
        from: Sender,
        min_amount: Ada(quantity),
    }
    
    output {
        to: Receiver,
        amount: Ada(quantity) - fees,
    }
}
```

## Party Definition

Parties are the participants in a transaction. They can be:

- Wallet addresses
- Script addresses

Example:
```tx3
party Sender;
party Receiver;
```

## Policy Definition

Policies are onchain validation scripts that enforce rules for transactions:

- Multi-signature requirements
- Custom validation logic

Example:
```tx3
policy TimeLock = 0x01233456789ABCDEF;
```

## Asset Definition

Assets represent the values that can be transferred in transactions:

- Native tokens (like BTC or ADA)
- Custom tokens
- NFTs

By defining an asset in your protocol, it enable its use as an [asset expression](#asset-expressions).

Example:
```tx3
asset MyToken = 0x01233456789ABCDEF.MYTOKEN";
```

## Asset Expressions

Asset expressions are used to compute and manipulate asset values in transactions. They support:

- Basic arithmetic operations (addition, subtraction)
- Asset constructors (chain native tokens & custom tokens)
- Binary operations between assets (add, subtract, multiply, divide)

Example:
```tx3
// Basic asset expressions
Ada(1)  // 1 ADA
Lovelace(1000000) // 1 ADA
MyToken(50)   // 50 tokens of the MyToken asset definition

// Arithmetic operations
source - Ada(amount) - fees
Ada(1000000) + MyToken(50)
```


## Data Expressions

Data expressions are used to construct and manipulate data values in transactions. They support:

- Literals (integers, booleans, strings, bytes)
- Record construction
- Variant construction
- Binary operations
- Property access

Example:
```tx3
// Basic literals
123
true
"hello"
0x"deadbeef"

// Record construction
State {
    lock_until: 1234567890,
    owner: 0xDEADBEEF,
    beneficiary: 0x12345678,
}

// Variant construction
Result::Success(42)
Result::Error("failed")

// Binary operations
a + b
a - b

// Property access
record.field
variant.field
```

Data expressions are commonly used for:
- Constructing datums
- Creating redeemers
- Computing values
- Validating conditions

## Built-in Data Types

Tx3 supports various data types which are built-in into the langage:

- Basic types: `Int`, `Bool`, `Bytes`, `String`
- Custom types through records and variants

## Custom Type Definitions

Example:
```tx3
type State {
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
