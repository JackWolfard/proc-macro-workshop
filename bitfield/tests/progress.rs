#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/01-specifier-types.rs");
    //t.pass("examples/02-storage.rs");
    //t.pass("examples/03-accessors.rs");
    //t.compile_fail("examples/04-multiple-of-8bits.rs");
    //t.pass("examples/05-accessor-signatures.rs");
    //t.pass("examples/06-enums.rs");
    //t.pass("examples/07-optional-discriminant.rs");
    //t.compile_fail("examples/08-non-power-of-two.rs");
    //t.compile_fail("examples/09-variant-out-of-range.rs");
    //t.pass("examples/10-bits-attribute.rs");
    //t.compile_fail("examples/11-bits-attribute-wrong.rs");
    //t.pass("examples/12-accessors-edge.rs");
}
