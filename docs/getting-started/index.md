---
title: Getting Started with Tx3
sidebar:
   label: Intro
   order: 1
---

This section will help you get started with Tx3, from installation to writing your first transaction template.

## Contents

1. [Installation](./getting-started/installation) - How to install and set up Tx3
2. [Basic Concepts](./getting-started/basic-concepts) - Understanding UTxO, transactions, and templates
3. [Quick Start Guide](./getting-started/quick-start) - Your first Tx3 program
4. [Development Environment](./getting-started/development-environment) - Setting up your development environment

## What is Tx3?

Tx3 is a domain-specific language designed specifically for describing transaction templates in UTxO-based blockchains. It helps you:

- Define clear interfaces for your dapps
- Describe transaction patterns
- Generate concrete transactions
- Visualize transaction flows

## Why Tx3?

In UTxO-based blockchains like Bitcoin & Cardano, dapps are defined by transaction patterns rather than explicit function interfaces. This makes it challenging for:

- Dapp authors to convey their dapp's interface
- Users to understand how to interact with dapps
- Developers to build tools that work with dapps

Tx3 solves these challenges by providing a language specifically designed for describing transaction patterns and generating interfaces.

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
   - Mainly chain-agnostic
   - Chain-specific features


## Next Steps

1. [Install Tx3](./getting-started/installation)
2. [Learn the basic concepts](./getting-started/basic-concepts)
3. [Write your first program](./getting-started/quick-start)
4. [Explore examples](./examples) 