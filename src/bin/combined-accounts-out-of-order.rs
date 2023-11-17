//! A buggy accounting example demonstrating two accounts
//! (savings and checking) that deposit/withdraw/transfer asynchronously.
//!
//! This is a buggy implementation that uses the locks library. Because
//! the example violates the locking order, it does not compile. The
//! library prevents the possibility of deadlock at build time.
//!
//! See also
//! - combined-accounts
//! - combined-accounts-deadlock
//!
use locks::prelude::*;
use std::sync::Arc;

define_level!(CheckingLock);
define_level!(SavingsLock);
order_level!(CheckingLock < SavingsLock);

#[derive(Default)]
struct Account {
    balance: usize,
}

impl Account {
    fn balance(&self) -> usize {
        self.balance
    }

    fn deposit(&mut self, amount: usize) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: usize) {
        self.balance -= amount;
    }
}

struct CombinedAccount {
    savings: SavingsLock<Account>,
    checking: CheckingLock<Account>,
}

impl CombinedAccount {
    fn new() -> Self {
        Self {
            savings: SavingsLock::new(Account::default()),
            checking: CheckingLock::new(Account::default()),
        }
    }

    fn with_checking<BaseLock, F, T>(&self, hdl: &mut Handle<BaseLock>, cb: F) -> T
    where
        CheckingLock<Account>: LockLevelBelow<BaseLock>,
        F: FnOnce(&mut Account) -> T,
    {
        hdl.with(&self.checking, |_, data| cb(data))
    }

    fn with_savings<BaseLock, F, T>(&self, hdl: &mut Handle<BaseLock>, cb: F) -> T
    where
        SavingsLock<Account>: LockLevelBelow<BaseLock>,
        F: FnOnce(&mut Account) -> T,
    {
        hdl.with(&self.savings, |_, data| cb(data))
    }

    fn transfer<BaseLock>(&self, hdl: &mut Handle<BaseLock>, amount: usize)
    where
        CheckingLock<Account>: LockLevelBelow<BaseLock>,
    {
        hdl.with(&self.checking, |hdl, savings| {
            hdl.with(&self.savings, |_, checking| {
                savings.withdraw(amount);
                checking.deposit(amount);
            })
        })
    }

    fn credit_check<BaseLock>(&self, hdl: &mut Handle<BaseLock>) -> usize
    where
        SavingsLock<Account>: LockLevelBelow<BaseLock>,
    {
        hdl.with(&self.savings, |hdl, savings| {
            hdl.with(&self.checking, |_, checking| {
                savings.balance() + checking.balance()
            })
        })
    }
}

#[locks::main]
fn main() {
    let jeff = CombinedAccount::new();
    jeff.with_checking(main, |checking| {
        checking.deposit(1_500);
    });
    jeff.with_savings(main, |savings| {
        savings.deposit(20_000);
    });

    let jeff_arc = Arc::new(jeff);

    // Loop to try and get an unlucky schedule where we deadlock
    while jeff_arc.with_savings(main, |s| s.balance()) > 0 {
        // Thread 1: Runs a credit check on Jeff
        let jeff = Arc::clone(&jeff_arc);
        let credit_check_thread = spawn(&MainLevel, move |hdl| {
            let checking = jeff.with_checking(hdl, |act| act.balance());
            let savings = jeff.with_savings(hdl, |act| act.balance());
            let score = jeff.credit_check(hdl);
            println!("Jeff has ${checking} in his checking account, ${savings} in his savings, and credit score is {score}");
        });

        // Thread 2: Jeff is trying to transfer money from his savings to his checking
        let jeff = Arc::clone(&jeff_arc);
        let transfer_thread = spawn(&MainLevel, move |hdl| {
            jeff.transfer(hdl, 100);
        });

        credit_check_thread.join(main).unwrap();
        transfer_thread.join(main).unwrap();
    }
}
