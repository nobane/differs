use differs::{AsField, FieldName, Fields, HasFields};
use std::collections::{HashMap, HashSet};

#[derive(Fields)]
#[allow(dead_code)]
struct Baz {
    d: &'static str,
}

#[derive(Fields)]
#[allow(dead_code)]
struct Bar {
    c: &'static str,
    b: Baz,
}

#[derive(Fields)]
#[allow(dead_code)]
struct Foo {
    a: i32,
    bar: Bar,
}

#[derive(Fields)]
pub struct Vec2(pub f32, pub f32);

#[derive(Fields)]
pub enum Message {
    Quit,
    Move(i32, i32),
    Write { text: String },
}

#[derive(Fields)]
#[allow(dead_code)]
pub struct SimpleStruct {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Fields)]
#[allow(dead_code)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Fields)]
#[allow(dead_code)]
struct Person {
    id: u32,
    name: String,
    address: Address,
    tags: Vec<String>,
    roles: HashSet<String>,
    metadata: HashMap<String, String>,
}

#[derive(Fields)]
#[allow(dead_code)]
struct WithSkippedField {
    included: String,
    #[differs(skip)]
    skipped: String,
}

#[derive(Fields)]
#[allow(dead_code)]
struct TupleStruct(String, u32, bool);

#[derive(Fields)]
struct UnitStruct;

#[derive(Fields)]
#[allow(dead_code)]
enum MessageComplex {
    Quit,
    Move(i32, i32),
    Write { text: String },
    Complex { id: u32, data: Vec<String> },
}

#[derive(Fields)]
#[allow(dead_code)]
enum SimpleEnum {
    A,
    B,
    C,
}

#[test]
fn scalar_field_path() {
    assert_eq!(Foo::fields().a().as_str(), "a");
}

#[test]
fn nested_field_paths() {
    assert_eq!(Foo::fields().bar().c().as_str(), "bar.c");
    assert_eq!(Foo::fields().bar().b().d().as_str(), "bar.b.d");
}

#[test]
fn tuple_struct_items() {
    assert_eq!(Vec2::fields().item0().as_str(), "item0");
    assert_eq!(Vec2::fields().item1().as_str(), "item1");
}

#[test]
fn enum_variant_paths() {
    assert_eq!(Message::fields().Quit().as_str(), "Quit");
    assert_eq!(Message::fields().Move().item0().as_str(), "Move.item0");
    assert_eq!(Message::fields().Move().item1().as_str(), "Move.item1");
    assert_eq!(Message::fields().Write().text().as_str(), "Write.text");
}

#[test]
fn fieldname_join_helper() {
    let root = FieldName::static_lit("root");
    let child = FieldName::join(root.as_str(), "child");
    assert_eq!(child.as_str(), "root.child");
}

#[test]
fn test_simple_struct_fields() {
    let name_field = SimpleStruct::fields().name();
    let age_field = SimpleStruct::fields().age();
    let active_field = SimpleStruct::fields().active();

    assert_eq!(name_field.as_str(), "name");
    assert_eq!(age_field.as_str(), "age");
    assert_eq!(active_field.as_str(), "active");
}

#[test]
fn test_nested_struct_fields() {
    let address_street = Person::fields().address().street();
    let address_city = Person::fields().address().city();
    let address_zip = Person::fields().address().zip();

    assert_eq!(address_street.as_str(), "address.street");
    assert_eq!(address_city.as_str(), "address.city");
    assert_eq!(address_zip.as_str(), "address.zip");
}

#[test]
fn test_container_fields() {
    let tags_field = Person::fields().tags();
    let roles_field = Person::fields().roles();
    let metadata_field = Person::fields().metadata();

    assert_eq!(tags_field.as_str(), "tags");
    assert_eq!(roles_field.as_str(), "roles");
    assert_eq!(metadata_field.as_str(), "metadata");
}

#[test]
fn test_skipped_fields() {
    let included_field = WithSkippedField::fields().included();
    assert_eq!(included_field.as_str(), "included");

    // The skipped field should not have a method generated
    // This test confirms compilation succeeds without the skipped field method
}

#[test]
fn test_tuple_struct_fields() {
    let item0 = TupleStruct::fields().item0();
    let item1 = TupleStruct::fields().item1();
    let item2 = TupleStruct::fields().item2();

    assert_eq!(item0.as_str(), "item0");
    assert_eq!(item1.as_str(), "item1");
    assert_eq!(item2.as_str(), "item2");
}

#[test]
fn test_unit_struct_fields() {
    let self_field = UnitStruct::fields().self_();
    assert_eq!(self_field.as_str(), "");
}

#[test]
fn test_enum_unit_variants() {
    let quit_field = Message::fields().Quit();
    let a_field = SimpleEnum::fields().A();
    let b_field = SimpleEnum::fields().B();
    let c_field = SimpleEnum::fields().C();

    assert_eq!(quit_field.as_str(), "Quit");
    assert_eq!(a_field.as_str(), "A");
    assert_eq!(b_field.as_str(), "B");
    assert_eq!(c_field.as_str(), "C");
}

#[test]
fn test_enum_tuple_variants() {
    let move_item0 = Message::fields().Move().item0();
    let move_item1 = Message::fields().Move().item1();

    assert_eq!(move_item0.as_str(), "Move.item0");
    assert_eq!(move_item1.as_str(), "Move.item1");
}

#[test]
fn test_enum_struct_variants() {
    let write_text = MessageComplex::fields().Write().text();
    let complex_id = MessageComplex::fields().Complex().id();
    let complex_data = MessageComplex::fields().Complex().data();

    assert_eq!(write_text.as_str(), "Write.text");
    assert_eq!(complex_id.as_str(), "Complex.id");
    assert_eq!(complex_data.as_str(), "Complex.data");
}

#[test]
fn test_as_field_trait() {
    // Test that FieldName implements AsField
    let field = Person::fields().name();
    let as_field = field.as_field();
    assert_eq!(as_field.as_str(), "name");

    // Test that &str implements AsField
    let str_field: &str = "test.field";
    let as_field = str_field.as_field();
    assert_eq!(as_field.as_str(), "test.field");

    // Test that String implements AsField
    let string_field = "another.test".to_string();
    let as_field = string_field.as_field();
    assert_eq!(as_field.as_str(), "another.test");
}

#[test]
fn test_field_name_methods() {
    let field = Person::fields().address().city();

    // Test as_str method
    assert_eq!(field.as_str(), "address.city");

    // Test Debug formatting
    let debug_str = format!("{:?}", field);
    assert!(debug_str.contains("address.city"));

    // Test Clone
    let cloned = field.clone();
    assert_eq!(cloned.as_str(), field.as_str());

    // Test PartialEq
    let same_field = Person::fields().address().city();
    assert_eq!(field, same_field);

    let different_field = Person::fields().address().street();
    assert_ne!(field, different_field);
}

#[test]
fn test_has_fields_trait() {
    // Test that all types properly implement HasFields
    let _ = SimpleStruct::fields();
    let _ = Person::fields();
    let _ = Address::fields();
    let _ = TupleStruct::fields();
    let _ = UnitStruct::fields();
    let _ = Message::fields();
    let _ = SimpleEnum::fields();
}

#[test]
fn test_nested_field_chaining() {
    // Test complex field path construction
    let nested_field = Person::fields().address().street();
    assert_eq!(nested_field.as_str(), "address.street");

    // Verify the intermediate types work correctly
    let address_fields = Person::fields().address();
    let street_from_intermediate = address_fields.street();
    assert_eq!(street_from_intermediate.as_str(), "address.street");
}

#[test]
fn test_field_name_static_construction() {
    // Test the internal static_lit method (used by derive macro)
    let static_field = FieldName::static_lit("test");
    assert_eq!(static_field.as_str(), "test");

    // Test from_string method
    let string_field = FieldName::from_string("dynamic.test".to_string());
    assert_eq!(string_field.as_str(), "dynamic.test");

    // Test join method
    let joined = FieldName::join("prefix", "suffix");
    assert_eq!(joined.as_str(), "prefix.suffix");

    let joined_empty_prefix = FieldName::join("", "field");
    assert_eq!(joined_empty_prefix.as_str(), "field");

    let joined_empty_suffix = FieldName::join("prefix", "");
    assert_eq!(joined_empty_suffix.as_str(), "prefix");
}

#[test]
fn test_enum_proxy_as_field() {
    // Test that enum variant proxies implement AsField correctly
    let move_proxy = Message::fields().Move();
    let move_as_field = move_proxy.as_field();
    assert_eq!(move_as_field.as_str(), "Move");

    let write_proxy = Message::fields().Write();
    let write_as_field = write_proxy.as_field();
    assert_eq!(write_as_field.as_str(), "Write");

    let complex_proxy = MessageComplex::fields().Complex();
    let complex_as_field = complex_proxy.as_field();
    assert_eq!(complex_as_field.as_str(), "Complex");
}

#[test]
fn test_root_fields_as_field() {
    // Test that the root Fields struct implements AsField
    let person_fields = Person::fields();
    let as_field = person_fields.as_field();
    assert_eq!(as_field.as_str(), "");

    // Test with a nested fields struct
    let address_fields = Person::fields().address();
    let address_as_field = address_fields.as_field();
    assert_eq!(address_as_field.as_str(), "address");
}

#[test]
fn test_multiple_enum_variants_same_name_different_types() {
    // This tests that we can have multiple variants with similar structures
    // without naming conflicts in the generated code
    let write_text = MessageComplex::fields().Write().text();
    let complex_id = MessageComplex::fields().Complex().id();

    assert_eq!(write_text.as_str(), "Write.text");
    assert_eq!(complex_id.as_str(), "Complex.id");

    // Ensure they're different types/paths
    assert_ne!(write_text.as_str(), complex_id.as_str());
}

#[test]
fn test_field_comparison_and_hashing() {
    let field1 = Person::fields().name();
    let field2 = Person::fields().name();
    let field3 = Person::fields().address().city();

    // Test equality
    assert_eq!(field1, field2);
    assert_ne!(field1, field3);

    // Test hashing (by using in HashMap)
    let mut map = HashMap::new();
    map.insert(field1.clone(), "value1");
    map.insert(field3.clone(), "value2");

    assert_eq!(map.get(&field2), Some(&"value1"));
    assert_eq!(map.len(), 2);
}
