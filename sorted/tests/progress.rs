#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/01-parse-enum.rs");
    t.compile_fail("examples/02-not-enum.rs");
    t.compile_fail("examples/03-out-of-order.rs");
    //t.compile_fail("examples/04-variants-with-data.rs");
    //t.compile_fail("examples/05-match-expr.rs");
    //t.compile_fail("examples/06-pattern-path.rs");
    //t.compile_fail("examples/07-unrecognized-pattern.rs");
    //t.pass("examples/08-underscore.rs");
}
