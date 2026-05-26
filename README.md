# StudyPad Savings

> A goal-based USDC savings vault for Southeast Asian university students, built on Stellar.

---

## Problem

A 19-year-old university student in Manila earns PHP 8,000/month from a part-time job but has no savings account — banks require a minimum deposit she can't maintain. Each semester, she fails to set aside enough for tuition because her funds sit in a GCash wallet with no friction against spending, costing her late fees of PHP 2,500 per missed deadline.

## Solution

StudyPad lets students lock USDC toward a specific academic goal (e.g. "Tuition Sem 2 2025") via a Soroban smart contract on Stellar. Funds are held on-chain and can only be withdrawn once the target is reached or the deadline ledger passes. Stellar's sub-cent transaction fees mean micro-deposits (as low as $0.10) are practical, and USDC eliminates local currency volatility. No bank account required — any wallet supporting Stellar (e.g. Lobstr, LOBSTR, Freighter) works.

---

## Stellar Features Used

| Feature | Why |
|---|---|
| **USDC on Stellar** | Stable store of value; no PHP volatility risk |
| **Soroban smart contracts** | Enforces lock-up logic on-chain; no trusted third party |
| **XLM** | Gas for all transactions (~$0.0001/tx) |
| **Trustlines** | Student wallet must trust USDC issuer to receive payouts |

---

## Target Users

- **Who:** University students aged 18–24 in the Philippines, Indonesia, and Vietnam
- **Income:** PHP 5,000–15,000/month from part-time work or family allowance
- **Behaviour:** Daily GCash / GoPay users; basic crypto literacy via school or Discord
- **Pain:** No savings product fits their income level or lacks enforcement against impulse spending

---

## Core Feature (MVP)

```
Student action         →  On-chain action                →  Result
─────────────────────────────────────────────────────────────────────
Create goal            →  write SavingGoal to storage    →  Goal recorded with target + deadline
Deposit 50 USDC        →  USDC transfer: wallet → contract→  balance += 50
Deposit 100 USDC       →  USDC transfer: wallet → contract→  balance = 150 (≥ target)
Withdraw               →  USDC transfer: contract → wallet→  150 USDC returned; goal.completed = true
```

Demo-able end-to-end in under 90 seconds on Stellar Testnet.

---

## Vision and Purpose

StudyPad is the first step toward a full academic financial layer on Stellar: savings → tuition disbursement → scholarship credentialing. Any institution, NGO, or scholarship body can top-up a student's vault as a conditional grant — the contract enforces the spend condition without any intermediary.

---

## Prerequisites

| Tool | Version |
|---|---|
| Rust | stable (≥ 1.74) |
| Soroban CLI | ≥ 21.0.0 |
| Node.js (for frontend) | ≥ 18 |

Install Soroban CLI:
```bash
cargo install --locked soroban-cli@21.0.0
```

---

## Build

```bash
soroban contract build
# Output: target/wasm32-unknown-unknown/release/studypad_savings.wasm
```

---

## Test

```bash
cargo test
# Runs all 5 tests in tests/test.rs
```

---

## Deploy to Testnet

```bash
# 1. Generate a test identity
soroban keys generate --global alice --network testnet

# 2. Fund it via Friendbot
soroban keys fund alice --network testnet

# 3. Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/studypad_savings.wasm \
  --source alice \
  --network testnet
# → prints CONTRACT_ID

# 4. Initialize (replace placeholders)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address alice) \
  --usdc_token GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5   # Testnet USDC
```

---

## Sample CLI Invocations

```bash
# Create a savings goal
soroban contract invoke \
  --id $CONTRACT_ID --source alice --network testnet \
  -- create_goal \
  --student $(soroban keys address alice) \
  --label "Tuition Sem 2 2025" \
  --target 150000000 \
  --deadline_ledger 999999

# Deposit 50 USDC (5_000_0000 stroops)
soroban contract invoke \
  --id $CONTRACT_ID --source alice --network testnet \
  -- deposit \
  --student $(soroban keys address alice) \
  --amount 50000000

# Check goal status
soroban contract invoke \
  --id $CONTRACT_ID --source alice --network testnet \
  -- get_goal \
  --student $(soroban keys address alice)

# Withdraw when goal is met
soroban contract invoke \
  --id $CONTRACT_ID --source alice --network testnet \
  -- withdraw \
  --student $(soroban keys address alice)
```

---

## Project Structure

```
studypad_savings/
├── Cargo.toml
├── README.md
├── src/
│   └── lib.rs          # Soroban contract
└── tests/
    └── test.rs         # 5 unit tests
```

---

## License

MIT © 2025 StudyPad Contributors

## Deployed Contract
