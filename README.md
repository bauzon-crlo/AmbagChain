# AmbagChain

A Soroban smart contract that enables college students to split shared expenses and settle payments transparently using Stellar transactions.

---

## Table of Contents

- [Overview](#overview)
- [Problem Statement](#problem-statement)
- [How It Works](#how-it-works)
- [Core MVP Feature](#defi-safety-features)
- [Contract API](#contract-api)
- [Data Structures](#data-structures)
- [Security Considerations](#security-considerations)
- [Timeline](#timeline)
- [Stellar Features used](#stellar-features-used)
- [Prerequisites](#prerequisites)
- [Build](#build)
- [Test](#test)
- [Deploy](#deploy)
- [Sample Invocations](#sample-invocations)
- [License](#license)

---

## Overview

`AmbagChain` is a decentralized expense-splitting system built on Soroban. It allows students to create shared bills (“ambagan”), track participants, and enforce fair payment settlement through on-chain verification.

Each expense is recorded as a bill, and participants individually settle their share via a smart contract, eliminating disputes and manual tracking.

---

## Problem Statement

### The Student Expense Problem

College students working on group projects frequently share expenses such as printing, materials, transportation, and food. Tracking who paid and who still owes money is often handled manually via chat or spreadsheets.

### What Goes Wrong Without It

**Price manipulation.** Some members forget or delay payment

**Stale data.** No reliable system to verify payments

**No cross-chain price portability.** Constant reminders create social friction

**Lack of composability.** Incorrect splits or forgotten contributions

### How `AmbagChain` Solves This

| Problem | Solution |
|---|---|
| No tracking system | On-chain bill records with participant list |
| Payment disputes | Immutable record of who paid |
| Manual reminders | Payment status visible to all |
| Unfair contribution | Equal share enforced by contract |

---

## How It Works

1. **Bill creation:** A student creates a bill with total amount and participants
2. **Share calculation:** The contract divides the total equally
3. **Payment:** Each participant calls `pay_share` to settle their portion
4. **Tracking:** Contract records payment status per user
5. **Completion:** Bill is marked complete once all participants pay

---

## Core MVP Feature

### `pay_share`

```rust
fn pay_share(e: Env, bill_id: u32, payer: Address)
```

## Single transaction that proves the system works end-to-end:
Validates that the payer is part of the bill
Ensures the user hasn’t already paid
Records payment status on-chain
Confirms settlement for that participant


### Contract API

### `create_bill`

```rust
fn create_bill(e: Env, creator: Address, total_amount: i128, participants: Vec<Address>) -> u32
```
Creates a new shared expense and returns a `bill_id`.
### `pay_share`

```rust
fn pay_share(e: Env, bill_id: u32, payer: Address)
```
Marks a participant as having paid their share.

### `get_bill`

```rust
fn get_bill(e: Env, bill_id: u32) -> Bill
```
Returns bill details including participants and payment status.

### `is_paid`

```rust
fn is_paid(e: Env, bill_id: u32, user: Address) -> bool
```
Checks if a participant has already paid.

---
### Data Structures

## `Bill`

| Field | Type | Description |
|---|---|---|
| `creator` | `Address` | Bill creator |
| `total_amount` | `i128` | Total amount of the bill |
| `participants` | `Vec<Address>` | List of members in the bill |
| `paid` | `Map<Address, bool>` | Payment status per user |


### Security Considerations

Duplicate payments prevented: A user cannot pay twice
Access control: Only listed participants can pay
Immutable tracking: Payment records cannot be altered
Simple logic = safer contract: Reduced attack surface

---

### Timeline

| Phase | Task | Duration |
|---|---|---|
| Phase 1 | Smart contract development | 2–3 days |
| Phase 2 | Testing (unit tests) | 1–2 days |
| Phase 3 | Deployment to testnet | 1 day |
| Phase 4 | Basic frontend (optional) | 2–3 days |

---

### Stellar Features Used

Soroban smart contract
XLM payments (conceptually tied to share settlement)

---


## Prerequisites
Rust (latest stable)
wasm32-unknown-unknown target
Stellar CLI
```bash
rustup target add wasm32-unknown-unknown
```
Soroban CLI


### Build

```bash
soroban contract build
```

### Test
```bash
cargo test
```

---

## Deployment

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/ambagchain.wasm \
  --source-account alice \
  --network testnet
```

### Sample Invocation

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source-account alice \
  --network testnet \
  -- pay_share \
  --bill_id 1 \
  --payer <USER_ADDRESS>
```

### License
MIT License

---

## Project Structure
```
.
├── src/
│   ├── lib.rs       # Core Soroban smart contract logic (bill creation, payment handling, storage)
│   └── test.rs      # Unit tests
├── Cargo.toml       # Project configuration, dependencies, and build settings
└── README.md        # Project documentation
```
