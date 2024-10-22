use std::{fs, path::PathBuf};

use anyhow::Context;
use cairo_lang_sierra_to_casm::{
    compiler::{compile, SierraToCasmConfig, CasmCairoProgram},
    metadata::calc_metadata_ap_change_only,
};
use clap::Parser;
use cairo_lang_compiler::{
    compile_prepared_db, db::RootDatabase, project::setup_project, CompilerConfig,
};

/// Compiles a Sierra file (Cairo Program) into serialized CASM.
/// Exits with 0/1 if the compilation succeeds/fails.
#[derive(Parser, Debug)]
#[clap(version, verbatim_doc_comment)]
struct Args {
    /// The path of the file to compile.
    file: String,
    /// The output file name (default: stdout).
    output: Option<String>,
    /// Add gas usage check
    #[arg(long, default_value_t = false)]
    gas_usage_check: bool,
    #[arg(long, default_value_t = false)]
    /// Add pythonic hints
    add_pythonic_hints: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = std::fs::read(&args.file)?;
    let filename = PathBuf::from(&args.file);
    let sierra_program = match serde_json::from_slice(&file) {
        Ok(program) => program,
        Err(_) => {
            let compiler_config = CompilerConfig {
                replace_ids: true,
                ..CompilerConfig::default()
            };
            let mut db = RootDatabase::builder()
                .detect_corelib()
                .skip_auto_withdraw_gas()
                .build()
                .unwrap();
            let main_crate_ids = setup_project(&mut db, &filename).unwrap();
            let sierra_program_with_dbg =
                compile_prepared_db(&mut db, main_crate_ids, compiler_config).unwrap();

            sierra_program_with_dbg.program
        }
    };
    let metadata = calc_metadata_ap_change_only(&sierra_program)
        .map_err(|_| anyhow::anyhow!("Failed calculating Sierra variables"))?;
    let config = SierraToCasmConfig {
        gas_usage_check: false,
        max_bytecode_size: usize::MAX,
    };
    let cairo_program =
        compile(&sierra_program, &metadata, config)?;
    let casm_cairo_program =
        CasmCairoProgram::new(&sierra_program, &cairo_program, args.add_pythonic_hints)
            .with_context(|| "Sierra to Casm compilation failed.")?;

    let res = serde_json::to_string(&casm_cairo_program)
        .with_context(|| "Casm contract Serialization failed.")?;

    match args.output {
        Some(path) => fs::write(path, res).with_context(|| "Failed to write casm contract.")?,
        None => println!("{res}"),
    }
    Ok(())
}
