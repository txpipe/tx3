# Visualization

This guide covers the three levels of visualization in Tx3, inspired by the C4 model approach.

## Overview

Tx3 provides three levels of visualization to help understand and document transaction patterns:

1. **Interaction Level (L1)**
   - High-level view
   - Business logic focus
   - Abstract technical details

2. **Transaction Level (L2)**
   - Detailed view
   - Transaction structure
   - Validation logic

3. **Validator Level (L3)**
   - Technical view
   - UTxO graph
   - Complete details

## Interaction Level (L1)

The interaction level focuses on the business logic and flow of transactions.

### Example
```tx3
// Simple transfer visualization
[User] -> [Transfer] -> [Recipient]
```

### Features
- Party interactions
- Transaction flow
- Business rules
- High-level state

### Use Cases
- System design
- Business documentation
- User guides
- Architecture reviews

## Transaction Level (L2)

The transaction level shows the detailed structure of transactions.

### Example
```tx3
// Transfer transaction visualization
Input: User UTxO
  ├─ Amount: 100 ADA
  └─ Datum: None

Outputs:
  ├─ Recipient UTxO
  │  ├─ Amount: 90 ADA
  │  └─ Datum: None
  └─ Change UTxO
     ├─ Amount: 10 ADA - fees
     └─ Datum: None

Validation:
  ├─ Signature: User
  └─ Time: Any
```

### Features
- Input/output structure
- Validation rules
- Data flow
- State transitions

### Use Cases
- Transaction design
- Debugging
- Testing
- Documentation

## Validator Level (L3)

The validator level shows the complete technical details of transactions.

### Example
```tx3
// Complete UTxO graph visualization
UTxO Set:
  ├─ Input UTxO
  │  ├─ Address: addr1...
  │  ├─ Value: 100 ADA
  │  ├─ Datum Hash: None
  │  └─ Script Hash: None
  │
  └─ Output UTxOs
     ├─ Recipient UTxO
     │  ├─ Address: addr2...
     │  ├─ Value: 90 ADA
     │  ├─ Datum Hash: None
     │  └─ Script Hash: None
     │
     └─ Change UTxO
        ├─ Address: addr1...
        ├─ Value: 10 ADA - fees
        ├─ Datum Hash: None
        └─ Script Hash: None

Script Context:
  ├─ Transaction Hash: tx1...
  ├─ Input Index: 0
  ├─ Redeemer: None
  └─ Datum: None
```

### Features
- Complete UTxO graph
- Script context
- Memory layout
- Binary format

### Use Cases
- Low-level debugging
- Script optimization
- Security analysis
- Protocol design

## Common Patterns

### Simple Transfer
```tx3
// L1: Interaction
[User] -> [Transfer] -> [Recipient]

// L2: Transaction
Input: User UTxO
Outputs: [Recipient UTxO, Change UTxO]

// L3: UTxO Graph
Input UTxO -> Output UTxOs
```

### Time-Locked Transaction
```tx3
// L1: Interaction
[User] -> [Lock] -> [TimeLock] -> [Beneficiary]

// L2: Transaction
Input: User UTxO
Output: Locked UTxO
  ├─ Amount: 100 ADA
  └─ Datum: Lock State

// L3: UTxO Graph
Input UTxO -> Locked UTxO
  ├─ Script Hash: TimeLock
  └─ Datum Hash: Lock State
```

### Multi-Asset Transaction
```tx3
// L1: Interaction
[User] -> [Transfer] -> [Recipient]
  ├─ ADA: 100
  └─ Token: 50

// L2: Transaction
Input: User UTxO
  ├─ ADA: 100
  └─ Token: 50
Output: Recipient UTxO
  ├─ ADA: 90
  └─ Token: 50

// L3: UTxO Graph
Input UTxO -> Output UTxOs
  ├─ Value: MultiAsset
  └─ Policy IDs: [Token]
```

## Best Practices

1. **Level Selection**
   - Choose appropriate level
   - Consider audience
   - Match use case

2. **Documentation**
   - Include all levels
   - Cross-reference
   - Update regularly

3. **Visualization**
   - Use clear diagrams
   - Consistent style
   - Highlight key points

4. **Maintenance**
   - Keep diagrams current
   - Version control
   - Review regularly

## Common Use Cases

### System Design
```tx3
// L1: System Overview
[Users] -> [DEX] -> [Liquidity Pools]
  ├─ Swap
  ├─ Add Liquidity
  └─ Remove Liquidity

// L2: Transaction Flow
Input: User UTxO
  ├─ Token A
  └─ ADA
Output: Pool UTxO
  ├─ Token A
  ├─ Token B
  └─ ADA

// L3: Technical Details
UTxO Graph
  ├─ Script Context
  ├─ Memory Layout
  └─ Binary Format
```

### Security Analysis
```tx3
// L1: Security Model
[Attacker] -> [Contract] -> [Victim]

// L2: Attack Vectors
Input: Malicious UTxO
  ├─ Invalid Datum
  └─ Invalid Redeemer
Output: Victim UTxO
  ├─ Stolen Assets
  └─ No Validation

// L3: Technical Vulnerabilities
Script Context
  ├─ Missing Checks
  ├─ Integer Overflow
  └─ State Corruption
```

## Next Steps

- [Examples](../examples/index.md) - Visualization examples
- [Best Practices](../best-practices/index.md) - Visualization guidelines
- [Reference](../reference/index.md) - Complete visualization reference 