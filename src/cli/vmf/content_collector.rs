use std::{collections::HashMap, fs, path::PathBuf};
use paris::{error, info, success, warn};
use plumber_core::uncased::UncasedStr;
use crate::library::content::{
	SourceContentFile, SourceMaterialData,
	build_source_files_map, collect_source_paths, create_game_filesystem,
	locate_gmod_install, collect_model_materials, read_material_data,
	remove_game_content, log_missing_files, copy_files_to_output,
	print_content_summary, make_material_path, make_model_path,
};

/// Collects all content (materials, textures, models) used by a VMF map file
pub fn collect_content(vmf: &PathBuf, source_path_strings: Vec<String>, output_path: &PathBuf) {
	// Validate source paths
	let source_paths = collect_source_paths(source_path_strings);
	if source_paths.is_empty() {
		warn!("No source paths were provided");
	}

	// Locate Garry's Mod installation
	let (_, game_dir) = match locate_gmod_install() {
		Some(dirs) => dirs,
		None => {
			error!("Failed to locate Steam or Garry's Mod installation");
			return;
		}
	};
	info!("Found <cyan>Garry's Mod</> install in \"<green>{}</>\"", game_dir.display());

	// Build source files map
	let source_files = build_source_files_map(&source_paths);
	info!("Found <cyan>{}</> files in all source paths", source_files.len());

	// Read and parse VMF
	info!("Reading vmf \"<green>{}</>\"...", vmf.display());
	let vmf_content = match fs::read(vmf) {
		Ok(content) => content,
		Err(err) => {
			error!("Failed to read vmf file: {}", err);
			return;
		}
	};

	info!("Parsing vmf...");
	let vmf_parsed = match plumber_core::vmf::from_bytes(&vmf_content) {
		Ok(parsed) => parsed,
		Err(err) => {
			error!("Failed to parse vmf file: {}", err);
			return;
		}
	};

	let mut used_materials: HashMap<String, SourceContentFile> = HashMap::new();
	let mut missing_materials: HashMap<String, String> = HashMap::new();
	let mut used_models: HashMap<String, SourceContentFile> = HashMap::new();
	let mut missing_models: HashMap<String, String> = HashMap::new();

	// Collect materials from world brushes
	info!("Collecting materials used by world solids / brushes...");
	for solid in vmf_parsed.world.solids {
		for side in solid.sides {
			let material_path = make_material_path(&side.material.into_string());
			match source_files.get(&material_path) {
				Some(f) => { used_materials.insert(material_path, f.to_owned()); }
				None => { missing_materials.insert(material_path, format!("Used by world brush / solid {}", solid.id)); }
			}
		}
	}

	// Collect models and materials from entities
	info!("Collecting models and materials used by entities...");
	for ent in vmf_parsed.entities {
		// Entity brush materials
		for solid in ent.solids {
			for side in solid.sides {
				let material_path = make_material_path(&side.material.into_string());
				match source_files.get(&material_path) {
					Some(f) => { used_materials.insert(material_path, f.to_owned()); }
					None => {
						missing_materials.insert(material_path, format!(
							"Used by brush / solid {} in entity {} ({})", solid.id, ent.id, ent.class_name
						));
					}
				}
			}
		}

		// Collect entities with "material" property
		if let Some(material) = ent.properties.get(UncasedStr::new("material")) {
			let path = make_material_path(material);
			match source_files.get(&path) {
				Some(f) => { used_materials.insert(path, f.to_owned()); }
				None => { missing_materials.insert(path, format!("Entity {} ({}) \"material\" property", ent.id, ent.class_name)); }
			}
		}

		// Collect entities with "texture" property
		if let Some(texture) = ent.properties.get(UncasedStr::new("texture")) {
			let path = make_material_path(texture);
			match source_files.get(&path) {
				Some(f) => { used_materials.insert(path, f.to_owned()); }
				None => { missing_materials.insert(path, format!("Entity {} ({}) \"texture\" property", ent.id, ent.class_name)); }
			}
		}

		// Collect model if this entity has one set
		if let Some(model) = ent.properties.get(UncasedStr::new("model")) {
			// Special case: env_sprite entities use their "model" property as a material path to the sprite material
			if ent.class_name == "env_sprite" {
				let path = make_material_path(model);
				match source_files.get(&path) {
					Some(f) => { used_materials.insert(path, f.to_owned()); }
					None => { missing_materials.insert(path, format!("Sprite material for entity {} ({})", ent.id, ent.class_name)); }
				}
			} else {
				let path = make_model_path(model);
				match source_files.get(&path) {
					Some(f) => { used_models.insert(path, f.to_owned()); }
					None => { missing_models.insert(path, format!("Entity {} ({})", ent.id, ent.class_name)); }
				}
			}
		}
	}

	// Create game filesystem for model material collection
	let game_fs = match create_game_filesystem(&game_dir) {
		Ok(fs) => fs,
		Err(err) => {
			error!("{}", err);
			return;
		}
	};

	let game_fs_open = match game_fs.open() {
		Ok(fs) => fs,
		Err(err) => {
			error!("Failed to open game file system: {}", err);
			return;
		}
	};

	// Collect materials used by models
	info!("Collecting materials used by <cyan>{}</> collected models...", used_models.len());
	for (_, content_file) in &used_models {
		let (model_used, model_missing) = collect_model_materials(&content_file.full_path, &source_files, &game_fs_open);
		used_materials.extend(model_used);
		missing_materials.extend(model_missing);
	}

	// Find materials and models included in the game and remove them from missing_materials / missing_models
	let (missing_mats_len, missing_models_len) = (missing_materials.len(), missing_models.len());
	if missing_mats_len > 0 || missing_models_len > 0 {
		info!("Looking for <red>{}</> currently missing materials and <red>{}</> models in game files...", missing_mats_len, missing_models_len);
		let found_mats = remove_game_content(&mut missing_materials, &game_fs_open);
		let found_models = remove_game_content(&mut missing_models, &game_fs_open);
		info!("Found <green>{}</>/<red>{}</> materials and <green>{}</>/<red>{}</> models in game files",
			found_mats, missing_mats_len, found_models, missing_models_len);
	}

	if missing_models.is_empty() {
		success!("<green>No models missing in source files!</>");
	} else {
		log_missing_files("models", &missing_models);
	}

	// Collect textures used by used_materials materials
	info!("Collecting textures used by <cyan>{}</> materials...", used_materials.len());
	let mut material_data = SourceMaterialData::new();
	for (_, source_file) in &used_materials {
		match read_material_data(&source_file.full_path, &source_files, &game_fs_open) {
			Ok(data) => material_data.extend(data),
			Err(err) => warn!("Failed to read material data of \"{}\": {}", source_file.full_path, err),
		}
	}

	// Add materials that were now found by read_material_data (e.g. patch material sources)
	used_materials.extend(material_data.used_materials);
	missing_materials.extend(material_data.missing_materials);

	// Try to find missing materials in game files again if there are more missing materials than in the previous check
	if missing_materials.len() > missing_mats_len {
		let found = remove_game_content(&mut missing_materials, &game_fs_open);
		if found > 0 {
			info!("Found <green>{}</>/<red>{}</> more missing materials in game files", found, missing_materials.len());
		}
	}

	if missing_materials.is_empty() {
		success!("<green>No materials missing in source files!</>");
	} else {
		log_missing_files("materials", &missing_materials);
	}

	// Find textures included in the game and remove them from missing_textures
	let missing_tex_len = material_data.missing_textures.len();
	if missing_tex_len > 0 {
		info!("Looking for <red>{}</> currently missing textures in game files...", missing_tex_len);
		let found = remove_game_content(&mut material_data.missing_textures, &game_fs_open);
		info!("Found <green>{}</>/<red>{}</> missing textures in game files", found, missing_tex_len);
	}

	if material_data.missing_textures.is_empty() {
		success!("<green>No textures missing in source files!</>");
	} else {
		log_missing_files("textures", &material_data.missing_textures);
	}

	// Content summary
	print_content_summary(
		source_files.len(),
		(&used_materials, &missing_materials),
		Some((&used_models, &missing_models)),
		(&material_data.used_textures, &material_data.missing_textures),
	);

	// Copy collected content to output directory
	info!("");
	info!("<cyan>Copying content to output directory \"{}\"...</>", output_path.display());

	info!("Copying <cyan>{}</> materials...", used_materials.len());
	copy_files_to_output(&used_materials, output_path, None);

	info!("Copying <cyan>{}</> textures...", material_data.used_textures.len());
	copy_files_to_output(&material_data.used_textures, output_path, None);

	info!("Copying <cyan>{}</> models...", used_models.len());
	copy_files_to_output(&used_models, output_path, Some(&["dx90.vtx", "phy", "vvd"]));

	success!("Done!");
}
