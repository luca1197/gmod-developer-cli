use std::{path::Path, fs::{create_dir_all, write}, vec};
use clap::Subcommand;
use paris::{success, error, info};
use crate::{library, templates};

#[derive(Subcommand)]
pub enum Actions {
	Create {
		#[arg(value_parser = validate_directory_name)]
		directory_name: String
	}
}

fn validate_directory_name(input: &str) -> Result<String, String> {
	return library::validation::validate_input_dirname("./entities", input, false);
}

pub fn create(directory_name: String) {

	info!("<on-cyan><black> Cancel using CTRL + C. </>");

	// Check for addon.json
	if !Path::new("./addon.json").is_file() {
		error!("Failed to find addon.json! Are you inside an addon directory?");
		return;
	}

	// Check for existing entity
	if Path::new(&format!("./lua/entities/{}", &directory_name)).is_dir() {
		let input_override = library::inquire::confirm_no("An entity with this name already exists in this addon! Should potentially existing files be overwritten?");
		if !input_override {
			info!("<on-red> Cancelled. </>");
			return;
		}
	}

	// Pretty name
	let input_pretty_name = library::inquire::text_required("Pretty name for the entity:");

	// Category
	let input_category = library::inquire::text_required("Entity category:");

	// Author
	let input_author = library::inquire::text_required("Entity author:");

	// Spawnable
	let input_spawnable = library::inquire::confirm_yes("Should the entity be spawnable via the spawn menu?");

	// Type
	let input_type_options = vec!["Basic physics entity", "NPC"];
	let input_type = library::inquire::selector_index("Select an entity type", &input_type_options);

	// Fill entity templates
	let (mut file_cl, mut file_sv, mut file_sh) = (String::new(), String::new(), String::new());

	match input_type {
		0 => {

			// Model
			let input_model = library::inquire::text_optional("Entity model path:", "models/hunter/blocks/cube025x025x025.mdl");

			// Fill templates
			file_cl = templates::entity::ENTITY_BASIC_CL.to_string();

			file_sv = templates::entity::ENTITY_BASIC_SV
				.replace("%MODEL%", &input_model)
				.to_string();

			file_sh = templates::entity::ENTITY_BASIC_SH
				.replace("%CATEGORY%", &input_category)
				.replace("%SPAWNABLE%", &input_spawnable.to_string())
				.replace("%PRINTNAME%", &input_pretty_name)
				.replace("%AUTHOR%", &input_author)
				.to_string();

		}
		1 => {

			// Model
			let input_model = library::inquire::text_optional("Entity model path:", "models/gman.mdl");

			// Fill templates
			file_cl = templates::entity::ENTITY_NPC_CL.to_string();

			file_sv = templates::entity::ENTITY_NPC_SV
				.replace("%MODEL%", &input_model)
				.to_string();

			file_sh = templates::entity::ENTITY_NPC_SH
				.replace("%CATEGORY%", &input_category)
				.replace("%SPAWNABLE%", &input_spawnable.to_string())
				.replace("%PRINTNAME%", &input_pretty_name)
				.replace("%AUTHOR%", &input_author)
				.to_string();

		}
		_ => {
			error!("Invalid entity type!");
			return;
		}
	}

	// Create entity directory
	let create_dir_res = create_dir_all(format!("./lua/entities/{}", &directory_name));
	if create_dir_res.is_err() {
		error!("Failed to create entity directory: {}", create_dir_res.unwrap_err().to_string());
		return;
	}

	// Write entity files
	let (create_cl_res, create_sv_res, create_sh_res) = (
		write(format!("./lua/entities/{}/cl_init.lua", &directory_name), &file_cl),
		write(format!("./lua/entities/{}/init.lua", &directory_name), &file_sv),
		write(format!("./lua/entities/{}/shared.lua", &directory_name), &file_sh),
	);

	if create_cl_res.is_err() {
		error!("Failed to create cl_init.lua: {}", create_cl_res.unwrap_err().to_string());
		return;
	}

	if create_sv_res.is_err() {
		error!("Failed to create init.lua: {}", create_sv_res.unwrap_err().to_string());
		return;
	}

	if create_sh_res.is_err() {
		error!("Failed to create shared.lua: {}", create_sh_res.unwrap_err().to_string());
		return;
	}

	success!("Created entity <magenta>{}</>!", &input_pretty_name);

}