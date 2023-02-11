use std::{path::Path, fs::{write, create_dir_all}};
use clap::Subcommand;
use inquire::{MultiSelect, validator::Validation, list_option::ListOption};
use paris::{success, error, info};
use itertools::Itertools;

use crate::templates;
use crate::library;

#[derive(Subcommand)]
pub enum Actions {
	Init {
		#[arg(value_parser = validate_target_directory)]
		target_directory: String
	}
}

fn validate_target_directory(input: &str) -> Result<String, String> {
	return library::validation::validate_input_dirname(".", input, false);
}

pub fn init(target_directory: String) {

	info!("<on-cyan><black> Cancel using CTRL + C. </>");

	// Check for existing addon with name
	if Path::new(&format!("./{}", &target_directory)).is_dir() {
		let input_override = library::inquire::confirm_no("A directory with this name already exists in the current directory! Should potentially existing files be overwritten?");
		if !input_override {
			info!("<on-red> Cancelled. </>");
			return;
		}
	}

	// Check for existing addon in current directory
	if Path::new("./addon.json").is_file() {
		let input_existing = library::inquire::confirm_no("The current directory seems to be an addon already. Would you still like to create one?");
		if !input_existing {
			info!("<on-red> Cancelled. </>");
			return;
		}
	}

	// Input name
	let input_pretty_name = library::inquire::text_required("Pretty name for the addon:");

	// Input type
	let input_type_options = vec!["ServerContent", "gamemode", "map", "weapon", "vehicle", "npc", "tool", "effects", "model", "entity"];
	let input_type = library::inquire::selector("Select addon type", &input_type_options);

	// Input tags
	let input_tags_options = vec!["fun", "roleplay", "scenic", "movie", "realism", "cartoon", "water", "comic", "build"];
	let input_tags = MultiSelect::new("Select 1-2 addon tags:", input_tags_options)
		.with_validator(|list: &[ListOption<&&str>]| {
			if list.len() < 1 || list.len() > 2 {
				return Ok(Validation::Invalid(
					format!("{} tags selected, but 1-2 are required.", list.len()).into()
				))
			}

			return Ok(Validation::Valid);
		})
		.prompt()
		.unwrap();

	// Create addon directory
	let create_dir_res = create_dir_all(&target_directory);
	if create_dir_res.is_err() {
		error!("Failed to create addon directory: {}", create_dir_res.unwrap_err().to_string());
		return;
	}

	// Replace placeholders and write addon.json
	let addon_json_content = templates::addon::ADDON_JSON
		.replace("%NAME%", &input_pretty_name)
		.replace("%TYPE%", &input_type)
		.replace("%TAGS%", &input_tags.iter().map(|s| format!("\"{}\"", s)).join(", "));

	let create_json_res = write(format!("./{target_directory}/addon.json"), addon_json_content);
	if create_json_res.is_err() {
		error!("Failed to create addon.json: {}", create_json_res.unwrap_err().to_string());
		return;
	}

	success!("Successfully created addon <magenta>{input_pretty_name}</>!");

}