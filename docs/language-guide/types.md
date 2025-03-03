# Data Types

This guide covers the data types available in Tx3 and how to use them.

## Basic Types

### Integer (`Int`)
```tx3
// Integer literals
123
-456
0
```
- Signed integer type
- Used for quantities, timestamps, etc.
- Supports basic arithmetic operations

### Boolean (`Bool`)
```tx3
// Boolean literals
true
false
```
- Logical values
- Used in conditions and flags
- Supports logical operations

### Bytes (`Bytes`)
```tx3
// Hex string literals
0x"deadbeef"
0x"1234"
```
- Raw byte data
- Used for addresses, hashes, etc.
- Represented as hex strings

### String (`String`)
```tx3
// String literals
"hello"
"world"
```
- Text data
- Used for names, messages, etc.
- UTF-8 encoded

## Asset Types

### Native Asset (`Ada`)
```tx3
// ADA literals
Ada(1000000)  // 1 ADA
Ada(500000)   // 0.5 ADA
```
- Native blockchain currency
- Fixed decimal places
- Basic unit operations

### Custom Assets
```tx3
// Asset definition
asset MyToken = "policy_id" "asset_name";

// Asset literals
MyToken(100)
```
- User-defined tokens
- Requires policy ID and name
- Supports basic arithmetic

## Custom Types

### Records
```tx3
// Record definition
record State {
    lock_until: Int,
    owner: Bytes,
    beneficiary: Bytes,
}

// Record construction
State {
    lock_until: 1234567890,
    owner: 0x"deadbeef",
    beneficiary: 0x"12345678",
}
```
- Named field collections
- Used for structured data
- Field access via dot notation

### Variants
```tx3
// Variant definition
variant Result {
    Success: Int,
    Error: String,
}

// Variant construction
Result::Success(42)
Result::Error("failed")
```
- Tagged unions
- Used for alternative values
- Pattern matching support

## Type Usage

### In Parameters
```tx3
tx transfer(
    amount: Int,
    recipient: Bytes,
    message: String
) {
    // ... transaction body
}
```

### In Data Expressions
```tx3
// Record construction
State {
    lock_until: 1234567890,
    owner: sender,
    beneficiary: receiver,
}

// Variant construction
Result::Success(42)
```

### In Asset Expressions
```tx3
// ADA amount
Ada(1000000)

// Custom token amount
MyToken(100)
```

## Type Safety

Tx3 provides several type safety features:

1. **Static Type Checking**
   - Types are checked at compile time
   - No runtime type errors
   - Clear error messages

2. **Type Inference**
   - Types can be inferred from context
   - Reduces type annotations
   - Maintains type safety

3. **Type Constraints**
   - Custom validation rules
   - Range checks
   - Format validation

## Common Patterns

### State Management
```tx3
record State {
    version: Int,
    data: Bytes,
    timestamp: Int,
}

tx update(new_data: Bytes) {
    input current {
        datum_is: State,
    }
    
    output {
        datum: State {
            version: current.version + 1,
            data: new_data,
            timestamp: current.timestamp,
        }
    }
}
```

### Error Handling
```tx3
variant Result {
    Success: Int,
    Error: String,
}

tx operation() {
    output {
        datum: Result::Success(42)
    }
}
```

## Next Steps

- [Transaction Templates](templates.md) - Learn how to use types in templates
- [Expressions](expressions.md) - Understanding type expressions
- [Chain-Specific Features](chain-specific.md) - Chain-specific type handling 