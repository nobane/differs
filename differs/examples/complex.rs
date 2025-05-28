use differs::{
    changed, diff_changes,
    Changed::{Added, AddedAt, Moved, Removed, RemovedAt},
    Diff, Fields,
    MapChanged::{AddedEntry, ChangedEntry, RemovedEntry},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

#[derive(Diff, Fields, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
struct Address {
    street: String,
    city: String,
}

#[derive(Diff, Fields, Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    id: u32,
    username: String,
    password: String,
    roles: HashSet<String>,
    preferences: HashMap<String, String>,
    address: Address,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Account {}
impl Hash for Account {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.id.hash(s)
    }
}

#[derive(Diff, Fields, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Company {
    name: String,
    staff: Vec<Account>,
}

fn dump_company(changes: &[CompanyChange<'_>]) {
    for ch in changes {
        changed!(ch;
            Company.name(v)                     => { println!("Company.name({v})"); };

            Company.staff(Added(acc))           => { println!("Company.staff(Added({}))",  acc.username); };
            Company.staff(Removed(acc))         => { println!("Company.staff(Removed({}))", acc.username); };

            Company.staff(AddedAt(i, acc, _))   => { println!("Company.staff(AddedAt({i}, {}))",  acc.username); };
            Company.staff(RemovedAt(i, acc, _)) => { println!("Company.staff(RemovedAt({i}, {}))", acc.username); };

            Company.staff(Moved(acc, f, t))     => { println!("Company.staff(Moved({}, {f}->{t}))", acc.username); };
        );
    }
}

fn dump_account(changes: &[AccountChange<'_>]) {
    for ch in changes {
        changed!(ch;
            /* scalar inside nested struct */
            Account.address.city(v)                     => {
                println!("Account.address.city({v})");
            };

            /* HashSet<String> */
            Account.roles(Added(role))                  => {
                println!("Account.roles(Added({role}))");
            };
            Account.roles(Removed(role))                => {
                println!("Account.roles(Removed({role}))");
            };

            /* HashMap<String,String> */
            Account.preferences(AddedEntry(k, v))       => {
                println!("Account.preferences(AddedEntry({k}, {v}))");
            };
            Account.preferences(RemovedEntry(k, v))     => {
                println!("Account.preferences(RemovedEntry({k}, {v}))");
            };
            Account.preferences(ChangedEntry(k))        => {
                println!("Account.preferences(ChangedEntry({k}))");
            };
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* baseline */
    let old = Company {
        name: "Acme Inc.".into(),
        staff: vec![
            Account {
                id: 1,
                username: "alice".into(),
                password: "secret".into(),
                roles: HashSet::from(["admin".into()]),
                preferences: HashMap::from([("theme".into(), "dark".into())]),
                address: Address {
                    street: "1 Main".into(),
                    city: "Paris".into(),
                },
            },
            Account {
                id: 2,
                username: "bob".into(),
                password: "hunter2".into(),
                roles: HashSet::from(["user".into()]),
                preferences: HashMap::new(),
                address: Address {
                    street: "99 Broadway".into(),
                    city: "London".into(),
                },
            },
        ],
    };

    /* edited version  */
    let mut new = old.clone();
    new.name = "Acme Corp.".into();
    new.staff[0].address.city = "Berlin".into();
    new.staff[0].roles.insert("devops".into());
    new.staff[1].roles.remove("user");
    new.staff[0]
        .preferences
        .insert("notifications".into(), "email".into());

    /* insert + move */
    let charlie = Account {
        id: 3,
        username: "charlie".into(),
        password: "pwd".into(),
        roles: HashSet::from(["user".into()]),
        preferences: HashMap::new(),
        address: Address {
            street: "5 High".into(),
            city: "Madrid".into(),
        },
    };
    new.staff.insert(0, charlie); // AddedAt
    new.staff.swap(1, 2); // Moved

    println!("Company diff");
    let diff = diff_changes(&old, &new);
    dump_company(&diff);

    println!("\n Alice diff");
    let old_alice = old.staff.iter().find(|a| a.id == 1).unwrap();
    let new_alice = new.staff.iter().find(|a| a.id == 1).unwrap();
    let alice_diff = diff_changes(old_alice, new_alice);
    dump_account(&alice_diff);

    Ok(())
}
