use std::{
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let a = Arc::new(Mutex::new(10));
    let b = Arc::new(Mutex::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    thread::spawn(move || loop {
        let a_val = a_clone.lock().unwrap();
        let b_val = b_clone.lock().unwrap();

        println!("t1 {a_val} {b_val}");
    });

    thread::spawn(move || loop {
        let b_val = b.lock().unwrap();
        let a_val = a.lock().unwrap();

        println!("t2 {a_val} {b_val}");
    })
    .join()
    .unwrap();
}
