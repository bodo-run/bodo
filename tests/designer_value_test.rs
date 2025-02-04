#[test]
fn test_designer_empty_value() {
    // Ensure that the designer module's public constant EMPTY equals ()
    assert_eq!(bodo::designer::EMPTY, ());
}
