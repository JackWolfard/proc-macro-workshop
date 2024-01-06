#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/01-parse.rs");
    t.pass("examples/02-create-builder.rs");
    t.pass("examples/03-call-setters.rs");
    t.pass("examples/04-call-build.rs");
    //t.pass("examples/05-method-chaining.rs");
    //t.pass("examples/06-optional-field.rs");
    //t.pass("examples/07-repeated-field.rs");
    //t.compile_fail("examples/08-unrecognized-attribute.rs");
    //t.pass("examples/09-redefined-prelude-types.rs");
}
