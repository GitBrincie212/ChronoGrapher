#[test]
fn pass_cases() {
    let t = trybuild::TestCases::new();
    t.pass("tests/main_macro/pass/*.rs");
}

#[test]
fn fail_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/main_macro/fail/*.rs");
}
