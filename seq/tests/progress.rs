#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/01-parse-header.rs");
    //t.pass("examples/02-parse-body.rs");
    //t.compile_fail("examples/03-expand-four-errors.rs");
    //t.pass("examples/04-paste-ident.rs");
    //t.pass("examples/05-repeat-section.rs");
    //t.pass("examples/06-init-array.rs");
    //t.pass("examples/07-inclusive-range.rs");
    //t.compile_fail("examples/08-ident-span.rs");
    //t.pass("examples/09-interaction-with-macrorules.rs");
}
