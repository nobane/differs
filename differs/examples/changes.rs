use differs::{
    changed, diff_changes, Changed,
    Changed::{Added, Removed},
    Diff,
};

#[derive(Diff, Clone, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
    residents: Vec<String>,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct User {
    age: u32,
    address: Address,
}

fn main() {
    let old = User {
        age: 30,
        address: Address {
            street: "1 Main".into(),
            city: "Paris".into(),
            residents: vec!["Alice".into(), "Bob".into()],
        },
    };

    let mut new = old.clone();
    new.age = 31; // scalar
    new.address.city = "Berlin".into(); // nested scalar
    new.address.residents.retain(|n| n != "Bob"); // remove one
    new.address.residents.push("Carol".into()); // add one

    let changes = diff_changes(&old, &new);

    for ch in &changes {
        #[allow(clippy::deprecated_cfg_attr)]
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match ch {
            UserChange::age(age) => println!("age -> {age}"),
            UserChange::address(AddressChange::self_(address)) => println!("address -> {address:?}"),
            UserChange::address(AddressChange::city(val)) => println!("city -> {val}"),
            UserChange::address(AddressChange::residents(Changed::Added(name))) => println!("resident + -> {name}"),
            UserChange::address(AddressChange::residents(Changed::Removed(name))) => println!("resident - -> {name}"),
            _ => {}
        }
    }

    for ch in &changes {
        changed!(ch;
            User.age(age)                      => { println!("age -> {age}"); };
            User.address@(snapshot)            => { println!("address -> {snapshot:?}"); };
            User.address.city(v)               => { println!("city -> {v}"); };
            User.address.residents(Added(v))   => { println!("+res -> {v}"); };
            User.address.residents(Removed(v)) => { println!("-res -> {v}"); };
        );
    }

    println!("\nCHANGES: {changes:?}");
}
