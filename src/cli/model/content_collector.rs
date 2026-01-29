use std::path::PathBuf;
use paris::{error, info, success, warn};
use crate::library::content::{
	SourceMaterialData,
	build_source_files_map, collect_source_paths, create_game_filesystem,
	locate_gmod_install, collect_model_materials, read_material_data,
	remove_game_content, log_missing_files, copy_files_to_output,
	print_content_summary,
};

/// Collects all content (materials, textures) used by a model file
pub fn collect_content(model: &PathBuf, source_path_strings: Vec<String>, output_path: &PathBuf) {
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

	// Create game filesystem
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

	// Get model path as string for processing
	let model_path_str = match model.to_str() {
		Some(s) => s,
		None => {
			error!("Failed to convert model path to string");
			return;
		}
	};

	// Collect materials used by the model
	info!("Collecting materials used by model \"<green>{}</>\"...", model.display());
	let (mut used_materials, mut missing_materials) = collect_model_materials(model_path_str, &source_files, &game_fs_open);

	// Check game files for missing materials
	let missing_mats_len = missing_materials.len();
	if missing_mats_len > 0 {
		info!("Looking for <red>{}</> currently missing materials in game files...", missing_mats_len);
		let found = remove_game_content(&mut missing_materials, &game_fs_open);
		info!("Found <green>{}</>/<red>{}</> materials in game files", found, missing_mats_len);
	}

	if missing_materials.is_empty() {
		success!("<green>No materials missing in source files!</>");
	} else {
		log_missing_files("materials", &missing_materials);
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

	// Add any additional materials found during texture collection (patch sources)
	used_materials.extend(material_data.used_materials);
	missing_materials.extend(material_data.missing_materials);

	// Try to find missing materials in game files again if there are more missing materials than in the previous check
	if missing_materials.len() > missing_mats_len {
		let found = remove_game_content(&mut missing_materials, &game_fs_open);
		if found > 0 {
			info!("Found <green>{}</>/<red>{}</> more missing materials in game files", found, missing_materials.len());
		}
	}

	if !missing_materials.is_empty() && missing_materials.len() != missing_mats_len {
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
		None, // No models to report for single model collection
		(&material_data.used_textures, &material_data.missing_textures),
	);

	// Copy all content to output directory
	info!("");
	info!("<cyan>Copying content to output directory \"{}\"...</>", output_path.display());

	info!("Copying <cyan>{}</> materials...", used_materials.len());
	copy_files_to_output(&used_materials, output_path, None);

	info!("Copying <cyan>{}</> textures...", material_data.used_textures.len());
	copy_files_to_output(&material_data.used_textures, output_path, None);

	success!("Done!");
}
