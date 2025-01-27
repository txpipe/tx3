use assert_json_diff::assert_json_eq;
use paste::paste;
use tx3_lang::ast::Program;

#[allow(dead_code)]
fn make_snapshot(example: &str, program: &Program) {
    let ast = serde_json::to_string_pretty(program).unwrap();
    std::fs::write(format!("tests/{}.ast", example), ast).unwrap();
}

fn test_parsing_example(example: &str) {
    let input = std::fs::read_to_string(format!("tests/{}.tx3", example)).unwrap();
    let program = tx3_lang::parser::Tx3Parser::parse_program(&input).unwrap();

    // Uncomment to update AST snapshots
    make_snapshot(example, &program);

    let ast = std::fs::read_to_string(format!("tests/{}.ast", example)).unwrap();
    let expected: Vec<Program> = serde_json::from_str(&ast).unwrap();

    assert_json_eq!(program, expected);
}

#[macro_export]
macro_rules! test_parsing {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<test_example_ $name>]() {
                test_parsing_example(stringify!($name));
            }
        }
    };
}

//test_parsing!(swap);
test_parsing!(asteria);
