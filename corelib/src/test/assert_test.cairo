#[test]
#[should_panic(expected: "Assertion failed")]
fn test_assert_macro_no_args() {
    assert!(1 == 1);
    assert!(1 == 2);
}

#[test]
#[should_panic(expected: "Assertion failed: panic data")]
fn test_assert_macro_single_arg() {
    let ba: ByteArray = "panic data";
    assert!(1 == 1, ba);
    assert!(1 == 2, ba);
}

#[test]
#[should_panic(expected: "Assertion failed: panic data")]
fn test_assert_macro_two_args_byte_array() {
    let ba: ByteArray = "panic data";
    assert!(1 == 1, "{}", ba);
    assert!(1 == 2, "{}", ba);
}

#[test]
#[should_panic(expected: "Assertion failed: 97")]
fn test_assert_macro_two_args_usize() {
    assert!(1 == 1, "{}", 97_usize);
    assert!(1 == 2, "{}", 97_usize);
}

#[test]
#[should_panic(expected: "Assertion failed: 97")]
fn test_assert_macro_two_args_felt252() {
    assert!(1 == 1, "{}", 97_felt252);
    assert!(1 == 2, "{}", 97_felt252);
}
