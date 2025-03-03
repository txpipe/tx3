# Chain-Specific Features

This guide covers blockchain-specific features in Tx3, with a focus on Cardano.

## Overview

Tx3 provides support for blockchain-specific features through:

- Chain-specific blocks
- Native asset handling
- Protocol parameters
- Network-specific features

## Cardano Features

### Protocol Parameters
```tx3
// Access protocol parameters
pparams.min_fee_coefficient
pparams.min_fee_constant
pparams.coins_per_utxo_byte
```

### Native Scripts
```tx3
// Define native script
policy TimeLock = import("validators/vesting.ak");

// Use native script
tx lock(until: Int) {
    input source {
        from: TimeLock,
        min_amount: Ada(amount),
    }
}
```

### Certificates
```tx3
cardano {
    certificates: [
        StakeRegistration { ... },
        StakeDelegation { ... },
        StakeDeregistration { ... },
    ]
}
```

### Withdrawals
```tx3
cardano {
    withdrawals: [
        (StakeCredential, Int),  // (stake credential, amount)
    ]
}
```

### Collateral
```tx3
cardano {
    collateral: input {
        from: User,
        min_amount: Ada(collateral_amount),
    }
}
```

## Common Patterns

### Stake Registration
```tx3
tx register_stake(
    stake_credential: StakeCredential
) {
    input source {
        from: User,
        min_amount: Ada(registration_fee),
    }
    
    cardano {
        certificates: [
            StakeRegistration {
                credential: stake_credential,
            }
        ]
    }
}
```

### Stake Delegation
```tx3
tx delegate_stake(
    stake_credential: StakeCredential,
    pool_id: PoolId
) {
    input source {
        from: User,
        min_amount: Ada(delegation_fee),
    }
    
    cardano {
        certificates: [
            StakeDelegation {
                credential: stake_credential,
                pool_id: pool_id,
            }
        ]
    }
}
```

### Reward Withdrawal
```tx3
tx withdraw_rewards(
    stake_credential: StakeCredential,
    amount: Int
) {
    input source {
        from: User,
        min_amount: Ada(withdrawal_fee),
    }
    
    cardano {
        withdrawals: [
            (stake_credential, amount)
        ]
    }
}
```

## Best Practices

1. **Protocol Parameters**
   - Check parameter values
   - Handle parameter changes
   - Consider hard forks

2. **Native Scripts**
   - Validate script logic
   - Test script behavior
   - Document requirements

3. **Certificates**
   - Verify certificate data
   - Check certificate order
   - Handle failures

4. **Withdrawals**
   - Validate amounts
   - Check credentials
   - Consider fees

## Common Use Cases

### Stake Pool Registration
```tx3
tx register_pool(
    pool_params: PoolParams
) {
    input source {
        from: Operator,
        min_amount: Ada(registration_fee),
    }
    
    cardano {
        certificates: [
            PoolRegistration {
                params: pool_params,
            }
        ]
    }
}
```

### Stake Pool Retirement
```tx3
tx retire_pool(
    pool_id: PoolId,
    epoch: Int
) {
    input source {
        from: Operator,
        min_amount: Ada(retirement_fee),
    }
    
    cardano {
        certificates: [
            PoolRetirement {
                pool_id: pool_id,
                epoch: epoch,
            }
        ]
    }
}
```

### Multi-Certificate Transaction
```tx3
tx multi_cert(
    stake_cred: StakeCredential,
    pool_id: PoolId
) {
    input source {
        from: User,
        min_amount: Ada(total_fee),
    }
    
    cardano {
        certificates: [
            StakeRegistration {
                credential: stake_cred,
            },
            StakeDelegation {
                credential: stake_cred,
                pool_id: pool_id,
            }
        ]
    }
}
```

## Network-Specific Features

### Testnet Support
```tx3
// Network selection
network = "testnet"

// Testnet-specific parameters
pparams.testnet = true
```

### Mainnet Support
```tx3
// Network selection
network = "mainnet"

// Mainnet-specific parameters
pparams.mainnet = true
```

## Next Steps

- [Best Practices](../best-practices/index.md) - Chain-specific guidelines
- [Reference](../reference/index.md) - Complete chain-specific reference
- [Examples](../examples/index.md) - Chain-specific examples 