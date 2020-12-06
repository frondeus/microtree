use microtree::GreenMutate;
use microtree::Red;
use microtree::{Ast, AstBuilder, Cache};

mod generated;

use generated::*;

fn print(ast: &impl Ast) {
    let s = ast.red().green().to_string();
    let s = s.replace("\t", "\\t").replace("\n", "\\n");
    println!("`{}`", s);
}

fn main() {
    let mut builder = Cache::default(); // Acts like cache
    let str = String::build()
        .fill(
            DQuote::build().with_leading("\t"),
            StringVal::build("Ala ma kota"),
            DQuote::build().with_trailing("\n"),
        )
        .build(&mut builder);

    print(&str);

    // Internal value between two doublequotes.
    let new_str = StringVal::build("Kot ma ale").build_green(&mut builder);

    let string_val = str.value_token().unwrap();

    let new_str = String::new(Red::root(string_val.red().replace(&mut builder, new_str))).unwrap();

    println!("New str: ");
    print(&new_str);

    use microtree::IntoBuilder;

    let num: Value = Number::build(4).into_builder().build(&mut builder);

    let num_val = num.as_number().unwrap().value().unwrap();
    println!("Number: {}", num_val);

    let array: Value = Array::build()
        .fill(
            LBracket::build(),
            vec![
                Number::build(4).with_leading("\n    ").into_dyn(),
                String::build()
                    .fill(
                        DQuote::build().with_leading("\n    "),
                        StringVal::build("Test"),
                        DQuote::build(),
                    )
                    .into_dyn(),
            ],
            Comma::build(),
            RBracket::build().with_leading("\n"),
        )
        .into_builder()
        .build(&mut builder);

    print(&array);

    println!("`{}`", array.red().green());

    let values = array.as_array().unwrap().values().collect::<Vec<_>>();
    for value in values {
        print(&value);
    }
}
