use std::{path::Path, fs::{create_dir_all, write}, vec};
use clap::Subcommand;
use paris::{success, error};
use crate::{library, templates};

#[derive(Subcommand)]
pub enum Actions {
	Create {
		#[arg(value_parser = validate_name)]
		name: String
	}
}

fn validate_name(input: &str) -> Result<String, String> {
	return library::validation::validate_input_dirname("./entities", input);
}

pub fn create(name: String) {

	// Check for addon.json
	if !Path::new("./addon.json").is_file() {
		error!("Failed to find addon.json! Are you inside an addon directory?");
		return;
	}

	// Pretty name
	let input_pretty_name = library::inquire::required_text("Pretty name for the entity:");

	// Category
	let input_category = library::inquire::required_text("Entity category:");

	// Author
	let input_author = library::inquire::required_text("Entity author:");

	// Spawnable
	let input_spawnable = library::inquire::confirm_yes("Should the entity be spawnable via the spawn menu?");

	// Type
	let input_type_options = vec!["Basic physics entity", "NPC"];
	let input_type = library::inquire::selector_index("Select an entity type", &input_type_options);

	// Create entity directory
	let create_dir_res = create_dir_all(format!("./lua/entities/{}", name));
	if create_dir_res.is_err() {
		error!("Failed to create entity directory: {}", create_dir_res.unwrap_err().to_string());
		return;
	}

	// Create entity files
	let (mut file_cl, mut file_sv, mut file_sh) = (String::new(), String::new(), String::new());

	match input_type {
		0 => {
			file_cl = templates::entity::ENTITY_BASIC_CL.to_string();
			file_sv = templates::entity::ENTITY_BASIC_SV.to_string();
			file_sh = templates::entity::ENTITY_BASIC_SH
				.replace("%CATEGORY%", &input_category)
				.replace("%SPAWNABLE%", &input_spawnable.to_string())
				.replace("%PRINTNAME%", &input_pretty_name)
				.replace("%AUTHOR%", &input_author)
				.to_string();
		}
		1 => {
			println!("npc")
		}
		_ => {
			error!("Invalid entity type!");
			return;
		}
	}

	// Write entity files
	let (create_cl_res, create_sv_res, create_sh_res) = (
		write(format!("./lua/entities/{}/cl_init.lua", &name), &file_cl),
		write(format!("./lua/entities/{}/init.lua", &name), &file_sv),
		write(format!("./lua/entities/{}/shared.lua", &name), &file_sh),
	);

}