# Tx3

⚠️ This is an early experimental project, everything here is subject to change or even disappear.

A domain-specific language for describing and visualizing transactions in UTxO-based blockchains, with a particular focus on Cardano.

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

### Example Swap

```tx3
datum PoolState {
   pair_a: Token,
   pair_b: Token,
}

datum SwapParams {
   amount: Int,
   ratio: Int,
}

party Buyer;

party Dex {
   address: addr1xxx,
}

tx swap(
   buyer: Buyer,
   ask: Token,
   bid: Token
) {
   input pool {
         from: dex,
         datum_is: PoolState,

         redeemer: SwapParams {
            ask: ask,
            bid: ask,
         }
   }
   
   input* payment {
         from: buyer,
         min_amount: fees + bid,
   }
   
   output {
         to: pool
         datum: PoolState {
            pair_a: inputs.pool.pair_a - ask,
            pair_b: inputs.pool.pair_b + bid,
            ...inputs.pool.datum
         }
   }

   output {
         to: buyer,
         amount: inputs.payment.amount + ask - bid - fees,
   }
}
```

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
