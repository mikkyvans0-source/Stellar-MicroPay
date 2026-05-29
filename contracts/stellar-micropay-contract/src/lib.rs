#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token, Address, Env, Symbol,
};

const PERSISTENT_LIFETIME_THRESHOLD: u32 = 100_000;
const PERSISTENT_BUMP_AMOUNT: u32 = 500_000;

#[contracttype]
#[derive(Clone, Debug)]
pub struct TipRecord {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ReceiptMetadata {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub memo: Symbol,
    pub ledger: u32,
}

#[contracttype]
pub enum DataKey {
    Admin,
    TipTotal(Address),
    TipCount(Address),
    TipRecord(Address, u32),
    ReceiptCount(Address),
    ReceiptRecord(Address, u32),
}

#[contract]
pub struct MicroPayContract;

#[contractimpl]
impl MicroPayContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().extend_ttl(&DataKey::Admin, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).expect("Contract not initialized");
        if current_admin != stored_admin {
            panic!("Unauthorized");
        }
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().extend_ttl(&DataKey::Admin, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn send_tip(
        env: Env,
        token_address: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) {
        from.require_auth();
        if amount <= 0 {
            panic!("Tip amount must be positive");
        }
        let token = token::Client::new(&env, &token_address);
        token.transfer(&from, &to, &amount);

        let current_total: i128 = env.storage().persistent().get(&DataKey::TipTotal(to.clone())).unwrap_or(0);
        let current_count: u32 = env.storage().persistent().get(&DataKey::TipCount(to.clone())).unwrap_or(0);

        env.storage().persistent().set(&DataKey::TipTotal(to.clone()), &(current_total + amount));
        env.storage().persistent().extend_ttl(&DataKey::TipTotal(to.clone()), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.storage().persistent().set(&DataKey::TipCount(to.clone()), &(current_count + 1));
        env.storage().persistent().extend_ttl(&DataKey::TipCount(to.clone()), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        let record = TipRecord {
            from: from.clone(),
            to: to.clone(),
            amount,
            ledger: env.ledger().sequence(),
        };
        env.storage().persistent().set(&DataKey::TipRecord(to.clone(), current_count), &record);
        env.storage().persistent().extend_ttl(&DataKey::TipRecord(to.clone(), current_count), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.events().publish((Symbol::new(&env, "tip"), from, to.clone()), amount);
    }

    pub fn get_tip_total(env: Env, recipient: Address) -> i128 {
        let key = DataKey::TipTotal(recipient);
        let val = env.storage().persistent().get(&key).unwrap_or(0);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        }
        val
    }

    pub fn get_tip_count(env: Env, recipient: Address) -> u32 {
        let key = DataKey::TipCount(recipient);
        let val = env.storage().persistent().get(&key).unwrap_or(0);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        }
        val
    }

    pub fn get_admin(env: Env) -> Address {
        let key = DataKey::Admin;
        let val: Address = env.storage().persistent().get(&key).expect("Contract not initialized");
        env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        val
    }

    pub fn get_tip_record(env: Env, recipient: Address, index: u32) -> TipRecord {
        let key = DataKey::TipRecord(recipient, index);
        let val: TipRecord = env.storage().persistent().get(&key).expect("Tip record not found");
        env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        val
    }

    pub fn mint_receipt(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        memo: Symbol,
    ) -> u32 {
        from.require_auth();
        if amount <= 0 {
            panic!("Receipt amount must be positive");
        }
        let count: u32 = env.storage().persistent().get(&DataKey::ReceiptCount(from.clone())).unwrap_or(0);

        let receipt = ReceiptMetadata {
            from: from.clone(),
            to,
            amount,
            timestamp: env.ledger().timestamp(),
            memo,
            ledger: env.ledger().sequence(),
        };

        env.storage().persistent().set(&DataKey::ReceiptRecord(from.clone(), count), &receipt);
        env.storage().persistent().extend_ttl(&DataKey::ReceiptRecord(from.clone(), count), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.storage().persistent().set(&DataKey::ReceiptCount(from.clone()), &(count + 1));
        env.storage().persistent().extend_ttl(&DataKey::ReceiptCount(from.clone()), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

        env.events().publish((Symbol::new(&env, "receipt"), from), count);
        count
    }

    pub fn get_receipt_count(env: Env, payer: Address) -> u32 {
        let key = DataKey::ReceiptCount(payer);
        let val = env.storage().persistent().get(&key).unwrap_or(0);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        }
        val
    }

    pub fn get_receipt(env: Env, payer: Address, index: u32) -> ReceiptMetadata {
        let key = DataKey::ReceiptRecord(payer, index);
        let val: ReceiptMetadata = env.storage().persistent().get(&key).expect("Receipt not found");
        env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        val
    }

    pub fn create_escrow(
        _env: Env,
        _from: Address,
        _to: Address,
        _amount: i128,
        _release_ledger: u32,
    ) {
        panic!("Escrow payments coming in v2.1 — see ROADMAP.md");
    }

    pub fn batch_send(
        env: Env,
        token_address: Address,
        from: Address,
        recipients: soroban_sdk::Vec<Address>,
        amounts: soroban_sdk::Vec<i128>,
    ) {
        from.require_auth();
        if recipients.len() != amounts.len() {
            panic!("arrays must have equal length");
        }
        let token = token::Client::new(&env, &token_address);
        for i in 0..recipients.len() {
            let to = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            if amount <= 0 {
                panic!("amount must be positive");
            }
            token.transfer(&from, &to, &amount);
            
            let current_total: i128 = env.storage().persistent().get(&DataKey::TipTotal(to.clone())).unwrap_or(0);
            let current_count: u32 = env.storage().persistent().get(&DataKey::TipCount(to.clone())).unwrap_or(0);

            env.storage().persistent().set(&DataKey::TipTotal(to.clone()), &(current_total + amount));
            env.storage().persistent().extend_ttl(&DataKey::TipTotal(to.clone()), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

            env.storage().persistent().set(&DataKey::TipCount(to.clone()), &(current_count + 1));
            env.storage().persistent().extend_ttl(&DataKey::TipCount(to.clone()), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

            let record = TipRecord {
                from: from.clone(),
                to: to.clone(),
                amount,
                ledger: env.ledger().sequence(),
            };
            env.storage().persistent().set(&DataKey::TipRecord(to.clone(), current_count), &record);
            env.storage().persistent().extend_ttl(&DataKey::TipRecord(to.clone(), current_count), PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
        }
    }
}
