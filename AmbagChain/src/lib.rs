#![no_std]
extern crate alloc;

use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, Address, Env, Map, Vec,
};

contractmeta!(
    key = "Description",
    val = "AmbagChain: Decentralized expense-splitting contract for Soroban. Create shared bills, track participants, and settle payments transparently."
);

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Error {
    /// Caller is not a participant of the bill.
    NotParticipant = 1,
    /// Participant has already paid their share.
    AlreadyPaid = 2,
    /// The requested bill does not exist.
    BillNotFound = 3,
    /// Participant list must not be empty.
    NoParticipants = 4,
    /// Total amount must be greater than zero.
    InvalidAmount = 5,
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A shared expense bill.
#[contracttype]
#[derive(Clone)]
pub struct Bill {
    /// Address that created the bill.
    pub creator: Address,
    /// Total amount to be split equally among participants (in stroops / smallest unit).
    pub total_amount: i128,
    /// Each participant's equal share = total_amount / participants.len()
    pub share_per_person: i128,
    /// All participants who owe a share.
    pub participants: Vec<Address>,
    /// Tracks which participants have settled (true = paid).
    pub paid: Map<Address, bool>,
    /// True once every participant has paid.
    pub completed: bool,
}

/// Storage keys
#[contracttype]
pub enum StorageKey {
    /// Maps bill_id -> Bill
    Bill(u32),
    /// Auto-incrementing counter for bill IDs
    BillCounter,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct AmbagChain;

#[contractimpl]
impl AmbagChain {
    // -----------------------------------------------------------------------
    // Write functions
    // -----------------------------------------------------------------------

    /// Create a new shared bill. Returns the new `bill_id`.
    ///
    /// * `creator`      – the address creating (and paying for) the bill record
    /// * `total_amount` – total cost in the smallest denomination (e.g. stroops)
    /// * `participants` – everyone who owes a share, including the creator if applicable
    pub fn create_bill(
        e: Env,
        creator: Address,
        total_amount: i128,
        participants: Vec<Address>,
    ) -> Result<u32, Error> {
        creator.require_auth();

        if total_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if participants.is_empty() {
            return Err(Error::NoParticipants);
        }

        let count = participants.len() as i128;
        let share_per_person = total_amount / count;

        // Build the paid map — everyone starts unpaid
        let mut paid: Map<Address, bool> = Map::new(&e);
        for i in 0..participants.len() {
            paid.set(participants.get(i).unwrap(), false);
        }

        // Mint a new bill ID
        let bill_id: u32 = e
            .storage()
            .instance()
            .get(&StorageKey::BillCounter)
            .unwrap_or(0u32);

        let bill = Bill {
            creator: creator.clone(),
            total_amount,
            share_per_person,
            participants,
            paid,
            completed: false,
        };

        e.storage()
            .instance()
            .set(&StorageKey::Bill(bill_id), &bill);
        e.storage()
            .instance()
            .set(&StorageKey::BillCounter, &(bill_id + 1));

        Ok(bill_id)
    }

    /// Mark a participant as having paid their share of a bill.
    ///
    /// Errors if:
    /// - The bill does not exist (`BillNotFound`)
    /// - The payer is not a participant (`NotParticipant`)
    /// - The payer has already paid (`AlreadyPaid`)
    pub fn pay_share(e: Env, bill_id: u32, payer: Address) -> Result<(), Error> {
        payer.require_auth();

        let mut bill: Bill = e
            .storage()
            .instance()
            .get(&StorageKey::Bill(bill_id))
            .ok_or(Error::BillNotFound)?;

        // Validate payer is a participant
        if !bill.paid.contains_key(payer.clone()) {
            return Err(Error::NotParticipant);
        }

        // Prevent double-payment
        if bill.paid.get(payer.clone()).unwrap_or(false) {
            return Err(Error::AlreadyPaid);
        }

        // Record payment
        bill.paid.set(payer, true);

        // Check if all participants have now paid
        let all_paid = (0..bill.participants.len()).all(|i| {
            let addr = bill.participants.get(i).unwrap();
            bill.paid.get(addr).unwrap_or(false)
        });
        bill.completed = all_paid;

        e.storage()
            .instance()
            .set(&StorageKey::Bill(bill_id), &bill);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read functions
    // -----------------------------------------------------------------------

    /// Return the full bill record for a given `bill_id`.
    pub fn get_bill(e: Env, bill_id: u32) -> Result<Bill, Error> {
        e.storage()
            .instance()
            .get(&StorageKey::Bill(bill_id))
            .ok_or(Error::BillNotFound)
    }

    /// Check whether a specific participant has paid their share.
    pub fn is_paid(e: Env, bill_id: u32, user: Address) -> Result<bool, Error> {
        let bill: Bill = e
            .storage()
            .instance()
            .get(&StorageKey::Bill(bill_id))
            .ok_or(Error::BillNotFound)?;

        if !bill.paid.contains_key(user.clone()) {
            return Err(Error::NotParticipant);
        }

        Ok(bill.paid.get(user).unwrap_or(false))
    }

    /// Returns true if every participant has settled their share.
    pub fn is_completed(e: Env, bill_id: u32) -> Result<bool, Error> {
        let bill: Bill = e
            .storage()
            .instance()
            .get(&StorageKey::Bill(bill_id))
            .ok_or(Error::BillNotFound)?;

        Ok(bill.completed)
    }

    /// Returns the share amount each participant owes for a given bill.
    pub fn get_share(e: Env, bill_id: u32) -> Result<i128, Error> {
        let bill: Bill = e
            .storage()
            .instance()
            .get(&StorageKey::Bill(bill_id))
            .ok_or(Error::BillNotFound)?;

        Ok(bill.share_per_person)
    }
}
