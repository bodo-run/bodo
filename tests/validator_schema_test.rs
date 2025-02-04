use bodo::config::BodoConfig;

#[test]
fn test_generate_schema_non_empty() {
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty());
    // Check that the schema contains the BodoConfig title.
    assert!(schema.contains("\"title\": \"BodoConfig\""));
}
