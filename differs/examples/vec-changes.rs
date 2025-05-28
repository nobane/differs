
use differs::{
    changed, diff_changes,
    Changed::{Added, AddedAt, Moved, Removed, RemovedAt},
    Diff,
};

#[derive(Diff, Clone, Debug, PartialEq)]
struct Bag {
    items: Vec<char>,
}

fn dump(label: &str, changes: &[BagChange]) {
    println!("\n=== {label} ===");
    for ch in changes {
        changed!(ch;
            Bag.items(Added(v))             => { println!("Added({v:?})"); };
            Bag.items(Removed(v))           => { println!("Removed({v:?})"); };
            Bag.items(AddedAt(i, v, c))     => { println!("AddedAt(idx={i:<2}, val={v:?}, c=#{c})"); };
            Bag.items(RemovedAt(i, v, c))   => { println!("RemovedAt(idx={i:<2}, val={v:?},c=#{c})"); };
            Bag.items(Moved(v,from,to))     => { println!("Moved(val={v:?}, from={from}, to={to})"); };
        );
    }
}

fn main() {
    // 1. Tail-removal of a duplicate
    let old = Bag {
        items: vec!['A', 'A', 'A', 'A'],
    };
    let new = Bag {
        items: vec!['A', 'A', 'A'],
    };
    dump("[A, A, A, A] -> [A, A, A]", &diff_changes(&old, &new));

    // 2. Remove first element ⇒ everything shifts left
    let old = Bag {
        items: vec!['A', 'B', 'C'],
    };
    let new = Bag {
        items: vec!['B', 'C'],
    };
    dump("[A, B, C] -> [B, C]", &diff_changes(&old, &new));

    // 3. Mixed removal + moves with duplicates
    let old = Bag {
        items: vec!['A', 'B', 'A', 'A'],
    };
    let new = Bag {
        items: vec!['B', 'A', 'A'],
    };
    dump("[A, B, A, A] -> [B, A, A]", &diff_changes(&old, &new));

    // 4. Pure insertions (one value appears for the first time, then again)
    let old = Bag {
        items: vec!['X', 'Y'],
    };
    let new = Bag {
        items: vec!['X', 'A', 'Y', 'A'],
    };
    dump("[X,Y] -> [X,A, Y,A]", &diff_changes(&old, &new));
}
