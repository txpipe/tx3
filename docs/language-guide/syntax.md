# Syntax Overview

This guide covers the basic syntax of the Tx3 language.

## Program Structure

A Tx3 program consists of a sequence of declarations and definitions:

```tx3
// Party declarations
party Sender;
party Receiver;

// Asset definitions
asset MyToken = "policy_id" "asset_name";

// Type definitions
record State {
    field1: Type1,
    field2: Type2,
}

// Transaction templates
tx transfer(amount: Int) {
    // ... transaction body
}
```

## Declarations

### Party Declarations
```tx3
party Name;
```
- Defines a participant in transactions
- Names must be unique
- Used in input/output blocks

### Asset Definitions
```tx3
asset Name = "policy_id" "asset_name";
```
- Defines a blockchain asset
- Requires policy ID and asset name
- Used in value expressions

### Type Definitions
```tx3
record Name {
    field1: Type1,
    field2: Type2,
}

variant Name {
    case1: Type1,
    case2: Type2,
}
```
- Defines custom data types
- Used for datums and redeemers
- Supports records and variants

## Transaction Templates

### Basic Structure
```tx3
tx name(param1: Type1, param2: Type2) {
    // ... transaction body
}
```
- Named transaction pattern
- Optional parameters
- Body contains input/output blocks

### Input Blocks
```tx3
input name {
    from: Party,
    min_amount: AssetExpr,
    datum_is: Type,
    redeemer: DataExpr,
    ref: DataExpr,
}
```
- Defines input UTxO requirements
- Named for reference in expressions
- Optional fields for validation

### Output Blocks
```tx3
output name? {
    to: DataExpr,
    amount: AssetExpr,
    datum: DataExpr,
}
```
- Defines output UTxO creation
- Optional name for reference
- Required fields for UTxO creation

### Mint/Burn Blocks
```tx3
mint {
    amount: AssetExpr,
    redeemer: DataExpr,
}

burn {
    amount: AssetExpr,
    redeemer: DataExpr,
}
```
- Defines token minting/burning
- Optional in transaction
- Requires amount and redeemer

## Expressions

### Data Expressions
```tx3
// Literals
123
true
"string"
0x"hex"

// Constructors
Type { field: value }

// Binary operations
expr1 + expr2
expr1 - expr2

// Property access
expr.field
```

### Asset Expressions
```tx3
// Asset constructor
Asset(amount)

// Binary operations
expr1 + expr2
expr1 - expr2

// Property access
expr.amount
```

## Comments

```tx3
// Single line comment

/* Multi-line
   comment */
```

## Whitespace and Formatting

- Whitespace is generally ignored
- Indentation is recommended for readability
- Semicolons are required after declarations
- Commas separate record/variant fields

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
}
```

### Time-Locked Transaction
```tx3
tx lock(until: Int) {
    input source {
        from: Owner,
        min_amount: Ada(amount),
    }
    
    output {
        to: TimeLock,
        amount: Ada(amount),
        datum: State { until },
    }
}
```

## Next Steps

- [Data Types](types.md) - Learn about available types
- [Transaction Templates](templates.md) - Deep dive into templates
- [Expressions](expressions.md) - Understanding expressions 