#[test]
fn test_trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/*.rs");
    t.compile_fail("tests/not_compile/*.rs");
}
