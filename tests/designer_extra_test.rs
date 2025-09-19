#[test]
fn test_designer_module_public_constant() {
    // The designer module should export a public constant EMPTY equal to ()
    #[allow(clippy::let_unit_value)]
    let empty_value = bodo::designer::EMPTY;
    assert_eq!(std::mem::size_of_val(&empty_value), 0);
}
