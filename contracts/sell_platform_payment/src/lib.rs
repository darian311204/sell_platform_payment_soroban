#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::{self, TokenClient},
    Address, Env, String, Symbol,
};

#[contract]
pub struct GamePaymentContract;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Merchant(Address),
    Payment(u64),
    NextPaymentId,
}

#[derive(Clone)]
#[contracttype]
pub struct Payment {
    pub id: u64,
    pub buyer: Address,
    pub merchant: Address,
    pub amount: i128,
    pub token: Address,
    pub game_id: String,
    pub completed: bool,
    pub refunded: bool,
}

#[contractimpl]
impl GamePaymentContract {

    // ==========================================
    // INIT
    // ==========================================
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);

        env.storage()
            .instance()
            .set(&DataKey::NextPaymentId, &1u64);
    }

    // ==========================================
    // ADMIN
    // ==========================================
    pub fn add_merchant(
        env: Env,
        admin: Address,
        merchant: Address,
    ) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        if stored_admin != admin {
            panic!("not admin");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Merchant(merchant.clone()), &true);
    }

    pub fn remove_merchant(
        env: Env,
        admin: Address,
        merchant: Address,
    ) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        if stored_admin != admin {
            panic!("not admin");
        }

        env.storage()
            .persistent()
            .remove(&DataKey::Merchant(merchant));
    }

    // ==========================================
    // CREATE PAYMENT
    // ==========================================
    pub fn create_payment(
        env: Env,
        buyer: Address,
        merchant: Address,
        token: Address,
        amount: i128,
        game_id: String,
    ) -> u64 {

        buyer.require_auth();

        let exists: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Merchant(merchant.clone()))
            .unwrap_or(false);

        if !exists {
            panic!("merchant not approved");
        }

        let mut next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPaymentId)
            .unwrap();

        let token_client = TokenClient::new(&env, &token);

        // buyer -> contract
        token_client.transfer(
            &buyer,
            &env.current_contract_address(),
            &amount,
        );

        let payment = Payment {
            id: next_id,
            buyer,
            merchant,
            amount,
            token,
            game_id,
            completed: false,
            refunded: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Payment(next_id), &payment);

        next_id += 1;

        env.storage()
            .instance()
            .set(&DataKey::NextPaymentId, &next_id);

        next_id - 1
    }

    // ==========================================
    // RELEASE ESCROW
    // ==========================================
    pub fn complete_payment(
        env: Env,
        merchant: Address,
        payment_id: u64,
    ) {

        merchant.require_auth();

        let mut payment: Payment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .unwrap();

        if payment.completed {
            panic!("already completed");
        }

        if payment.refunded {
            panic!("already refunded");
        }

        if payment.merchant != merchant {
            panic!("not merchant");
        }

        let token_client =
            TokenClient::new(&env, &payment.token);

        token_client.transfer(
            &env.current_contract_address(),
            &merchant,
            &payment.amount,
        );

        payment.completed = true;

        env.storage()
            .persistent()
            .set(&DataKey::Payment(payment_id), &payment);
    }

    // ==========================================
    // REFUND
    // ==========================================
    pub fn refund_payment(
        env: Env,
        admin: Address,
        payment_id: u64,
    ) {

        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        if stored_admin != admin {
            panic!("not admin");
        }

        let mut payment: Payment = env
            .storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .unwrap();

        if payment.completed {
            panic!("already released");
        }

        if payment.refunded {
            panic!("already refunded");
        }

        let token_client =
            TokenClient::new(&env, &payment.token);

        token_client.transfer(
            &env.current_contract_address(),
            &payment.buyer,
            &payment.amount,
        );

        payment.refunded = true;

        env.storage()
            .persistent()
            .set(&DataKey::Payment(payment_id), &payment);
    }

    // ==========================================
    // GET PAYMENT
    // ==========================================
    pub fn get_payment(
        env: Env,
        payment_id: u64,
    ) -> Payment {

        env.storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .unwrap()
    }
}