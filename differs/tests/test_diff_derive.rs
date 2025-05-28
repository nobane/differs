use differs::{
    changed, diff_changes, Changed,
    Changed::{Added, AddedAt, Moved, Removed, RemovedAt},
    Diff,
    MapChanged::{AddedEntry, ChangedEntry, RemovedEntry},
};
use std::collections::{HashMap, HashSet};

#[derive(Diff, Clone, Debug, PartialEq)]
struct SimpleStruct {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct Person {
    id: u32,
    name: String,
    address: Address,
    tags: Vec<String>,
    roles: HashSet<String>,
    metadata: HashMap<String, String>,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct WithSkippedField {
    included: String,
    #[differs(skip)]
    skipped: String,
}

#[derive(Diff, Clone, Debug, PartialEq, Eq)]
struct Leaf {
    value: i32,
}

#[derive(Diff, Clone, Debug, PartialEq, Eq)]
struct Container {
    scalar: i32,
    leaf: Leaf,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct Bag {
    items: Vec<char>,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct Roles {
    roles: HashSet<&'static str>,
}

#[derive(Diff, Clone, Debug, PartialEq)]
struct Prefs {
    prefs: HashMap<&'static str, &'static str>,
}

#[test]
fn test_no_changes() {
    let old = SimpleStruct {
        name: "John".to_string(),
        age: 30,
        active: true,
    };
    let new = old.clone();

    let changes = diff_changes(&old, &new);
    assert!(changes.is_empty());
}

#[test]
fn test_scalar_changes() {
    let old = SimpleStruct {
        name: "John".to_string(),
        age: 30,
        active: true,
    };
    let mut new = old.clone();
    new.name = "Jane".to_string();
    new.age = 31;

    let changes = diff_changes(&old, &new);
    println!("{changes:?}");
    assert_eq!(changes.len(), 3);

    let mut found_name = false;
    let mut found_age = false;
    let mut found_self = false;

    for change in &changes {
        match change {
            SimpleStructChange::self_(changed) => {
                assert_eq!(changed.name.as_ref(), "Jane");
                assert_eq!(*changed.age, 31);
                assert!(changed.active);
                found_self = true;
            }
            SimpleStructChange::name(name) => {
                assert_eq!(name.as_ref(), "Jane");
                found_name = true;
            }
            SimpleStructChange::age(age) => {
                assert_eq!(**age, 31);
                found_age = true;
            }
            _ => {}
        }
    }

    assert!(found_self);
    assert!(found_name);
    assert!(found_age);
}

#[test]
fn test_whole_struct_change() {
    let old = SimpleStruct {
        name: "John".to_string(),
        age: 30,
        active: true,
    };
    let new = SimpleStruct {
        name: "Jane".to_string(),
        age: 25,
        active: false,
    };

    let changes = diff_changes(&old, &new);

    // Should have self_ change plus individual field changes
    let has_self_change = changes
        .iter()
        .any(|c| matches!(c, SimpleStructChange::self_(_)));
    assert!(has_self_change);
}

#[test]
fn test_nested_struct_changes() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec![],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.name = "Alicia".to_string();
    new.address.city = "Boston".to_string();
    new.address.zip = "02101".to_string();

    let changes = diff_changes(&old, &new);

    let mut found_name = false;
    let mut found_city = false;
    let mut found_zip = false;

    for change in &changes {
        match change {
            PersonChange::name(name) => {
                assert_eq!(name.as_ref(), "Alicia");
                found_name = true;
            }
            PersonChange::address(AddressChange::city(city)) => {
                assert_eq!(city.as_ref(), "Boston");
                found_city = true;
            }
            PersonChange::address(AddressChange::zip(zip)) => {
                assert_eq!(zip.as_ref(), "02101");
                found_zip = true;
            }
            _ => {}
        }
    }

    assert!(found_name);
    assert!(found_city);
    assert!(found_zip);
}

#[test]
fn test_vec_additions_and_removals() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec!["developer".to_string(), "rust".to_string()],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.tags = vec![
        "developer".to_string(),
        "senior".to_string(),
        "backend".to_string(),
    ];

    let changes = diff_changes(&old, &new);

    let mut found_removed_rust = false;
    let mut found_added_senior = false;
    let mut found_added_backend = false;

    for change in &changes {
        if let PersonChange::tags(tag_change) = change {
            match tag_change {
                RemovedAt(_, tag, _) if tag.as_str() == "rust" => {
                    found_removed_rust = true;
                }
                AddedAt(_, tag, _) if tag.as_str() == "senior" => {
                    found_added_senior = true;
                }
                AddedAt(_, tag, _) if tag.as_str() == "backend" => {
                    found_added_backend = true;
                }
                _ => {}
            }
        }
    }

    assert!(found_removed_rust);
    assert!(found_added_senior);
    assert!(found_added_backend);
}

#[test]
fn test_vec_moves() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.tags = vec!["c".to_string(), "a".to_string(), "b".to_string()]; // moved c to front

    let changes = diff_changes(&old, &new);

    let mut found_move = false;

    for change in &changes {
        if let PersonChange::tags(Moved(tag, from, to)) = change {
            if tag.as_str() == "c" && *from == 2 && *to == 0 {
                found_move = true;
            }
        }
    }

    assert!(found_move);
}

#[test]
fn test_hashset_changes() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec![],
        roles: HashSet::from(["admin".to_string(), "user".to_string()]),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.roles.remove("user");
    new.roles.insert("moderator".to_string());

    let changes = diff_changes(&old, &new);

    let mut found_removed_user = false;
    let mut found_added_moderator = false;

    for change in &changes {
        if let PersonChange::roles(role_change) = change {
            match role_change {
                Removed(role) if role.as_str() == "user" => {
                    found_removed_user = true;
                }
                Added(role) if role.as_str() == "moderator" => {
                    found_added_moderator = true;
                }
                _ => {}
            }
        }
    }

    assert!(found_removed_user);
    assert!(found_added_moderator);
}

#[test]
fn test_hashmap_changes() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec![],
        roles: HashSet::new(),
        metadata: HashMap::from([
            ("theme".to_string(), "dark".to_string()),
            ("lang".to_string(), "en".to_string()),
        ]),
    };

    let mut new = old.clone();
    new.metadata.remove("lang");
    new.metadata
        .insert("timezone".to_string(), "UTC".to_string());
    new.metadata
        .insert("theme".to_string(), "light".to_string());

    let changes = diff_changes(&old, &new);

    let mut found_removed_lang = false;
    let mut found_added_timezone = false;
    let mut found_changed_theme = false;

    for change in &changes {
        if let PersonChange::metadata(map_change) = change {
            match map_change {
                RemovedEntry(key, value) if key.as_str() == "lang" && value.as_str() == "en" => {
                    assert!(!found_removed_lang);
                    found_removed_lang = true;
                }
                AddedEntry(key, value) if key.as_str() == "timezone" && value.as_str() == "UTC" => {
                    assert!(!found_added_timezone);
                    found_added_timezone = true;
                }
                ChangedEntry(key) if key.as_str() == "theme" => {
                    assert!(!found_changed_theme);
                    found_changed_theme = true;
                }
                _ => {}
            }
        }
    }

    assert!(found_removed_lang);
    assert!(found_added_timezone);
    assert!(found_changed_theme);
}

#[test]
fn test_skipped_fields() {
    let old = WithSkippedField {
        included: "old".to_string(),
        skipped: "old_skipped".to_string(),
    };

    let new = WithSkippedField {
        included: "new".to_string(),
        skipped: "new_skipped".to_string(),
    };

    let changes = diff_changes(&old, &new);

    // Should only have change for 'included' field, not 'skipped'
    assert_eq!(changes.len(), 2);

    let mut found_self = false;
    let mut found_value = false;

    println!("{changes:?}");
    for ch in changes {
        match &ch {
            WithSkippedFieldChange::self_(changed) => {
                assert_eq!(changed.included.as_ref(), "new");
                assert_eq!(changed.skipped.as_ref(), "new_skipped");

                assert!(!found_self);
                found_self = true;
            }
            WithSkippedFieldChange::included(value) => {
                assert_eq!(value.as_ref(), "new");
                assert!(!found_value);
                found_value = true;
            }
        }
    }
    assert!(found_self);
    assert!(found_value);
}

#[test]
fn test_changed_macro() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec!["rust".to_string()],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.name = "Alicia".to_string();
    new.address.city = "Boston".to_string();
    new.tags.push("backend".to_string());

    let changes = diff_changes(&old, &new);

    let mut name_changed = false;
    let mut city_changed = false;
    let mut tag_added = false;

    for change in &changes {
        changed!(change;
            Person.name(name) => {
                assert_eq!(name.as_ref(), "Alicia");
                name_changed = true;
            };
            Person.address.city(city) => {
                assert_eq!(city.as_ref(), "Boston");
                city_changed = true;
            };
            Person.tags(AddedAt(_, tag, _)) => {
                if tag.as_str() == "backend" {
                    tag_added = true;
                }
            };
        );
    }

    assert!(name_changed);
    assert!(city_changed);
    assert!(tag_added);
}

#[test]
fn test_changed_macro_with_snapshot() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec![],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.address = Address {
        street: "456 Oak Ave".to_string(),
        city: "Boston".to_string(),
        zip: "02101".to_string(),
    };

    let changes = diff_changes(&old, &new);

    let mut snapshot_found = false;

    for change in &changes {
        changed!(change;
            Person.address@(snapshot) => {
                snapshot_found = true;
                assert_eq!(snapshot.street.as_ref(), "456 Oak Ave");
                assert_eq!(snapshot.city.as_ref(), "Boston");
                assert_eq!(snapshot.zip.as_ref(), "02101");
            };
        );
    }

    assert!(snapshot_found);
}

#[test]
fn test_empty_vec_to_populated() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec![],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.tags = vec!["rust".to_string(), "developer".to_string()];

    let changes = diff_changes(&old, &new);

    let mut additions = 0;

    for change in &changes {
        if let PersonChange::tags(AddedAt(_, _, _)) = change {
            additions += 1;
        }
    }

    assert_eq!(additions, 2);
}

#[test]
fn test_populated_vec_to_empty() {
    let old = Person {
        id: 1,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        },
        tags: vec!["rust".to_string(), "developer".to_string()],
        roles: HashSet::new(),
        metadata: HashMap::new(),
    };

    let mut new = old.clone();
    new.tags = vec![];

    let changes = diff_changes(&old, &new);

    let mut removals = 0;

    for change in &changes {
        if let PersonChange::tags(RemovedAt(_, _, _)) = change {
            removals += 1;
        }
    }

    assert_eq!(removals, 2);
}

#[test]
fn scalar_change_yields_variant() {
    let old = Leaf { value: 1 };
    let new = Leaf { value: 2 };

    let diff = diff_changes(&old, &new);

    println!("{diff:?}");

    assert!(matches!(
        diff.as_slice(),
        [
            LeafChange::self_(LeafSnapshot { value: &2 }),
            LeafChange::value(&2)
        ]
    ));
}

#[test]
fn nested_change_is_propagated() {
    let old = Container {
        scalar: 123,
        leaf: Leaf { value: 1 },
    };
    let new = Container {
        scalar: 123,
        leaf: Leaf { value: 42 },
    };

    let diff = diff_changes(&old, &new);

    assert!(diff
        .iter()
        .any(|ch| matches!(ch, ContainerChange::leaf(LeafChange::value(42)))));
}

#[test]
fn vec_move_and_removal() {
    //    0   1   2   3
    // old: A   B   A   A
    // new:     B   A   A   (remove idx 0, move B 1→0)
    let old = Bag {
        items: vec!['A', 'B', 'A', 'A'],
    };
    let new = Bag {
        items: vec!['B', 'A', 'A'],
    };

    let diff = diff_changes(&old, &new);

    let mut moved = false;
    let mut removed = false;

    for ch in diff {
        match ch {
            BagChange::items(Changed::Moved('B', 1, 0)) => moved = true,
            BagChange::items(Changed::RemovedAt(0, 'A', _)) => removed = true,
            _ => {}
        }
    }

    assert!(moved && removed);
}

#[test]
fn vec_added_at_and_added() {
    let old = Bag {
        items: vec!['X', 'Y'],
    };
    let new = Bag {
        items: vec!['X', 'A', 'Y', 'A'],
    };

    let diff = diff_changes(&old, &new);

    assert!(diff
        .iter()
        .any(|ch| matches!(ch, BagChange::items(Changed::AddedAt(1, 'A', _)))));
    assert!(diff
        .iter()
        .any(|ch| matches!(ch, BagChange::items(Changed::AddedAt(3, 'A', _)))));
}

#[test]
fn hashset_added_and_removed() {
    let old = Roles {
        roles: HashSet::from(["admin", "user"]),
    };
    let new = Roles {
        roles: HashSet::from(["admin", "devops"]),
    };

    let diff = diff_changes(&old, &new);

    assert!(diff
        .iter()
        .any(|ch| matches!(ch, RolesChange::roles(Changed::Removed(&"user")))));
    assert!(diff
        .iter()
        .any(|ch| matches!(ch, RolesChange::roles(Changed::Added(&"devops")))));
}

#[test]
fn hashmap_add_remove_entry() {
    let old = Prefs {
        prefs: HashMap::from([("theme", "dark"), ("layout", "grid")]),
    };
    let new = Prefs {
        prefs: HashMap::from([("theme", "dark"), ("notifications", "email")]),
    };

    let diff = diff_changes(&old, &new);

    assert!(diff
        .iter()
        .any(|ch| matches!(ch, PrefsChange::prefs(RemovedEntry(&"layout", &"grid")))));
    assert!(diff.iter().any(|ch| matches!(
        ch,
        PrefsChange::prefs(AddedEntry(&"notifications", &"email"))
    )));
}

#[test]
fn self_snapshot_emitted_when_anything_changed() {
    let old = Leaf { value: 1 };
    let new = Leaf { value: 2 };

    let diff = diff_changes(&old, &new);
    assert!(matches!(diff.first(), Some(LeafChange::self_(_))));
}
