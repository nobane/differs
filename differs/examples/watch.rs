use differs::{ChangeEventBus, Fields, HasFields as _};
use serde::{Deserialize, Serialize};

#[derive(Fields, Serialize, Deserialize, Debug, Clone)]
struct Foo {
    a: i64,
    bar: Bar,
}

#[derive(Fields, Serialize, Deserialize, Debug, Clone)]
struct Bar {
    c: String,
    b: Baz,
}
#[derive(Fields, Serialize, Deserialize, Debug, Clone)]
struct Baz {
    d: String,
}

fn main() {
    let mut bus = ChangeEventBus::new();
    
    // Subscribe to a simple scalar
    let rx_a = bus.subscribe(Foo::fields().a());

    // Subscribe to a nested field
    let rx_c = bus.subscribe(Foo::fields().bar().c());

    // Pretend our file-watcher detected changes:
    bus.publish("a", "42".into());
    bus.publish("bar.c", "\"updated\"".into());

    println!("got a   = {}", rx_a.recv().unwrap());
    println!("got bar.c = {}", rx_c.recv().unwrap());
}
