# Tx3

A domain-specific language for describing protocols that run on UTxO-based blockchains, with a particular focus on Cardano.

## Rationale

In account-based blockchains, dapps are primarily defined by smart contracts with explicit function interfaces that represent user interactions and state mutations. These dapps have a clear API surface that can be used by different parties to interact with its business logic.

The deterministic nature of the UTxO approach is a great property, but it has a drawback: the interface of a dapp is not explicit, there's nothing describing how the different parties can interact with it. Dapps are defined by transaction patterns that represent deterministic "state transitions". A party interacting with an UTxO-based dapp has to understand the underlying business logic in order to construct transactions representing their intents.

This is why we need a strict but flexible mechanism to describe patterns of transactions (which we'll call transaction templates) that dapp authors can use to convey the interface of their dapp and dapp consumers can use to interact with it.

## Scope

- a language to describe a dapp as a set of transaction templates
- a set of diagram conventions to visualize a dapp at different levels of abstraction
- an interpreter that takes a transaction template and a set of parameters and generates a concrete transaction
- a tool to generate executable onchain / offchain code from transaction templates

## The Language

Tx3 is a purely functional language designed specifically for describing transaction templates in UTxO-based blockchains. Rather than trying to be a general-purpose language, it focuses exclusively on this narrow but important domain, allowing it to provide powerful and precise tools for working with blockchain transactions.

The language's narrow focus allows it to be highly opinionated and tightly integrated with core UTxO blockchain concepts. Instead of abstracting away blockchain-specific details, Tx3 embraces them - concepts fundamental to UTxO blockchains like inputs, outputs, datums, and redeemers are treated as first-class citizens in the language. This deep integration enables more natural and expressive ways to work with these concepts.

A key concept in Tx3 is the distinction between concrete transactions and transaction templates. While a transaction represents a specific state transition, a template is a function - it defines a pattern that can generate many different concrete transactions based on parameters provided at runtime. This parameterization is what makes templates so powerful for defining reusable transaction patterns.

### Simple Transfer Example

Let's start with a very simple example: a transfer of funds from one party to another.

```tx3
party Sender;

party Receiver;

tx transfer(
    quantity: Int
) {
    input source {
        from: Sender,
        min_amount: Ada(quantity),
    }
    
    output {
        to: Receiver,
        amount: Ada(quantity),
    }

    output {
        to: Sender,
        amount: source - Ada(quantity) - fees,
    }
}
```

Things to notice in the example:

- The `tx` keyword is used to define a transaction template. You can think of it as a function that takes some parameters and returns a transaction.
- The `party` keyword is used to define a party. Parties are used to identify the participants in a transaction.
- Transactions are mainly defined by their inputs and outputs. Inputs describe criteria for selecting the UTxOs from the party's UTxO set. Outputs describe the new UTxOs that will be created by the transaction.
- Outputs are defined in terms of value (and data) expressions computed from the inputs.

### Vesting Example

Here's a more complex example: a vesting contract that allows a beneficiary to claim funds after a certain amount of time has passed.

```tx3
party Owner;

party Beneficiary;

policy TimeLock = import(validators/vesting.ak);

record State {
  lock_until: Int,
  owner: Bytes,
  beneficiary: Bytes,
}

tx lock(
    quantity: Int,
    until: Int
) {
    input source {
        from: Owner,
        min_amount: quantity,
    }
    
    output target {
        to: TimeLock,
        amount: Ada(quantity),
        datum: State {
            lock_until: until,
            owner: Owner,
            beneficiary: Beneficiary,
        }
    }

    output {
        to: Owner,
        amount: source - Ada(quantity) - fees,
    }
}

tx unlock(
    locked_utxo: UtxoRef
) {
    input gas {
        from: Beneficiary,
        min_amount: fees,
    }

    input locked {
        ref: locked_utxo,
        redeemer: (),
    }

    output target {
        to: Beneficiary,
        amount: gas + locked - fees,
    }
}
```

Things to notice in the example:

- The `policy` keyword is used to define a policy, an onchain validation script of some sort. Policies can also act as parties in the transaction (by inferring a script address).
- The `record` keyword is used to define a record. Records are used to define the data structure that can be used as datums or redeemers.
- You can have more than one transaction template in a tx3 protocol. In this case, the `lock` and `unlock` templates are part of the `vesting` protocol.
- Datums and redeemers can be constructed by specifying the fields of the record. Data can be computed from parameters or from the inputs.

## The Diagrams

Tx3 provides three levels of transaction visualization (hence the name), inspired by the C4 model approach:

1. **Interaction Level (L1)**
   - High-level view of interactions between different parties/contracts
   - Focus on business logic and flow
   - Abstract away technical UTxO details

2. **Transaction Level (L2)**
   - Detailed view of individual transactions
   - Shows transaction inputs, outputs, and validation logic
   - Includes script requirements and constraints

3. **Validator Level (L3)**
   - Lowest level of detail
   - Complete UTxO graph representation
   - Full technical details of the transaction structure
