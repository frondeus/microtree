use microtree::{Ast, AstBuilder, GreenBuilder, IntoBuilder};

mod generated;

use generated::*;

fn print(ast: &impl Ast) {
    let s = ast.red().green().to_string();
    let s = s.replace("\t", "\\t").replace("\n", "\\n");
    println!("`{}`", s);
}

fn main() {
    let mut builder = GreenBuilder::default(); // Acts like cache

    let sexp: Value = List::build()
        .fill(
            LParen::build(),
            vec![
                Atom::build("bar").into_dyn(),
                Atom::build("foo").with_pre("\n  ").into_dyn(),
                Cons::build()
                    .fill(
                        LParen::build().with_pre(" "),
                        Atom::build("car").into_builder(),
                        Dot::build().with_pre(" "),
                        Atom::build("cdr").with_pre(" ").into_builder(),
                        RParen::build(),
                    )
                    .into_dyn(),
                Nil::build()
                    .fill(LParen::build().with_pre(" "), RParen::build())
                    .into_dyn(),
            ],
            RParen::build(),
        )
        .into_builder()
        .build(&mut builder);

    println!("DEBUG: {:?}\n", sexp);

    println!("ESCAPED:");
    print(&sexp);

    println!("\nPLAIN:");
    println!("`{}`", sexp.red().green());
}
