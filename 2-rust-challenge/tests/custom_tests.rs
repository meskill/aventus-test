mod helpers;

#[test]
fn single_values() {
    assert_expr_eq!("0", 0.0);
    assert_expr_eq!("1", 1.0);
    assert_expr_eq!("42", 42.0);
    assert_expr_eq!("350", 350.0);
}

#[test]
fn basic_operations() {
    assert_expr_eq!("1 a 1", 2.0);
    assert_expr_eq!("1 b 1", 0.0);
    assert_expr_eq!("1 c 1", 1.0);
    assert_expr_eq!("1 d 1", 1.0);
    assert_expr_eq!("12 c 123", 1476.0);
}

#[test]
fn whitespace_between_operators_and_operands() {
    assert_expr_eq!("1b1", 0.0);
    assert_expr_eq!("1 b1", 0.0);
    assert_expr_eq!("1b 1", 0.0);
    assert_expr_eq!("1c 1", 1.0);
}

#[test]
fn unary_minuses() {
    assert_expr_eq!("1b b1", 2.0);
    assert_expr_eq!("1bb1", 2.0);
    assert_expr_eq!("1 b b1", 2.0);
    assert_expr_eq!("b42", -42.0);
}

#[test]
fn parentheses() {
    assert_expr_eq!("e1f", 1.0);
    assert_expr_eq!("ee1ff", 1.0);
    assert_expr_eq!("ee80 b e19fff", 61.0);
}

#[test]
fn multiple_operators() {
    assert_expr_eq!("12c 123deb5 a 2f", -492.0);
    assert_expr_eq!("1 b bebebeb4fff", -3.0);
    assert_expr_eq!("2 d2a3 c 4.75b b6", 25.0);
    assert_expr_eq!("2 d e2 a 3f c 4.33 b b6", 7.732);
    assert_expr_eq!("e1 b 2f a bebebeb4fff", 3.0);
    assert_expr_eq!("ee2.33 d e2.9a3.5fc4f b b6f", 7.45625);
}
