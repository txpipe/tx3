---
title: Quick Start Guide
---

This guide will help you write your first Tx3 program. We'll create a simple transfer program that allows one party to send ADA to another.

## Prerequisites

- Basic understanding of UTxO model (see [Basic Concepts](basic-concepts))
- Tx3 installed (see [Installation](installation))
- Development environment set up (see [Development Environment](development-environment))

## Your First Tx3 Program

Let's create a simple transfer program. Create a new file called `transfer.tx3`:

```tx3
// Define the parties involved in the transaction
party Sender;
party Receiver;

// Define the transaction template
tx transfer(
    quantity: Int
) {
    // Input: the UTxO to spend from
    input source {
        from: Sender,
        min_amount: Ada(quantity),
    }
    
    // Output: send ADA to the receiver
    output {
        to: Receiver,
        amount: Ada(quantity),
    }

    // Output: return change to the sender
    output {
        to: Sender,
        amount: source - Ada(quantity) - fees,
    }
}
```

## Understanding the Code

Let's break down what this program does:

1. **Party Definitions**
   ```tx3
   party Sender;
   party Receiver;
   ```
   - Defines two parties that will participate in the transaction
   - These are placeholder names that will be bound to actual addresses

2. **Transaction Template**
   ```tx3
   tx transfer(quantity: Int)
   ```
   - Defines a transaction template named `transfer`
   - Takes a parameter `quantity` of type `Int`
   - This parameter determines how much ADA to transfer

3. **Input Block**
   ```tx3
   input source {
       from: Sender,
       min_amount: Ada(quantity),
   }
   ```
   - Defines an input UTxO named `source`
   - Must be owned by the `Sender`
   - Must contain at least `quantity` ADA

4. **Output Blocks**
   ```tx3
   output {
       to: Receiver,
       amount: Ada(quantity),
   }
   ```
   - Creates a new UTxO owned by the `Receiver`
   - Contains exactly `quantity` ADA

   ```tx3
   output {
       to: Sender,
       amount: source - Ada(quantity) - fees,
   }
   ```
   - Creates a change UTxO owned by the `Sender`
   - Contains the remaining ADA after transfer and fees

## Using the Program

To use this program:

1. **Compile the Program**
   ```bash
   tx3c transfer.tx3
   ```

2. **Generate Interface**
   ```bash
   tx3c --gen-interface transfer.tx3
   ```

3. **Use in Your Application**
   ```rust
   use tx3_lang::Protocol;
   
   let mut protocol = Protocol::load_file("transfer.tx3")?;
   let tx = protocol.new_tx("transfer")?
       .with_arg("quantity", 1000000)?;
   ```

## Next Steps

- [Language Guide](../language-guide) - Learn more about Tx3 syntax and features
- [Examples](../examples) - See more complex examples
- [Reference](../reference) - Complete language reference 