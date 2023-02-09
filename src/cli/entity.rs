use std::{path::Path, fs::create_dir};
use clap::Subcommand;
use paris::{success, error};
use regex::Regex;

#[derive(Subcommand)]
pub enum Actions {
	Create {
		#[arg(value_parser = validate_name)]
		name: String
	}
}

fn validate_name(input: &str) -> Result<String, String> {

	let name: String = input.parse().unwrap();

	let regex = Regex::new(r"[^\w\d_-]").unwrap();

	if regex.is_match(&name) {
		return Err(format!("name should only contain letters, numbers, dashes and underscores!"));
	}

	if Path::new(format!("./{name}").as_str()).exists() {
		return Err(format!("Directory with specified name already exists!"))
	}

	return Ok(name);

}

pub fn create(name: String) {

	

}