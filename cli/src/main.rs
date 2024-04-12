use std::{io::Write, path::PathBuf};

use clap::Parser;
use color_eyre::Result;

mod path;

#[derive(Parser, Debug)]
struct Args {
    /// Defines where your TS bindings will be saved by setting TS_RS_EXPORT_DIR
    #[arg(long, short, default_value = "./bindings")]
    output_directory: PathBuf,

    /// Disables warnings caused by using serde attributes that ts-rs cannot process
    #[arg(long)]
    no_warnings: bool,

    /// Adds the ".js" extension to import paths
    #[arg(long)]
    esm_imports: bool,

    /// Formats the generated TypeScript files
    #[arg(long)]
    format: bool,

    #[arg(long = "index")]
    generate_index_ts: bool,
}

macro_rules! feature {
    ($cargo_invocation: expr, $args: expr, { $($field: ident => $feature: literal),* $(,)? }) => {
        $(
            if $args.$field {
                $cargo_invocation
                    .arg("--features")
                    .arg(format!("ts-rs/{}", $feature));
            }
        )*
    };
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let mut cargo_invocation = std::process::Command::new("cargo");

    let metadata_path = args.output_directory.join("ts_rs.meta");
    if metadata_path.exists() {
        std::fs::remove_file(&metadata_path)?;
    }

    cargo_invocation
        .arg("test")
        .arg("export_bindings_")
        .arg("--features")
        .arg("ts-rs/export")
        .env("TS_RS_EXPORT_DIR", path::absolute(&args.output_directory)?);

    feature!(cargo_invocation, args, {
        no_warnings => "no-serde-warnings",
        esm_imports => "import-esm",
        format => "format",
    });

    cargo_invocation.spawn()?.wait()?;

    if args.generate_index_ts {
        let metadata = std::fs::read_to_string(&metadata_path)?
            .lines()
            .map(ToOwned::to_owned)
            .collect::<std::collections::HashSet<_>>();

        let index_path = args.output_directory.join("index.ts");

        if index_path.exists() {
            std::fs::remove_file(&index_path)?;
        }

        let mut index = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(index_path)?;

        for file in metadata.iter() {
            index.write_fmt(format_args!("export * from {file:?};\n"))?;
        }
    }

    std::fs::remove_file(&metadata_path)?;

    Ok(())
}
