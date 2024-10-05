// Wrap custom message to reduce repetition
#[macro_export]
macro_rules! assert_expr_eq {
    ($expr: expr, $expect: expr) => {
        let result = eval::eval($expr).unwrap();

        assert_eq!(
            result, $expect,
            "\nexpected expression \"{}\" to equal \"{:?}\", but got \"{:?}\"",
            $expr, $expect, result,
        );
    };
}
