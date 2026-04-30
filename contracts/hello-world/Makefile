# SariEscrow 🛒

> **Trustless on-chain escrow for buyers and sellers across Southeast Asia — powered by Stellar & Soroban.**

---

## Problem

A sari-sari store owner in Cebu, Philippines wants to bulk-order goods from a supplier in Makati she has never met. Sending payment upfront risks losing her ₱8,000 to a scam; the supplier refuses to ship without payment. The deadlock kills the deal and costs both parties a week of income.

## Solution

SariEscrow lets the buyer lock USDC into a Soroban smart contract with one tap. The supplier ships knowing funds are secured on Stellar. Once the buyer confirms receipt, the contract instantly releases USDC to the seller — no bank, no middleman, no 3–5 day clearing time. Disputes are resolved by a designated admin who can refund or release with a single on-chain call.

---

## Stellar Features Used

| Feature | Why |
|---|---|
| **USDC (Stellar asset)** | Stable settlement currency; trustline required |
| **Soroban smart contracts** | Enforce escrow logic, dispute resolution on-chain |
| **XLM** | Transaction fees |
| **Trustlines** | Buyer and seller opt in to USDC before transacting |

---

## Timeline

| Phase | Duration | Deliverable |
|---|---|---|
| Smart contract + tests | Day 1–2 | `lib.rs` passing all 5 tests |
| Stellar testnet deploy | Day 2 | Contract address on Futurenet |
| React/Next.js frontend | Day 3–4 | Buyer & seller dashboards |
| Demo polish + pitch | Day 5 | 2-min demo video |

---

## Vision & Purpose

SariEscrow targets the 70 million unbanked and under-banked micro-merchants across SEA who rely on informal trust networks for trade. By replacing that trust with programmable escrow on Stellar, SariEscrow removes the single biggest barrier to SME e-commerce in the region: payment risk.

Long-term: integrate local anchors (GCash, Maya, OVO) so buyers can fund USDC escrow directly from e-wallets with zero crypto knowledge required.

---

## Prerequisites

- **Rust** ≥ 1.74 (`rustup install stable`)
- **Soroban CLI** ≥ 20.x (`cargo install --locked soroban-cli`)
- **Stellar testnet account** with USDC trustline

---

## Build

```bash
soroban contract build
# Output: target/wasm32-unknown-unknown/release/sari_escrow.wasm
```

---

## Test

```bash
cargo test
# Runs all 5 unit tests in src/test.rs
```

---

## Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/sari_escrow.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

Save the returned `CONTRACT_ID` for the CLI invocations below.

---

## Sample CLI Invocations

### 1 — Create an order (buyer locks 50 USDC)
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_SECRET> \
  --network testnet \
  -- create_order \
  --buyer  GBUYER... \
  --seller GSELLER... \
  --token  GUSDC... \
  --amount 50000000 \
  --admin  GADMIN...
```
> Returns: `order_id` (e.g. `1`)

### 2 — Seller marks order as shipped
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <SELLER_SECRET> \
  --network testnet \
  -- mark_shipped \
  --caller   GSELLER... \
  --order_id 1
```

### 3 — Buyer confirms receipt (releases USDC to seller)
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_SECRET> \
  --network testnet \
  -- confirm_receipt \
  --caller   GBUYER... \
  --order_id 1
```

### 4 — Raise a dispute
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_SECRET> \
  --network testnet \
  -- raise_dispute \
  --caller   GBUYER... \
  --order_id 1
```

### 5 — Admin resolves dispute (refund buyer)
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- resolve_dispute \
  --caller      GADMIN... \
  --order_id    1 \
  --refund_buyer true
```

---

## License

MIT © 2025 SariEscrow Contributors