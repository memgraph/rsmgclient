mod common;

#[test]
fn it_adds_two() {
    common::setup();
    assert_eq!(4, rsmgclient::add_two(2));
}
