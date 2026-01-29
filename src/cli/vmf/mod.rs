use std::path::PathBuf;
use clap::Subcommand;
use crate::library;

pub mod content_collector;
pub mod stats;

#[derive(Subcommand)]
pub enum Actions {
	CollectContent {
		#[arg(value_parser = validate_vmf_path)]
		vmf_path: PathBuf,
		#[arg(short, long, help = "Path to a directory which contains content the map potentially uses. The directory should contain subdirectories like `materials/` and `models/`. This option can be used multiple times.")]
		source_path: Vec<String>,
		#[arg(short, long, value_parser = validate_output_path, help="Path to a directory where all of the content the map uses will be copied to.")]
		output_path: PathBuf,
	},
	Stats {
		#[arg(value_parser = validate_vmf_path)]
		vmf_path: PathBuf
	}
}

fn validate_vmf_path(input: &str) -> Result<PathBuf, String> {
	return library::validation::validate_input_file_exists(input, "vmf");
}

fn validate_output_path(input: &str) -> Result<PathBuf, String> {
	return library::validation::validate_path_is_directory(input);
}
