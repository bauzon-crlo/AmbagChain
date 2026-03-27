// test.rs
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spin up a fresh environment with mocked auth.
fn setup() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

/// Register the contract and return a client + contract address.
fn deploy(e: &Env) -> (AmbagChainClient, Address) {
    let contract_id = e.register(AmbagChain, ());
    let client = AmbagChainClient::new(e, &contract_id);
    (client, contract_id)
}

/// Build a Vec<Address> of `n` random addresses.
fn make_participants(e: &Env, n: u32) -> Vec<Address> {
    let mut v = Vec::new(e);
    for _ in 0..n {
        v.push_back(Address::generate(e));
    }
    v
}

// ---------------------------------------------------------------------------
// create_bill
// ---------------------------------------------------------------------------

#[test]
fn test_create_bill_returns_incrementing_ids() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);

    let id0 = client.create_bill(&creator, &300i128, &participants).unwrap();
    let id1 = client.create_bill(&creator, &600i128, &participants).unwrap();

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

#[test]
fn test_create_bill_sets_correct_share() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 4);

    let bill_id = client.create_bill(&creator, &400i128, &participants).unwrap();
    let bill = client.get_bill(&bill_id).unwrap();

    assert_eq!(bill.total_amount, 400i128);
    assert_eq!(bill.share_per_person, 100i128); // 400 / 4
    assert_eq!(bill.participants.len(), 4);
    assert!(!bill.completed);
}

#[test]
fn test_create_bill_all_participants_start_unpaid() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);

    let bill_id = client.create_bill(&creator, &300i128, &participants).unwrap();

    for i in 0..participants.len() {
        let addr = participants.get(i).unwrap();
        assert!(!client.is_paid(&bill_id, &addr).unwrap());
    }
}

#[test]
fn test_create_bill_rejects_zero_amount() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 2);

    let result = client.try_create_bill(&creator, &0i128, &participants);
    assert!(result.is_err());
}

#[test]
fn test_create_bill_rejects_empty_participants() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let empty: Vec<Address> = Vec::new(&e);

    let result = client.try_create_bill(&creator, &100i128, &empty);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// pay_share
// ---------------------------------------------------------------------------

#[test]
fn test_pay_share_records_payment() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);
    let payer = participants.get(0).unwrap();

    let bill_id = client.create_bill(&creator, &300i128, &participants).unwrap();
    client.pay_share(&bill_id, &payer).unwrap();

    assert!(client.is_paid(&bill_id, &payer).unwrap());
}

#[test]
fn test_pay_share_does_not_complete_bill_until_all_paid() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);

    let bill_id = client.create_bill(&creator, &300i128, &participants).unwrap();

    // Pay first two
    client.pay_share(&bill_id, &participants.get(0).unwrap()).unwrap();
    client.pay_share(&bill_id, &participants.get(1).unwrap()).unwrap();

    assert!(!client.is_completed(&bill_id).unwrap());
}

#[test]
fn test_pay_share_completes_bill_when_all_paid() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);

    let bill_id = client.create_bill(&creator, &300i128, &participants).unwrap();

    for i in 0..participants.len() {
        client.pay_share(&bill_id, &participants.get(i).unwrap()).unwrap();
    }

    assert!(client.is_completed(&bill_id).unwrap());
}

#[test]
fn test_pay_share_rejects_double_payment() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 2);
    let payer = participants.get(0).unwrap();

    let bill_id = client.create_bill(&creator, &200i128, &participants).unwrap();
    client.pay_share(&bill_id, &payer).unwrap();

    // Second call must fail
    let result = client.try_pay_share(&bill_id, &payer);
    assert!(result.is_err());
}

#[test]
fn test_pay_share_rejects_non_participant() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 2);
    let outsider = Address::generate(&e);

    let bill_id = client.create_bill(&creator, &200i128, &participants).unwrap();

    let result = client.try_pay_share(&bill_id, &outsider);
    assert!(result.is_err());
}

#[test]
fn test_pay_share_rejects_nonexistent_bill() {
    let e = setup();
    let (client, _) = deploy(&e);
    let payer = Address::generate(&e);

    let result = client.try_pay_share(&99u32, &payer);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// get_bill / get_share / is_paid
// ---------------------------------------------------------------------------

#[test]
fn test_get_bill_returns_correct_data() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 5);

    let bill_id = client.create_bill(&creator, &500i128, &participants).unwrap();
    let bill = client.get_bill(&bill_id).unwrap();

    assert_eq!(bill.creator, creator);
    assert_eq!(bill.total_amount, 500i128);
    assert_eq!(bill.share_per_person, 100i128);
    assert_eq!(bill.participants.len(), 5);
}

#[test]
fn test_get_bill_nonexistent_returns_error() {
    let e = setup();
    let (client, _) = deploy(&e);

    let result = client.try_get_bill(&42u32);
    assert!(result.is_err());
}

#[test]
fn test_get_share_matches_expected() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 3);

    let bill_id = client.create_bill(&creator, &900i128, &participants).unwrap();
    assert_eq!(client.get_share(&bill_id).unwrap(), 300i128);
}

#[test]
fn test_is_paid_returns_false_for_unpaid_participant() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 2);
    let p0 = participants.get(0).unwrap();

    let bill_id = client.create_bill(&creator, &200i128, &participants).unwrap();

    assert!(!client.is_paid(&bill_id, &p0).unwrap());
}

#[test]
fn test_is_paid_returns_error_for_non_participant() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let participants = make_participants(&e, 2);
    let outsider = Address::generate(&e);

    let bill_id = client.create_bill(&creator, &200i128, &participants).unwrap();

    let result = client.try_is_paid(&bill_id, &outsider);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Single-participant edge case
// ---------------------------------------------------------------------------

#[test]
fn test_single_participant_bill() {
    let e = setup();
    let (client, _) = deploy(&e);
    let creator = Address::generate(&e);
    let mut participants: Vec<Address> = Vec::new(&e);
    let solo = Address::generate(&e);
    participants.push_back(solo.clone());

    let bill_id = client.create_bill(&creator, &100i128, &participants).unwrap();
    assert_eq!(client.get_share(&bill_id).unwrap(), 100i128);

    client.pay_share(&bill_id, &solo).unwrap();
    assert!(client.is_completed(&bill_id).unwrap());
}
