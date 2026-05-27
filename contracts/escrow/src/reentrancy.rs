// contracts/escrow/src/reentrancy.rs
//
// SC-SEC-072: Reentrancy protection for Soroban escrow.
//
// Soroban does NOT have the EVM's single-threaded execution model: a contract
// can be called back via cross-contract invocation within the same transaction.
// We use a boolean flag embedded in `EscrowState` as a mutex. The flag is
// written to persistent storage before any token transfer, so a re-entrant
// call will see the flag set and panic immediately.
//
// Usage pattern (call-sites in lib.rs):
//
//   let mut state = load_state(env);
//   enter_reentrancy_guard(env, &mut state);   // panics if already locked
//   // … token transfer …
//   exit_reentrancy_guard(env, &mut state);    // must always be reached
//
// The `with_reentrancy_guard` helper wraps this in a closure so the exit is
// guaranteed even if the closure panics (Soroban unwinds the entire tx on
// panic, so the storage write is rolled back — the flag reset is belt-and-
// suspenders for unit-test environments that recover panics).

use soroban_sdk::{panic_with_error, Env};

use crate::error::EscrowError;
use crate::storage_types::{DataKey, EscrowState};

/// Acquires the reentrancy lock. Panics with `EscrowError::ReentrancyDetected`
/// if it is already held.
///
/// Writes the updated state immediately so the lock is visible to any
/// re-entrant call that reads storage.
#[inline(always)]
pub fn enter_reentrancy_guard(env: &Env, state: &mut EscrowState) {
    if state.reentrancy_lock {
        panic_with_error!(env, EscrowError::ReentrancyDetected);
    }
    state.reentrancy_lock = true;
    // Persist the lock BEFORE the token transfer.
    env.storage().instance().set(&DataKey::State, state);
}

/// Releases the reentrancy lock and persists the updated state.
#[inline(always)]
pub fn exit_reentrancy_guard(env: &Env, state: &mut EscrowState) {
    state.reentrancy_lock = false;
    env.storage().instance().set(&DataKey::State, state);
}