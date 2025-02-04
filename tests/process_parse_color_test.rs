use bodo::process::parse_color;
use colored::Color;

#[test]
fn test_parse_color_valid() {
    assert_eq!(parse_color("red"), Some(Color::Red));
    assert_eq!(parse_color("Blue"), Some(Color::Blue));
    assert_eq!(parse_color("BriGhtGreen"), Some(Color::BrightGreen));
    assert_eq!(parse_color("unknown"), None);
}

#[test]
fn test_parse_color_case_insensitivity() {
    assert_eq!(parse_color("ReD"), Some(Color::Red));
    assert_eq!(parse_color("GREEN"), Some(Color::Green));
}

#[test]
fn test_color_line_contains_prefix() {
    use bodo::process::color_line;
    let line = "Hello world";
    let output = color_line("TEST", &Some("cyan".to_string()), line, false);
    assert!(output.contains("TEST"));
    assert!(output.contains("Hello world"));
}
