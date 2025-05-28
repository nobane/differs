use differs::{Fields, HasFields as _};
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
    z: Baz,
}
#[derive(Fields, Serialize, Deserialize, Debug, Clone)]
struct Baz {
    d: String,
}
#[derive(Fields)]
pub struct Vec2(pub f32, pub f32);

#[derive(Fields)]
pub enum Message {
    Quit,                   // "Quit"
    Move(i32, i32),         // "Move.item0", "Move.item1"
    Write { text: String }, // "Write.text"
}

fn main() {
    println!("a   = {:?}", Foo::fields().a());
    println!("bar.c = {:?}", Foo::fields().bar().c());
    println!("bar.b.d = {:?}", Foo::fields().bar().b().d());
    println!("bar.z.d = {:?}", Foo::fields().bar().z().d());
    println!("{:?}", Vec2::fields().item0()); // "item0"
    println!("{:?}", Message::fields().Quit()); // "Quit"
    println!("{:?}", Message::fields().Move().item1()); // "Move.item1"
    println!("{:?}", Message::fields().Write().text()); // "Write.text"
}
