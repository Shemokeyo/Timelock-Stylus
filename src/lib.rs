// This attribute disables the main entry point unless the "export-abi" feature is enabled.
// It's used for Stylus contracts to control how the contract is compiled and exported.
#![cfg_attr(not(feature = "export-abi"), no_main)]

// Import the alloc crate for heap-allocated types like Vec.
extern crate alloc;

// Bring Vec into scope from the alloc crate.
use alloc::vec::Vec;

// Import Ethereum primitive types: Address (20 bytes) and U256 (256-bit unsigned integer).
use alloy_primitives::{Address, U256};
// Import macros and types for Solidity compatibility and error handling.
use alloy_sol_types::{sol, SolError};
// Import Stylus SDK modules for interacting with the EVM and blockchain environment.
use stylus_sdk::{block, call::transfer_eth, evm, prelude::*};

// Use the sol! macro to define Solidity-style events and errors for the contract.
sol! {
    // Event emitted when someone deposits ETH into the contract.
    event Deposit(address indexed from, uint256 amount);
    // Event emitted when ETH is withdrawn from the contract.
    event Withdrawal(address indexed to, uint256 amount);

    // Custom Solidity-style errors for better error handling.
    error NotOwner();         // Thrown if a non-owner tries to call owner-only functions.
    error FundsLocked();      // Thrown if funds are still locked and withdrawal is attempted.
    error ZeroBalance();      // Thrown if withdrawal is attempted with zero balance.
    error AlreadyInitialized(); // Thrown if init is called more than once.
    error NotInitialized();   // Thrown if contract is used before initialization.
}

// Use the sol_storage! macro to define the contract's persistent storage layout.
sol_storage! {
    #[entrypoint] // Marks this struct as the main contract entry point.
    pub struct TimelockWallet {
        address owner;              // The owner of the wallet (can withdraw/extend lock).
        uint256 unlock_timestamp;   // The timestamp after which funds can be withdrawn.
    }
}

// Implementation block for public functions of the TimelockWallet contract.
#[public]
impl TimelockWallet {
    /// One-time initialiser. Must be called after deployment.
    /// Sets the owner and unlock timestamp. Prevents re-initialization.
    pub fn init(&mut self, unlock_timestamp: U256) -> Result<(), Vec<u8>> {
        // Prevent re-initialization: owner must be unset (Address::ZERO).
        if self.owner.get() != Address::ZERO {
            return Err(AlreadyInitialized {}.abi_encode());
        }
        // Set the owner to the caller of this function.
        self.owner.set(self.vm().msg_sender());
        // Set the unlock timestamp to the provided value.
        self.unlock_timestamp.set(unlock_timestamp);
        Ok(())
    }

    /// Payable function to deposit ETH into the contract.
    /// Emits a Deposit event. Only works after initialization.
    #[payable]
    pub fn deposit(&self) -> Result<(), Vec<u8>> {
        // Ensure the contract is initialized.
        if self.owner.get() == Address::ZERO {
            return Err(NotInitialized {}.abi_encode());
        }
        // Emit a Deposit event with sender and amount.
        evm::log(Deposit {
            from: self.vm().msg_sender(),
            amount: self.vm().msg_value(),
        });
        Ok(())
    }

    /// Withdraw all ETH to a specified address if the lock has expired.
    /// Only the owner can call this, and only after unlock time.
    pub fn withdraw(&mut self, to: Address) -> Result<(), Vec<u8>> {
        // Ensure the contract is initialized.
        if self.owner.get() == Address::ZERO {
            return Err(NotInitialized {}.abi_encode());
        }
        // Only the owner can withdraw.
        if self.vm().msg_sender() != self.owner.get() {
            return Err(NotOwner {}.abi_encode());
        }
        // Get the current block timestamp.
        let now = U256::from(block::timestamp());
        // Check if the lock period has expired.
        if now < self.unlock_timestamp.get() {
            return Err(FundsLocked {}.abi_encode());
        }
        // Get the contract's ETH balance.
        let balance = self.vm().balance(self.vm().contract_address());
        // Prevent withdrawal if balance is zero.
        if balance.is_zero() {
            return Err(ZeroBalance {}.abi_encode());
        }
        // Transfer all ETH to the specified address.
        transfer_eth(to, balance)?;
        // Emit a Withdrawal event.
        evm::log(Withdrawal { to, amount: balance });
        Ok(())
    }

    /// Extend the lock period to a new unlock timestamp.
    /// Only the owner can call this, and only to increase the lock time.
    pub fn extend_lock(&mut self, new_unlock: U256) -> Result<(), Vec<u8>> {
        // Ensure the contract is initialized.
        if self.owner.get() == Address::ZERO {
            return Err(NotInitialized {}.abi_encode());
        }
        // Only the owner can extend the lock.
        if self.vm().msg_sender() != self.owner.get() {
            return Err(NotOwner {}.abi_encode());
        }
        // New unlock time must be strictly greater than the current one.
        if new_unlock <= self.unlock_timestamp.get() {
            return Err(FundsLocked {}.abi_encode());
        }
        // Set the new unlock timestamp.
        self.unlock_timestamp.set(new_unlock);
        Ok(())
    }

    /// Returns the address of the wallet owner.
    pub fn owner(&self) -> Address {
        self.owner.get()
    }

    /// Returns the unlock timestamp.
    pub fn unlock_time(&self) -> U256 {
        self.unlock_timestamp.get()
    }
}
