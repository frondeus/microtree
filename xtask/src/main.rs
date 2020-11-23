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
        }
        _ => eprintln!("cargo xtask codegen"),
    }

    Ok(())
}
