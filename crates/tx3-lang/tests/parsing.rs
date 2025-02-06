use assert_json_diff::assert_json_eq;
use paste::paste;
use tx3_lang::ast::Program;

fn make_snapshot_if_missing(example: &str, program: &Program) {
    let path = format!("tests/{}.ast", example);

    if !std::fs::exists(&path).unwrap() {
        let ast = serde_json::to_string_pretty(program).unwrap();
        std::fs::write(&path, ast).unwrap();
    }
}

fn test_parsing_example(example: &str) {
    let program = tx3_lang::parse::parse_file(&format!("tests/{}.tx3", example)).unwrap();

    make_snapshot_if_missing(example, &program);

    let ast = std::fs::read_to_string(format!("tests/{}.ast", example)).unwrap();
    let expected: Program = serde_json::from_str(&ast).unwrap();

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

test_parsing!(swap);
test_parsing!(asteria);
