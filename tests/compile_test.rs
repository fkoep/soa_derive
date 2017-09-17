#[macro_use]
extern crate soa_derive;

#[derive(Soa)]
pub struct PubTest {
    foo: usize,
    bar: String,
}

#[derive(Soa)]
pub struct NonPubTest {
    foo: usize,
    bar: String,
}
