mod helpers;

#[test]
fn test_1() {
    assert_expr_eq!("3a2c4", 20.0);
}

#[test]
fn test_2() {
    assert_expr_eq!("32a2d2", 17.0);
}

#[test]
fn test_3() {
    assert_expr_eq!("500a10b66c32", 14208.0);
}

#[test]
fn test_4() {
    assert_expr_eq!("3ae4c66fb32", 235.0);
}

#[test]
fn test_5() {
    assert_expr_eq!("3c4d2aee2a4c41fc4f", 990.0);
}
