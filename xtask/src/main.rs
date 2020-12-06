use anyhow::Result;
use microtree_codegen::codegen;
use pico_args::Arguments;

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcmd = args.subcommand()?.unwrap_or_default();

    match subcmd.as_str() {
        "codegen" => {
            codegen(
                "crates/lib/examples/json/json.config.json",
                "crates/lib/examples/json/json.ungram",
                "crates/lib/examples/json/generated/",
            )?;

            codegen(
                "crates/lib/examples/sexp/sexp.config.json",
                "crates/lib/examples/sexp/sexp.ungram",
                "crates/lib/examples/sexp/generated/",
            )?;

            codegen(
                "crates/parser/examples/sexp/sexp.config.json",
                "crates/parser/examples/sexp/sexp.ungram",
                "crates/parser/examples/sexp/generated/",
            )?;

            codegen(
                "crates/parser/examples/modes/modes.config.json",
                "crates/parser/examples/modes/modes.ungram",
                "crates/parser/examples/modes/generated/",
            )?;
        }
        _ => eprintln!("cargo xtask codegen"),
    }

    Ok(())
}
