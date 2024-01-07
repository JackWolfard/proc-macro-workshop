#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/01-parse.rs");
    t.pass("examples/02-impl-debug.rs");
    t.pass("examples/03-custom-format.rs");
    t.pass("examples/04-type-parameter.rs");
    //t.pass("examples/05-phantom-data.rs");
    //t.pass("examples/06-bound-trouble.rs");
    //t.pass("examples/07-associated-type.rs");
    //t.pass("examples/08-escape-hatch.rs");
}
