# Tx3 Language Guide

This guide provides a comprehensive overview of the Tx3 language, its syntax, and features.

## Contents

1. [Syntax Overview](syntax.md) - Basic syntax and grammar
2. [Data Types](types.md) - Available data types and their usage
3. [Transaction Templates](templates.md) - How to define and use transaction templates
4. [Parties and Policies](parties-policies.md) - Working with parties and validation policies
5. [Assets and Values](assets.md) - Handling blockchain assets and values
6. [Expressions](expressions.md) - Expression syntax and evaluation
7. [Chain-Specific Features](chain-specific.md) - Features specific to different blockchains
8. [Visualization](visualization.md) - Understanding the three visualization levels

## Language Design Principles

Tx3 is designed with the following principles in mind:

1. **Domain-Specific**
   - Focused on UTxO transaction patterns
   - Built-in support for blockchain concepts
   - Clear transaction boundaries

2. **Safety First**
   - Type safety
   - Deterministic execution
   - No unbounded computation

3. **Declarative**
   - Clear transaction intent
   - Explicit interfaces
   - Self-documenting code

4. **Extensible**
   - Chain-specific features
   - Custom validation
   - Plugin architecture

## Quick Reference

### Basic Syntax
```tx3
// Party definition
party Name;

// Asset definition
asset Token = "policy_id" "asset_name";

// Transaction template
tx name(params) {
    input name { ... }
    output { ... }
}
```

### Common Patterns
```tx3
// Simple transfer
tx transfer(amount: Int) {
    input source { ... }
    output { ... }
}

// Time-locked transaction
tx lock(until: Int) {
    input source { ... }
    output { ... }
    policy TimeLock { ... }
}
```

## Next Steps

- [Syntax Overview](syntax.md) - Learn the basic syntax
- [Examples](../examples/index.md) - See real-world examples
- [Reference](../reference/index.md) - Complete language reference 