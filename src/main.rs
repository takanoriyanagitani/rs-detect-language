use std::io;
use std::process::ExitCode;

use clap::Parser;
use lingua::LanguageDetector;

use rs_detect_language::Config;
use rs_detect_language::print_detected_from_stdin;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(long)]
    minimum_relative_distance: Option<f64>,
    #[arg(long)]
    low_accuracy_mode: bool,
    #[arg(long)]
    preload: bool,
    #[arg(long)]
    spoken_language_only: bool,
    #[arg(long)]
    max_output_languages: Option<usize>,
    #[arg(long, default_value_t = 1048576)]
    max_input_sample_bytes: u64,
}

fn sub(cli: Cli) -> Result<(), io::Error> {
    let mut cfg = Config::default();
    if let Some(dist) = cli.minimum_relative_distance {
        cfg = cfg
            .with_min_rel_dist(dist)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{e:?}")))?;
    }
    if cli.low_accuracy_mode {
        cfg = cfg.disable_high_accuracy_mode();
    }
    if cli.preload {
        cfg = cfg.disable_lazy_load();
    }

    let det: LanguageDetector = if cli.spoken_language_only {
        cfg.build_from_all_spoken_languages()
    } else {
        cfg.build_from_all_languages()
    };
    print_detected_from_stdin(&det, cli.max_input_sample_bytes, cli.max_output_languages)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    sub(cli).map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
