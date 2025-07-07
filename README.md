![Timelock Wallet]

# Timelock Wallet (Arbitrum Stylus)

Rust implementation of an Ethereum time-lock wallet compiled to WASM using [Stylus](https://docs.arbitrum.io/stylus).
Funds deposited into the contract can only be withdrawn by the owner **after** a pre-defined unlock timestamp.
The owner can also extend (but never shorten) the lock period.

## Contract Overview

Key functions:

| Function | Description |
|----------|-------------|
| `init(uint256 unlockTimestamp)` | One-time initializer that sets the owner (`msg.sender`) and initial unlock time. |
| `deposit()` _(payable)_ | Accepts ETH deposits and emits a `Deposit` event. |
| `withdraw(address to)` | Sends the entire contract balance to `to` once the lock has expired. |
| `extend_lock(uint256 newUnlockTimestamp)` | Extends the lock period. `newUnlockTimestamp` must be **greater** than the current value. |
| `owner()` | Returns the owner address. |
| `unlock_time()` | Returns the current unlock timestamp. |

Events:

```solidity
event Deposit(address indexed from, uint256 amount);
event Withdrawal(address indexed to, uint256 amount);
```

Custom errors:

```solidity
error NotOwner();
error FundsLocked();
error ZeroBalance();
error AlreadyInitialized();
error NotInitialized();
```

The contract source lives in [`src/lib.rs`](./src/lib.rs).

## Quick Start

1. Install Rust and the Stylus tooling:

```bash
rustup target add wasm32-unknown-unknown
cargo install --force cargo-stylus cargo-stylus-check
```

2. Build the contract for WASM:

```bash
cargo stylus build
```

3. Validate the contract and run on-chain checks against the Stylus Sepolia testnet:

```bash
cargo stylus check
```

4. Deploy (replace the private key path with your own):

```bash
cargo stylus deploy \
  --private-key-path <PRIVKEY_FILE_PATH>
```

`cargo stylus deploy` automatically uploads the WASM, deploys the proxy, and activates the program.

## Interacting From Rust (ethers-rs)

```rust
abigen!(
    Timelock,
    r#"[
        function deposit() payable
        function withdraw(address to)
        function extend_lock(uint256 newUnlock)
        function owner() view returns (address)
        function unlock_time() view returns (uint256)
    ]"#
);

let timelock = Timelock::new(address, client);

// deposit 0.1 ETH
let value = ethers::utils::parse_ether(0.1)?;

timelock.deposit().value(value).send().await?.await?;

// try to withdraw (will revert if still locked)
timelock.withdraw(my_address).send().await?.await?;
```

## ABI Export

Generate a Solidity-compatible ABI:

```bash
cargo stylus export-abi
```

The ABI is written to `./target/abi/TimelockWallet.json`.

## Project Layout

```
src/
  ├── lib.rs   # Timelock wallet contract
  └── main.rs  # dummy main (required by Stylus)
examples/
  └── counter.rs   # sample off-chain interaction (kept for reference)
```

## Hands-On Labs

During the workshop this repository was used in an instructor-led lab.  
A PDF with the step-by-step exercises is included in the repo:

[Timelock Wallet – Hands-On Labs](./Timelock-wallet-lab.pdf)

## License

Apache-2.0 OR MIT – choose whichever you prefer.
