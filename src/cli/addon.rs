use std::{path::Path, fs::{create_dir, write}};
use clap::Subcommand;
use inquire::{Text, required, MultiSelect, validator::Validation, Select, list_option::ListOption};
use paris::{success, error, info};
use regex::Regex;
use itertools::Itertools;

use crate::templates;

#[derive(Subcommand)]
pub enum Actions {
	Init {
		#[arg(value_parser = validate_target_directory)]
		target_directory: String
	}
}

fn validate_target_directory(input: &str) -> Result<String, String> {

	let dir: String = input.parse().unwrap();

	let regex = Regex::new(r"[^\w\d_-]").unwrap();

	if regex.is_match(&dir) {
		return Err(format!("target_directory should only contain letters, numbers, dashes and underscores!"));
	}

	if Path::new(format!("./{dir}").as_str()).exists() {
		return Err(format!("Directory with specified target_directory already exists!"))
	}

	return Ok(dir);

}

pub fn init(target_directory: String) {

	info!("<on-cyan><black> Cancel using CTRL + C. </>");

	let input_pretty_name = Text::new("Pretty name for the addon:")
		.with_validator(required!("This field is required!"))
		.prompt()
		.unwrap();

	// Input type
	let input_type_options = vec!["ServerContent", "gamemode", "map", "weapon", "vehicle", "npc", "tool", "effects", "model", "entity"];
	let input_type = Select::new("Select addon type", input_type_options)
		.prompt()
		.unwrap();

	// // Input tags
	let input_tags_options = vec!["fun", "roleplay", "scenic", "movie", "realism", "cartoon", "water", "comic", "build"];
	// let mut input_tags_selected = vec![];
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
	let create_dir_res = create_dir(&target_directory);
	if create_dir_res.is_err() {
		error!("Failed to create addon directory: {}", create_dir_res.unwrap_err().to_string())
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