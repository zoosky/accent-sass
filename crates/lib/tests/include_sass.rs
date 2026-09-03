#[cfg(feature = "macro")]
#[test]
fn basic() {
    let css: &str = accent_sass::include!("./input.scss");

    assert_eq!(css, "a{color:red}");
}
