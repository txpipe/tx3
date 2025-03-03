# Transaction Templates

This guide covers how to define and use transaction templates in Tx3.

## Overview

Transaction templates are the core building blocks of Tx3 programs. They define patterns for constructing valid transactions by specifying:

- Required inputs
- Expected outputs
- Validation conditions
- Parameter bindings

## Basic Structure

```tx3
tx name(param1: Type1, param2: Type2) {
    // Input blocks
    input name { ... }
    
    // Output blocks
    output { ... }
    
    // Optional mint/burn blocks
    mint { ... }
    burn { ... }
    
    // Optional chain-specific blocks
    cardano { ... }
}
```

## Input Blocks

Input blocks define the UTxOs that must be consumed by the transaction:

```tx3
input name {
    from: Party,           // Required: who owns the UTxO
    min_amount: AssetExpr, // Optional: minimum value required
    datum_is: Type,        // Optional: required datum type
    redeemer: DataExpr,    // Optional: redeemer data
    ref: DataExpr,         // Optional: reference to specific UTxO
}
```

### Examples

```tx3
// Basic input
input source {
    from: Sender,
    min_amount: Ada(1000000),
}

// Input with datum
input locked {
    from: TimeLock,
    datum_is: State,
    redeemer: UnlockData { timestamp },
}

// Input with reference
input specific {
    ref: utxo_ref,
    from: Owner,
}
```

## Output Blocks

Output blocks define the UTxOs that will be created:

```tx3
output name? {
    to: DataExpr,      // Required: recipient address
    amount: AssetExpr, // Required: value to send
    datum: DataExpr,   // Optional: datum to attach
}
```

### Examples

```tx3
// Basic output
output {
    to: Receiver,
    amount: Ada(1000000),
}

// Named output with datum
output locked {
    to: TimeLock,
    amount: Ada(1000000),
    datum: State {
        lock_until: until,
        owner: Owner,
        beneficiary: Beneficiary,
    }
}
```

## Mint/Burn Blocks

Mint and burn blocks define token operations:

```tx3
mint {
    amount: AssetExpr,  // Required: amount to mint
    redeemer: DataExpr, // Required: minting policy data
}

burn {
    amount: AssetExpr,  // Required: amount to burn
    redeemer: DataExpr, // Required: burning policy data
}
```

### Examples

```tx3
// Minting tokens
mint {
    amount: MyToken(100),
    redeemer: MintData { quantity: 100 },
}

// Burning tokens
burn {
    amount: MyToken(50),
    redeemer: BurnData { quantity: 50 },
}
```

## Chain-Specific Blocks

Chain-specific blocks allow for blockchain-specific features:

```tx3
cardano {
    // Cardano-specific fields
    collateral: InputBlock,
    certificates: [Certificate],
    withdrawals: [(StakeCredential, Int)],
}
```

## Parameter Binding

Templates can take parameters that are bound at runtime:

```tx3
tx transfer(
    amount: Int,
    recipient: Bytes,
    message: String
) {
    input source {
        from: Sender,
        min_amount: Ada(amount),
    }
    
    output {
        to: recipient,
        amount: Ada(amount),
        datum: Message { text: message },
    }
}
```

## Common Patterns

### Simple Transfer
```tx3
tx transfer(amount: Int) {
    input source {
        from: Sender,
        min_amount: Ada(amount),
    }
    
    output {
        to: Receiver,
        amount: Ada(amount),
    }
    
    output {
        to: Sender,
        amount: source - Ada(amount) - fees,
    }
}
```

### Time-Locked Transaction
```tx3
tx lock(until: Int) {
    input source {
        from: Owner,
        min_amount: Ada(amount),
    }
    
    output locked {
        to: TimeLock,
        amount: Ada(amount),
        datum: State {
            lock_until: until,
            owner: Owner,
            beneficiary: Beneficiary,
        }
    }
    
    output {
        to: Owner,
        amount: source - Ada(amount) - fees,
    }
}
```

### Multi-Asset Transfer
```tx3
tx transfer_multi(
    ada_amount: Int,
    token_amount: Int
) {
    input source {
        from: Sender,
        min_amount: Ada(ada_amount) + MyToken(token_amount),
    }
    
    output {
        to: Receiver,
        amount: Ada(ada_amount) + MyToken(token_amount),
    }
    
    output {
        to: Sender,
        amount: source - (Ada(ada_amount) + MyToken(token_amount)) - fees,
    }
}
```

## Best Practices

1. **Input Selection**
   - Use clear input names
   - Specify minimum amounts
   - Validate datum types

2. **Output Creation**
   - Name outputs for reference
   - Include change outputs
   - Attach appropriate datums

3. **Parameter Usage**
   - Use descriptive names
   - Validate parameter ranges
   - Document requirements

4. **Error Prevention**
   - Check input sufficiency
   - Validate datum formats
   - Consider fee requirements

## Next Steps

- [Expressions](expressions.md) - Learn about expressions in templates
- [Parties and Policies](parties-policies.md) - Working with parties and validation
- [Chain-Specific Features](chain-specific.md) - Blockchain-specific features 