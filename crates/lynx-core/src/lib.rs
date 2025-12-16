pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn swc_parser_is_available() {
        use swc_ecma_parser::{Parser, StringInput, Syntax};

        let src = "const x = 1;";
        let input = StringInput::new(src, Default::default(), Default::default());
        let syntax = Syntax::Es(Default::default());
        let _ = Parser::new(syntax, input, None);
    }
}
