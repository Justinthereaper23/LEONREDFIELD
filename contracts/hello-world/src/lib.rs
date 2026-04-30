// ScholarPay - Soroban Smart Contract
// Disburses USDC scholarship funds to verified student wallets on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, Symbol, token,
};

// ---------- Storage Keys ----------
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const TOKEN_KEY: Symbol = symbol_short!("TOKEN");

// ---------- Data Types ----------

/// Holds scholarship info per student
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Scholarship {
    pub student: Address,
    pub amount: i128,
    pub released: bool,
}

/// Storage map key: student Address → Scholarship
#[contracttype]
pub enum DataKey {
    Scholarship(Address),
}

// ---------- Contract ----------

#[contract]
pub struct ScholarPayContract;

#[contractimpl]
impl ScholarPayContract {
    /// Initialize the contract with an admin and USDC token address.
    /// Must be called once before any other function.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialized");
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&TOKEN_KEY, &token);
    }

    /// Admin registers a student and assigns a scholarship amount (in stroops / smallest unit).
    /// Only the admin can call this function.
    pub fn register_student(env: Env, student: Address, amount: i128) {
        // Verify caller is admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();

        // Amount must be positive
        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Save scholarship record — not yet released
        let scholarship = Scholarship {
            student: student.clone(),
            amount,
            released: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Scholarship(student), &scholarship);
    }

    /// Admin triggers USDC release to a student wallet.
    /// Transfers tokens from the contract's own balance to the student.
    pub fn release(env: Env, student: Address) {
        // Verify caller is admin
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();

        // Load scholarship record
        let key = DataKey::Scholarship(student.clone());
        let mut scholarship: Scholarship = env
            .storage()
            .persistent()
            .get(&key)
            .expect("student not registered");

        // Prevent double-release
        if scholarship.released {
            panic!("scholarship already released");
        }

        // Transfer USDC from this contract to the student
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &scholarship.student,
            &scholarship.amount,
        );

        // Mark as released so it cannot be released again
        scholarship.released = true;
        env.storage().persistent().set(&key, &scholarship);
    }

    /// Read-only: returns scholarship details for a given student.
    pub fn get_scholarship(env: Env, student: Address) -> Scholarship {
        env.storage()
            .persistent()
            .get(&DataKey::Scholarship(student))
            .expect("student not found")
    }

    /// Read-only: returns the configured admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }
}