# Assets and Values

This guide covers how to work with blockchain assets and values in Tx3.

## Overview

Tx3 provides built-in support for working with blockchain assets:

- Native currency (ADA)
- Custom tokens
- NFTs
- Multi-asset bundles

## Asset Definitions

### Native Asset (ADA)
```tx3
// ADA is built-in
Ada(1000000)  // 1 ADA
Ada(500000)   // 0.5 ADA
```

### Custom Assets
```tx3
// Define custom asset
asset MyToken = "policy_id" "asset_name";

// Use custom asset
MyToken(100)
```

### Asset Bundles
```tx3
// Combine multiple assets
Ada(1000000) + MyToken(50)

// Complex bundles
Ada(1000000) + MyToken(50) + NFT("policy_id", "asset_name")
```

## Asset Expressions

### Basic Operations
```tx3
// Addition
asset1 + asset2

// Subtraction
asset1 - asset2

// Property access
asset.amount
```

### Examples

```tx3
// Simple transfer
tx transfer(amount: Int) {
    input source {
        from: Sender,
        min_amount: Ada(amount),
    }
    
    output {
        to: Receiver,
        amount: Ada(amount),
    }
}

// Multi-asset transfer
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
}
```

## Asset Validation

### Minimum Amounts
```tx3
input source {
    from: Sender,
    min_amount: Ada(1000000),  // At least 1 ADA
}
```

### Exact Amounts
```tx3
output {
    to: Receiver,
    amount: Ada(1000000),  // Exactly 1 ADA
}
```

### Asset Combinations
```tx3
input source {
    from: Sender,
    min_amount: Ada(1000000) + MyToken(50),  // Multiple assets
}
```

## Common Patterns

### Token Swap
```tx3
tx swap(
    input_amount: Int,
    output_amount: Int
) {
    input source {
        from: User,
        min_amount: TokenA(input_amount),
    }
    
    output {
        to: User,
        amount: TokenB(output_amount),
    }
}
```

### NFT Transfer
```tx3
tx transfer_nft(
    policy_id: Bytes,
    asset_name: Bytes
) {
    input source {
        from: Owner,
        min_amount: NFT(policy_id, asset_name),
    }
    
    output {
        to: Recipient,
        amount: NFT(policy_id, asset_name),
    }
}
```

### Liquidity Pool
```tx3
tx add_liquidity(
    ada_amount: Int,
    token_amount: Int
) {
    input ada {
        from: User,
        min_amount: Ada(ada_amount),
    }
    
    input token {
        from: User,
        min_amount: MyToken(token_amount),
    }
    
    output {
        to: Pool,
        amount: Ada(ada_amount) + MyToken(token_amount),
    }
}
```

## Best Practices

1. **Asset Naming**
   - Use descriptive names
   - Follow consistent conventions
   - Document asset properties

2. **Amount Validation**
   - Check minimum amounts
   - Validate asset combinations
   - Consider decimals

3. **Fee Handling**
   - Account for transaction fees
   - Include fee buffer
   - Handle rounding

4. **Asset Bundling**
   - Group related assets
   - Optimize bundle size
   - Consider UTxO limits

## Common Use Cases

### Token Minting
```tx3
tx mint_tokens(
    amount: Int
) {
    input source {
        from: MintingAuthority,
        min_amount: Ada(fees),
    }
    
    mint {
        amount: MyToken(amount),
        redeemer: MintData { quantity: amount },
    }
}
```

### Token Burning
```tx3
tx burn_tokens(
    amount: Int
) {
    input source {
        from: User,
        min_amount: MyToken(amount),
    }
    
    burn {
        amount: MyToken(amount),
        redeemer: BurnData { quantity: amount },
    }
}
```

### Asset Locking
```tx3
tx lock_assets(
    amount: Int
) {
    input source {
        from: User,
        min_amount: Ada(amount) + MyToken(amount),
    }
    
    output locked {
        to: LockContract,
        amount: Ada(amount) + MyToken(amount),
        datum: LockState {
            amount: amount,
            owner: User,
        }
    }
}
```

## Next Steps

- [Expressions](expressions.md) - Learn about asset expressions
- [Chain-Specific Features](chain-specific.md) - Blockchain-specific asset handling
- [Best Practices](../best-practices/index.md) - Asset management guidelines 