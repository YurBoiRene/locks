use std::{
    sync::{Arc, Mutex},
    thread,
};

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
    savings: Mutex<Account>,
    checking: Mutex<Account>,
}

impl CombinedAccount {
    fn new() -> Self {
        Self {
            savings: Mutex::new(Account::default()),
            checking: Mutex::new(Account::default()),
        }
    }

    fn transfer(&self, amount: usize) {
        let mut checking = self.checking.lock().unwrap();
        let mut savings = self.savings.lock().unwrap();
        savings.withdraw(amount);
        checking.deposit(amount);
    }

    fn credit_check(&self) -> usize {
        let savings = self.savings.lock().unwrap();
        let checking = self.checking.lock().unwrap();
        checking.balance() + savings.balance()
    }
}

fn main() {
    let jeff = CombinedAccount::new();

    jeff.checking.lock().unwrap().deposit(1_500);
    jeff.savings.lock().unwrap().deposit(20_000);

    let jeff_arc = Arc::new(jeff);

    // Loop to try and get an unlucky schedule where we deadlock
    while jeff_arc.savings.lock().unwrap().balance() > 0 {
        // Thread 1: Runs a credit check on Jeff
        let jeff = Arc::clone(&jeff_arc);
        let credit_check_thread = thread::spawn(move || {
            let checking = jeff.checking.lock().unwrap().balance();
            let savings = jeff.savings.lock().unwrap().balance();
            let score = jeff.credit_check();
            println!("Jeff has ${checking} in his checking account, ${savings} in his savings, and credit score is {score}");
        });

        // Thread 2: Jeff is trying to transfer money from his savings to his checking
        let jeff = Arc::clone(&jeff_arc);
        let transfer_thread = thread::spawn(move || {
            jeff.transfer(100);
        });

        credit_check_thread.join().unwrap();
        transfer_thread.join().unwrap();
    }
}
