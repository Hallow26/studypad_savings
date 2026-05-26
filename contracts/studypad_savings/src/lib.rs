#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    token, Address, Bytes, Env,
    log,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Goal(Address),
    UsdcToken,
    Admin,
}

// ─── Data Structures ─────────────────────────────────────────────────────────

// NOTE: soroban-sdk 21.x does not support `String` inside #[contracttype] structs.
// We store the label as `Bytes` instead (UTF-8 encoded by the caller).
#[contracttype]
#[derive(Clone)]
pub struct SavingGoal {
    pub label: Bytes,
    pub target: i128,
    pub balance: i128,
    pub deadline_ledger: u32,
    pub completed: bool,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct StudyPadContract;

#[contractimpl]
impl StudyPadContract {

    /// Called once at deployment. Sets the USDC token address and admin.
    pub fn initialize(env: Env, admin: Address, usdc_token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        log!(&env, "StudyPad initialized");
    }

    /// Student creates a savings goal.
    /// label           — UTF-8 bytes, e.g. b"Tuition Sem 2 2025"
    /// target          — USDC stroops (1 USDC = 10_000_000)
    /// deadline_ledger — ledger sequence after which early withdrawal is allowed
    pub fn create_goal(
        env: Env,
        student: Address,
        label: Bytes,
        target: i128,
        deadline_ledger: u32,
    ) {
        student.require_auth();

        if target <= 0 {
            panic!("target must be positive");
        }

        if env.storage().persistent().has(&DataKey::Goal(student.clone())) {
            let existing: SavingGoal = env
                .storage()
                .persistent()
                .get(&DataKey::Goal(student.clone()))
                .unwrap();
            if !existing.completed {
                panic!("active goal already exists; withdraw first");
            }
        }

        let goal = SavingGoal {
            label,
            target,
            balance: 0,
            deadline_ledger,
            completed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Goal(student.clone()), &goal);

        log!(&env, "Goal created");
    }

    /// Student deposits USDC into their savings goal.
    pub fn deposit(env: Env, student: Address, amount: i128) {
        student.require_auth();

        if amount <= 0 {
            panic!("deposit amount must be positive");
        }

        let key = DataKey::Goal(student.clone());

        let mut goal: SavingGoal = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("no active goal found"));

        if goal.completed {
            panic!("goal already completed");
        }

        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .unwrap();

        let token_client = token::Client::new(&env, &usdc_addr);
        token_client.transfer(&student, &env.current_contract_address(), &amount);

        goal.balance += amount;

        env.storage().persistent().set(&key, &goal);

        log!(&env, "Deposited stroops");
    }

    /// Withdraw full balance once goal is met OR deadline has passed.
    pub fn withdraw(env: Env, student: Address) {
        student.require_auth();

        let key = DataKey::Goal(student.clone());

        let mut goal: SavingGoal = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("no goal found"));

        if goal.completed {
            panic!("already withdrawn");
        }

        let current_ledger = env.ledger().sequence();
        let goal_met = goal.balance >= goal.target;
        let deadline_passed = current_ledger > goal.deadline_ledger;

        if !goal_met && !deadline_passed {
            panic!("goal not yet met and deadline not passed; keep saving!");
        }

        let payout = goal.balance;
        goal.balance = 0;
        goal.completed = true;

        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .unwrap();

        let token_client = token::Client::new(&env, &usdc_addr);
        token_client.transfer(&env.current_contract_address(), &student, &payout);

        env.storage().persistent().set(&key, &goal);

        log!(&env, "Withdrew stroops");
    }

    /// Read-only: returns a student's current goal.
    pub fn get_goal(env: Env, student: Address) -> SavingGoal {
        env.storage()
            .persistent()
            .get(&DataKey::Goal(student))
            .unwrap_or_else(|| panic!("no goal found"))
    }

    /// Admin-only emergency fund recovery.
    pub fn admin_recover(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .unwrap();

        let token_client = token::Client::new(&env, &usdc_addr);
        token_client.transfer(&env.current_contract_address(), &to, &amount);

        log!(&env, "Admin recovery done");
    }
}
