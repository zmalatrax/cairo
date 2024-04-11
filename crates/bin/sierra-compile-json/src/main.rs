use std::fs;

use anyhow::Context;
use cairo_lang_sierra::ProgramParser;
use cairo_lang_sierra_to_casm::{compiler::CasmCairoProgram, metadata::calc_metadata};
use cairo_lang_utils::logging::init_logging;
use clap::Parser;
use indoc::indoc;

/// Command line args parser.
/// Exits with 0/1 if the input is formatted correctly/incorrectly.
#[derive(Parser, Debug)]
#[clap(version, verbatim_doc_comment)]
struct Args {
    /// The file to compile
    file: String,
    output: String,
}


fn main() -> anyhow::Result<()> {
    init_logging(log::LevelFilter::Off);
    log::info!("Starting Sierra compilation.");

    let args = Args::parse();

    let sierra_code = fs::read_to_string(args.file).with_context(|| "Could not read file!")?;
    let Ok(program) = ProgramParser::new().parse(&sierra_code) else {
        anyhow::bail!(indoc! {"
            Failed to parse sierra program.
            Note: StarkNet contracts should be compiled with `starknet-sierra-compile`."
        })
    };

    let gas_usage_check = true;
    let cairo_program = cairo_lang_sierra_to_casm::compiler::compile(
        &program,
        &calc_metadata(&program, Default::default())
            .with_context(|| "Failed calculating Sierra variables.")?,
        gas_usage_check,
    )
    .with_context(|| "Compilation failed.")?;

    // If the main_function is exposed, the builtins can later be computed so it's OK
    // let (builtins, builtin_offset) = get_function_builtins(func, append_output);
    let main_func = program.find_function("::main")?;

    let casm_cairo_program = CasmCairoProgram::new(&cairo_program, main_func);

    let res = serde_json::to_string_pretty(&casm_cairo_program)
        .with_context(|| "Serialization failed.")?;

    fs::write(args.output, res).with_context(|| "Failed to write output.")
}