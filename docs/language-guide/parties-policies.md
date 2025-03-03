# Parties and Policies

This guide covers how to work with parties and validation policies in Tx3.

## Parties

Parties are the participants in transactions. They can represent:

- Wallet addresses
- Smart contracts
- Script addresses
- Other blockchain entities

### Party Declarations

```tx3
// Basic party declaration
party Name;

// Multiple parties
party Sender;
party Receiver;
party Escrow;
```

### Party Usage

Parties are used in input and output blocks:

```tx3
tx transfer(amount: Int) {
    input source {
        from: Sender,  // Party as input owner
        min_amount: Ada(amount),
    }
    
    output {
        to: Receiver,  // Party as output recipient
        amount: Ada(amount),
    }
}
```

## Policies

Policies are onchain validation scripts that enforce rules for transactions. They can implement:

- Time locks
- Multi-signature requirements
- Custom validation logic
- Other blockchain-specific rules

### Policy Definitions

```tx3
// Import policy from file
policy TimeLock = import("validators/vesting.ak");

// Define policy inline
policy MultiSig {
    required_signatures: 2,
    signers: [Party],
}
```

### Policy Usage

Policies are used in input blocks and can act as parties:

```tx3
tx lock(until: Int) {
    input source {
        from: Owner,
        min_amount: Ada(amount),
    }
    
    output locked {
        to: TimeLock,  // Policy as output recipient
        amount: Ada(amount),
        datum: State {
            lock_until: until,
            owner: Owner,
            beneficiary: Beneficiary,
        }
    }
}
```

## Common Patterns

### Time-Locked Transaction
```tx3
// Define parties
party Owner;
party Beneficiary;

// Define time lock policy
policy TimeLock = import("validators/vesting.ak");

// Transaction template
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
}
```

### Multi-Signature Transaction
```tx3
// Define parties
party Owner1;
party Owner2;
party Recipient;

// Define multi-sig policy
policy MultiSig {
    required_signatures: 2,
    signers: [Owner1, Owner2],
}

// Transaction template
tx transfer(amount: Int) {
    input source {
        from: MultiSig,
        min_amount: Ada(amount),
    }
    
    output {
        to: Recipient,
        amount: Ada(amount),
    }
}
```

### Escrow Transaction
```tx3
// Define parties
party Buyer;
party Seller;
party Escrow;

// Define escrow policy
policy EscrowLock = import("validators/escrow.ak");

// Transaction template
tx create_escrow(
    amount: Int,
    timeout: Int
) {
    input buyer_utxo {
        from: Buyer,
        min_amount: Ada(amount),
    }
    
    output escrow {
        to: EscrowLock,
        amount: Ada(amount),
        datum: EscrowState {
            buyer: Buyer,
            seller: Seller,
            amount: amount,
            timeout: timeout,
        }
    }
}
```

## Best Practices

1. **Party Naming**
   - Use descriptive names
   - Follow consistent conventions
   - Document role in comments

2. **Policy Design**
   - Keep policies focused
   - Document requirements
   - Consider security implications

3. **Validation Rules**
   - Define clear conditions
   - Handle edge cases
   - Consider failure modes

4. **Security Considerations**
   - Validate all inputs
   - Check authorization
   - Consider timeouts

## Common Use Cases

### Time-Based Operations
```tx3
policy TimeLock {
    min_time: Int,
    max_time: Int,
}

tx time_locked_transfer(
    amount: Int,
    unlock_time: Int
) {
    input source {
        from: Sender,
        min_amount: Ada(amount),
    }
    
    output locked {
        to: TimeLock,
        amount: Ada(amount),
        datum: TimeLockData {
            unlock_time: unlock_time,
            owner: Sender,
        }
    }
}
```

### Multi-Party Operations
```tx3
policy Threshold {
    threshold: Int,
    parties: [Party],
}

tx threshold_transfer(
    amount: Int,
    required_signatures: Int
) {
    input source {
        from: Threshold,
        min_amount: Ada(amount),
    }
    
    output {
        to: Recipient,
        amount: Ada(amount),
    }
}
```

## Next Steps

- [Chain-Specific Features](chain-specific.md) - Learn about blockchain-specific features
- [Expressions](expressions.md) - Understanding expressions with parties and policies
- [Best Practices](../best-practices/index.md) - Security and design guidelines 