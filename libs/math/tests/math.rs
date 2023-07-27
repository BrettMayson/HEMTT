#[test]
fn basic() {
    assert_eq!(hemtt_math::eval("1 + 1"), Some(2.0));
    assert_eq!(hemtt_math::eval("1 + 1 * 2"), Some(3.0));
    assert_eq!(hemtt_math::eval("1 + 1 * 2 / 2"), Some(2.0));
    assert_eq!(hemtt_math::eval("1 - 1"), Some(0.0));
    assert_eq!(hemtt_math::eval("1 - 1 * 2"), Some(-1.0));
    assert_eq!(hemtt_math::eval("1 - 1 * 2 / 2"), Some(0.0));
}

#[test]
fn parens() {
    assert_eq!(hemtt_math::eval("(1 + 1) * 2"), Some(4.0));
    assert_eq!(hemtt_math::eval("1 + (1 * 2)"), Some(3.0));
    assert_eq!(hemtt_math::eval("1 + (1 * 2) / 2"), Some(2.0));
    assert_eq!(hemtt_math::eval("1 - (1 + 1)"), Some(-1.0));
    assert_eq!(hemtt_math::eval("1 - (1 * 2)"), Some(-1.0));
    assert_eq!(hemtt_math::eval("1 - (1 * 2) / 2"), Some(0.0));
}

#[test]
fn negation() {
    assert_eq!(hemtt_math::eval("1 + -1"), Some(0.0));
    assert_eq!(hemtt_math::eval("1 + -1 * 2"), Some(-1.0));
    assert_eq!(hemtt_math::eval("1 + -1 * 2 / 2"), Some(0.0));
    assert_eq!(hemtt_math::eval("1 - -1"), Some(2.0));
}
