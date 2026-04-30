# ScholarPay 🎓

> Instant USDC scholarship disbursement for students — powered by Stellar & Soroban.

---

## Problem

A university student in the Philippines waits 2–4 weeks for scholarship funds processed through slow bank transfers, causing missed tuition deadlines and unnecessary late fees.

## Solution

ScholarPay lets universities deploy a Soroban smart contract that automatically releases USDC directly to verified student wallets on Stellar — no bank middleman, instant settlement, full on-chain transparency.

---

## Timeline

| Phase | Description | Duration |
|-------|-------------|----------|
| Week 1 | Contract development & local testing | 7 days |
| Week 2 | Frontend MVP + testnet deployment | 7 days |
| Week 3 | Demo prep & student wallet UX | 3 days |

---

## Stellar Features Used

- ✅ **USDC transfers** — stable disbursements in USD Coin on Stellar
- ✅ **Soroban smart contracts** — trustless, auditable release logic
- ✅ **Trustlines** — students must establish a USDC trustline before receiving funds

---

## Vision & Purpose

ScholarPay aims to eliminate the bureaucratic delay in scholarship disbursements across Southeast Asia. By anchoring funds on Stellar, universities gain full auditability and students receive money in seconds — not weeks. In the future, ScholarPay can integrate local anchors (e.g., GCash, Maya) to allow instant PHP conversion directly from the student's wallet.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 1.74+)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup) v20+
- Stellar Testnet account with XLM (use [Friendbot](https://friendbot.stellar.org))

```bash
# Install Soroban CLI
cargo install --locked soroban-cli --version 20.0.0

# Add Wasm target
rustup target add wasm32-unknown-unknown
```

---

## Build

```bash
soroban contract build
# Output: target/wasm32-unknown-unknown/release/scholar_pay.wasm
```

---

## Test

```bash
cargo test
```

Expected: 5 tests pass ✅

---

## Deploy to Testnet

```bash
# 1. Configure testnet identity
soroban keys generate --global alice --network testnet

# 2. Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/scholar_pay.wasm \
  --source alice \
  --network testnet

# Output: CONTRACT_ID (save this)
```

---

## Sample CLI Invocations

### Initialize the contract
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin GADMIN...STELLARADDRESS \
  --token GAUSDC...TOKENADDRESS
```

### Register a student
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  register_student \
  --student GSTUDENT...ADDRESS \
  --amount 5000000   # 0.5 USDC (7 decimal places)
```

### Release scholarship funds
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  release \
  --student GSTUDENT...ADDRESS
```

### Check scholarship status
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- \
  get_scholarship \
  --student GSTUDENT...ADDRESS
```

---

## Project Structure

```
scholarpay/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      # Soroban smart contract
    └── test.rs     # 5 unit tests
```

---

## License

MIT © 2025 ScholarPay Contributors