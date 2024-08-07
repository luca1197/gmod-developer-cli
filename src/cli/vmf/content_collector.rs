use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use crate::library::validation::validate_path_is_directory;
use paris::{error, info, success, warn};
use plumber_core::{fs::{FileSystem, OpenFileSystem}, steam::App, uncased::UncasedStr};
use walkdir::WalkDir;
use simple_error::{bail, SimpleError};

#[derive(Debug, Clone)]
pub struct SourceContentFile {
	full_path: String,
	local_path: String,
}

pub fn collect_content(vmf: &PathBuf, source_path_strings: Vec<String>, output_path: &PathBuf) {

	//
	// Validate source_paths
	//
	let mut source_paths: Vec<PathBuf> = vec!();
	for source_path_string in source_path_strings {
		match validate_path_is_directory(&source_path_string) {
			Ok(path) => source_paths.push(path),
			Err(err) => warn!("Skipping provided source path \"{}\": {}", source_path_string, err)
		}
	}

	if source_paths.len() == 0 {
		warn!("No source paths were provided");
	}

	//
	// Locate game install
	//
	let mut steam_dir = match steamlocate::SteamDir::locate() {
		Some(dir) => dir,
		None => {
			error!("Failed to locate Steam installation");
			return;
		}
	};
	
	const GMOD_APP_ID: u32 = 4_000;
	let game_dir = match steam_dir.app(&GMOD_APP_ID) {
		Some(app) => &app.path,
		None => {
			error!("Failed to locate Garry's Mod installation");
			return;
		}
	};

	info!("Found <cyan>Garry's Mod</> install in \"<green>{}</>\"", game_dir.display());

	//
	// Create a hashmap with all source path files (Key is lowercased path local to source path, this is the "standardized" path used throughout the command)
	//
	let mut source_files: HashMap<String, SourceContentFile> = HashMap::new();
	
	for source_path in source_paths {

		info!("Reading source path \"<green>{}</>\"...", &source_path.display());

		for entry in WalkDir::new(&source_path).follow_links(true) {

			// Get entry
			let entry = match entry {
				Ok(entry) => entry,
				Err(err) => {
					error!("Failed to read entry in source path \"{}\": {}", &source_path.display(), err.to_string());
					continue;
				}
			};

			// Skip directories
			if entry.file_type().is_dir() {
				continue;
			}

			// Get full path
			let entry_path = entry.path();
			let entry_path_string = match entry_path.to_str() {
				Some(path) => path.to_string(),
				None => {
					error!("Failed to get full path to entry \"{}\" in source path \"{}\"", entry_path.display(), &source_path.display());
					continue;
				}
			};

			// Get local / relative path
			let local_path = match entry_path.strip_prefix(&source_path) {
				Ok(path) => path,
				Err(err) => {
					error!("Failed to make local path for entry \"{}\" in source path \"{}\": {}", entry_path.display(), &source_path.display(), err.to_string());
					continue;
				}
			};

			let local_path_string = match local_path.to_str() {
				Some(path) => path.to_string(),
				None => {
					error!("Failed to get local path to entry \"{}\" in source path \"{}\"", entry_path.display(), &source_path.display());
					continue;
				}
			};

			// Skip duplicates
			let hashmap_key = local_path_string.replace("/", "\\").to_lowercase();
			if source_files.contains_key(&hashmap_key) {
				continue;
			}

			// Insert into source_files
			source_files.insert(hashmap_key, SourceContentFile {
				full_path: entry_path_string,
				local_path: local_path_string,
			});

		}

	}

	info!("Found <cyan>{}</> files in all source paths", source_files.len());

	//
	// Read vmf
	//
	info!("Reading vmf \"<green>{}</>\"...", vmf.display());
	let vmf_content = match fs::read(vmf) {
		Ok(content) => content,
		Err(err) => {
			error!("Failed to read vmf file in \"{}\": {}", vmf.display(), err.to_string());
			return;
		}
	};

	//
	// Parse vmf
	//
	info!("Parsing vmf...");
	let vmf_parsed = match plumber_core::vmf::from_bytes(&vmf_content) {
		Ok(parsed) => parsed,
		Err(err) => {
			error!("Failed to parse vmf file in \"{}\": {}", vmf.display(), err.to_string());
			return;
		}
	};

	let mut used_materials: HashMap<String, SourceContentFile> = HashMap::new();
	let mut missing_materials: HashMap<String, String> = HashMap::new();
	let mut used_models: HashMap<String, SourceContentFile> = HashMap::new();
	let mut missing_models: HashMap<String, String> = HashMap::new();

	//
	// Collect materials from all world solids / brushes
	//
	info!("Collecting materials used by world solids / brushes...");
	for solid in vmf_parsed.world.solids {

		for side in solid.sides {

			let side_material_source_path = format!(
				"materials\\{}.vmt",
				&side.material
					.into_string()
					.replace("/", "\\")
					.to_lowercase()
			);

			// Check if source file exists and add it to used_materials or missing_materials accordingly
			match source_files.get(&side_material_source_path) {
				Some(source_file) => {
					// Add to used_materials
					used_materials.insert(side_material_source_path, source_file.to_owned());
				},
				None => {
					// Add to missing_materials
					missing_materials.insert(side_material_source_path, format!("Used by world brush / solid {}", solid.id));
				}
			}

		}

	}

	//
	// Collect models and materials from entities
	//
	info!("Collecting models and materials used by entities...");
	for ent in vmf_parsed.entities {

		// Collect materials from all entity solids / brushes
		for solid in ent.solids {

			for side in solid.sides {

				// Construct path local to source file paths (to_lowercase, replace / with \, add materials\ and add .vmt, everything to match source_files keys)
				let side_material_source_path = format!(
					"materials\\{}.vmt",
					&side.material
						.into_string()
						.replace("/", "\\")
						.to_lowercase()
				);

				// Check if source file exists and add it to used_materials or missing_materials accordingly
				match source_files.get(&side_material_source_path) {
					Some(source_file) => {
						// Add to used_materials
						used_materials.insert(side_material_source_path, source_file.to_owned());
					},
					None => {
						// Add to missing_materials
						missing_materials.insert(side_material_source_path, format!("Used by brush / solid {} in entity {} with class {}", solid.id, ent.id, ent.class_name));
					}
				}

			}

		}

		// Collect entities with "material" property
		match ent.properties.get(UncasedStr::new("material")) {
			Some(material) => {

				let mut material_source_path = format!("materials\\{}", material)
					.replace("/", "\\")
					.to_lowercase();

				if !material_source_path.ends_with(".vmt") {
					material_source_path.push_str(".vmt");
				}

				match source_files.get(&material_source_path) {
					Some(source_file) => {
						used_materials.insert(material_source_path, source_file.to_owned());
					},
					None => {
						missing_materials.insert(material_source_path, format!("Used by entity {} with class {} in \"material\" property", ent.id, ent.class_name));
					}
				}

			},
			None => {}
		}

		// Collect entities with "texture" property
		match ent.properties.get(UncasedStr::new("texture")) {
			Some(material) => {

				let mut material_source_path = format!("materials\\{}", material)
					.replace("/", "\\")
					.to_lowercase();

				if !material_source_path.ends_with(".vmt") {
					material_source_path.push_str(".vmt");
				}

				match source_files.get(&material_source_path) {
					Some(source_file) => {
						used_materials.insert(material_source_path, source_file.to_owned());
					},
					None => {
						missing_materials.insert(material_source_path, format!("Used by entity {} with class {} in \"texture\" property", ent.id, ent.class_name));
					}
				}

			},
			None => {}
		}

		// Collect model if this entity has one set
		match ent.properties.get(UncasedStr::new("model")) {
			Some(model) => {

				// Special case: env_sprite entities use their "model" property as a material path to the sprite material
				if ent.class_name == "env_sprite" {

					let mut source_file_path = format!("materials\\{}", model)
						.replace("/", "\\")
						.to_lowercase();

					if !source_file_path.ends_with(".vmt") {
						source_file_path.push_str(".vmt");
					}

					// Check if source file exists and add it to used_materials or missing_materials accordingly
					match source_files.get(&source_file_path) {
						Some(source_file) => {
							used_materials.insert(source_file_path, source_file.to_owned());
						},
						None => {
							missing_materials.insert(source_file_path, format!("Used as sprite material by entity {} with class {}", ent.id, ent.class_name));
						}
					};

				} else {

					// Construct path local to source file paths (see side_material_local_path)
					let model_source_path = model
						.to_owned()
						.replace("/", "\\")
						.to_lowercase();

					match source_files.get(&model_source_path) {
						Some(source_file) => {
							// Add to used_models
							used_models.insert(model_source_path, source_file.to_owned());
						},
						None => {
							// Add to missing_models
							missing_models.insert(model_source_path, format!("Used by entity {} with class {}", ent.id, ent.class_name));
						}
					}

				}

				

			},
			None => {}
		}

	}

	//
	// Collect materials used by used_models models
	//
	info!("Collecting materials used by <cyan>{}</> collected models...", used_models.len());
	let game_app = App { app_id: GMOD_APP_ID, name: String::from("Garry's Mod"), install_dir: game_dir.to_owned() };
	let game_fs = match FileSystem::from_app(&game_app) {
		Ok(fs) => fs,
		Err(err) => {
			error!("Failed to create game file system: {}", err.to_string());
			return;
		}
	};

	let game_fs_open = match game_fs.open() {
		Ok(fs) => fs,
		Err(err) => {
			error!("Failed to open game file system: {}", err.to_string());
			return;
		}
	};

	// Iterate models and add their materials to used_materials
	for (_, content_file) in &used_models {

		// Only .mdl file (no vtx / phy / vvd)
		if !content_file.full_path.ends_with(".mdl") {
			continue;
		}

		// Read model
		let model = match plumber_core::mdl::Model::read(Path::new(&content_file.full_path), &game_fs_open) {
			Ok(model) => model,
			Err(err) => {
				warn!("Failed to read model \"{}\": {}", content_file.full_path, err.to_string());
				continue;
			}
		};

		// Verify model
		let model_verified = match model.verify() {
			Ok(model) => model,
			Err(err) => {
				warn!("Failed to verify model \"{}\": {}", content_file.full_path, err.to_string());
				continue;
			}
		};

		// Get materials
		let materials = match model_verified.mdl_header.iter_textures() {
			Ok(materials) => materials,
			Err(err) => {
				warn!("Failed to get materials of model \"{}\": {}", content_file.full_path, err.to_string());
				continue;
			}
		};

		// Get cdmaterials / texture_paths
		let cdmaterials_list = match model_verified.mdl_header.texture_paths() {
			Ok(texture_paths) => texture_paths,
			Err(err) => {
				warn!("Failed to get texture paths / cdmaterials of model \"{}\": {}", content_file.full_path, err.to_string());
				continue;
			}
		};

		// Add materials to used_materials / missing_materials
		for material in materials {

			// Get material name
			let material_name = match material.name() {
				Ok(name) => name,
				Err(err) => {
					warn!("Failed to get name of a material of model \"{}\": {}", content_file.full_path, err.to_string());
					continue;
				}
			};

			// Try to find material in source_files in any of its cdmaterials paths
			for cdmaterials in &cdmaterials_list {

				let source_file_path = format!("materials\\{}{}.vmt", cdmaterials, material_name)
					.replace("/", "\\")
					.to_lowercase();
			
				// Add material to used_materials or missing_materials depending on whether it exists in source_files
				match source_files.get(&source_file_path) {
					Some(source_file) => {
						// Add to used_materials
						used_materials.insert(source_file_path, source_file.to_owned());
					},
					None => {
						// Add to missing_materials
						missing_materials.insert(source_file_path, format!("Used by model \"{}\"", content_file.full_path));
					}
				}

				//println!("{}: {} -> {} ? {}", content_file.local_path, texture_path, material_name, source_files.contains_key(&source_file_path));

			}

		}

	}

	//
	// Find materials and models included in the game and remove them from missing_materials / missing_models
	//
	let (missing_materials_len, missing_models_len) = (missing_materials.len(), missing_models.len());
	if missing_materials_len > 0 || missing_models_len > 0 {

		info!("Looking for <red>{}</> currently missing materials and <red>{}</> models in game files...", missing_materials_len, missing_models_len);
		
		let found_missing_materials = hashmap_remove_game_content(&mut missing_materials, &game_fs_open);
		let found_mssing_models = hashmap_remove_game_content(&mut missing_models, &game_fs_open);

		info!("Found <green>{}</>/<red>{}</> currently missing materials and <green>{}</>/<red>{}</> models in game files", found_missing_materials, missing_materials_len, found_mssing_models, missing_models_len);

	}

	// Log missing models
	if missing_models.len() > 0 {
		log_missing_files_hashmap("models", &missing_models);
	} else {
		success!("<green>No models missing in source files!</>");
	}

	//
	// Collect textures used by used_materials materials
	//
	info!("Collecting textures used by <cyan>{}</> materials...", used_materials.len());
	let mut used_materials_data = SourceMaterialData::new();
	for (_, source_file) in &used_materials {

		match read_material_data(&source_file.full_path, &source_files, &game_fs_open) {
			Ok(data) => used_materials_data.extend(data),
			Err(err) => warn!("Failed to read material data of \"{}\": {}", source_file.full_path, err.to_string()),
		}

	}

	// Add materials that were now found by read_material_data (e.g. patch material sources)
	used_materials.extend(used_materials_data.used_materials);
	missing_materials.extend(used_materials_data.missing_materials);

	// Try to find missing materials in game files again if there are more missing materials than in the previous check
	if missing_materials.len() > missing_materials_len {
		let found_missing_materials = hashmap_remove_game_content(&mut missing_materials, &game_fs_open);
		if found_missing_materials > 0 {
			info!("Found <green>{}</>/<red>{}</> more currently missing materials in game files", found_missing_materials, missing_materials_len);
		}
	}

	// Log missing materials
	if missing_materials.len() > 0 {
		log_missing_files_hashmap("materials", &missing_materials);
	} else {
		success!("<green>No materials missing in source files!</>");
	}


	// Find textures included in the game and remove them from missing_textures
	let missing_textures_len = used_materials_data.missing_textures.len();
	if missing_textures_len > 0 {

		info!("Looking for <red>{}</> currently missing textures in game files...", &missing_textures_len);

		let found_missing_textures = hashmap_remove_game_content(&mut used_materials_data.missing_textures, &game_fs_open);

		info!("Found <green>{}</>/<red>{}</> currently missing textures in game files", found_missing_textures, &missing_textures_len);

	}

	// Log missing textures
	if used_materials_data.missing_textures.len() > 0 {
		log_missing_files_hashmap("textures", &used_materials_data.missing_textures);
	} else {
		success!("<green>No textures missing in source files!</>");
	}

	//
	// Content summary
	//
	info!("<magenta>CONTENT SUMMARY:</>");
	info!("\t<magenta>↳</> Source files: Total <cyan>{}</>", &source_files.len());
	info!("\t<magenta>↳</> Materials: Found <green>{}</>; Missing <red>{}</>", &used_materials.len(), &missing_materials.len());
	info!("\t<magenta>↳</> Models: Found <green>{}</>; Missing <red>{}</>", &used_models.len(), &missing_models.len());
	info!("\t<magenta>↳</> Textures: Found <green>{}</>; Missing <red>{}</>", &used_materials_data.used_textures.len(), &used_materials_data.missing_textures.len());

	//
	// Copy all content to output directory
	//
	info!("");
	info!("<cyan>Copying content to output directory \"{}\"...</>", &output_path.display());

	// Copy materials
	info!("Copying <cyan>{}</> materials...", &used_materials.len());
	copy_files_to_output(&used_materials, &output_path, None);

	// Copy textures
	info!("Copying <cyan>{}</> textures...", &used_materials_data.used_textures.len());
	copy_files_to_output(&used_materials_data.used_textures, &output_path, None);

	// Copy models
	info!("Copying <cyan>{}</> models...", &used_models.len());
	copy_files_to_output(&used_models, &output_path, Some(&vec!["dx90.vtx", "phy", "vvd"]));

	success!("Done!");
	
}

#[derive(Debug)]
pub struct SourceMaterialData {
	pub used_materials: HashMap<String, SourceContentFile>,
	pub missing_materials: HashMap<String, String>,
	pub used_textures: HashMap<String, SourceContentFile>,
	pub missing_textures: HashMap<String, String>,
}

impl SourceMaterialData {
	pub fn new() -> Self {
		Self {
			used_materials: HashMap::new(),
			missing_materials: HashMap::new(),
			used_textures: HashMap::new(),
			missing_textures: HashMap::new(),
		}
	}
	pub fn extend(&mut self, other: Self) {
		self.used_materials.extend(other.used_materials);
		self.missing_materials.extend(other.missing_materials);
		self.used_textures.extend(other.used_textures);
		self.missing_textures.extend(other.missing_textures);
	}
}

pub fn read_material_data(full_path: &str, source_files: &HashMap<String, SourceContentFile>, open_fs: &plumber_core::fs::OpenFileSystem)
	-> Result<SourceMaterialData, SimpleError> 
{

	// Read material
	let material_file = match fs::read(full_path) {
		Ok(material_file) => material_file,
		Err(err) => {
			bail!("Failed to read material file \"{}\": {}", full_path, err.to_string());
		}
	};

	// Parse material
	let material_parsed = match plumber_core::vmt::from_bytes(&material_file) {
		Ok(material_parsed) => material_parsed,
		Err(err) => {
			bail!("Failed to parse material file \"{}\": {}", full_path, err.to_string());
		}
	};

	return get_material_data(material_parsed, source_files, open_fs, full_path);

}

pub fn get_material_data(vmt: plumber_core::vmt::Vmt, source_files: &HashMap<String, SourceContentFile>, open_fs: &plumber_core::fs::OpenFileSystem, logging_reference_material: &str)
	-> Result<SourceMaterialData, SimpleError>
{

	let mut collection = SourceMaterialData::new();

	// Into shader
	let material_shader: plumber_core::vmt::Shader = match vmt.resolve_shader_os(open_fs, |patch_path_local| {
		
		//
		// SPECIAL CASE: Patch material
		// Try to find the material this patch material is patching
		//

		let mut patch_source_file_path = patch_path_local
			.replace("/", "\\")
			.to_lowercase();

		if !patch_source_file_path.ends_with(".vmt") {
			patch_source_file_path.push_str(".vmt");
		}

		// Get patched material source file
		match source_files.get(&patch_source_file_path) {
			Some(source_file) => {

				// Add patch material *itself* to the collection
				collection.used_materials.insert(patch_source_file_path, source_file.to_owned());

				// Read patch material and add its data to the collection
				// This is necessary since plumber_core will actually apply the patch, while the engine still needs the material to patch it itself
				let patch_source_data = read_material_data(&source_file.full_path, source_files, open_fs)
					.map_err(|err| plumber_core::vmt::ShaderResolveError::Io { path: String::from(&source_file.full_path), error: format!("[Patch material] {}", err.to_string()) })?;

				collection.extend(patch_source_data);

				return Ok(PathBuf::from(&source_file.full_path));

			},
			None => {
				return Err(plumber_core::vmt::ShaderResolveError::Io { path: String::from(patch_path_local), error: String::from("Did not find source file for material to be patched") });
			}
		}

		//
		// END SPECIAL CASE: Patch material
		//

	}) {
		Ok(material_shader) => material_shader,
		Err(err) => {
			bail!("Failed to parse shader: {}", err.to_string());
		}
	};

	// Iterate material parameters and add their value to used_textures / missing_textures if it is a texture parameter
	for (param_key, param_value) in material_shader.parameters {

		//
		// SPECIAL CASE: $bottommaterial
		// This is a material parameter that takes a material as input, so we need to add it to the material collection
		//
		if &param_key == UncasedStr::new("$bottommaterial") {

			let mut source_file_path = format!("materials\\{}", param_value)
				.replace("/", "\\")
				.to_lowercase();

			if !source_file_path.ends_with(".vmt") {
				source_file_path.push_str(".vmt");
			}

			match source_files.get(&source_file_path) {
				Some(source_file) => {
					collection.used_materials.insert(source_file_path, source_file.to_owned());
				},
				None => {
					collection.missing_materials.insert(source_file_path, format!("Used by material \"{}\" in material parameter \"$bottommaterial\"", logging_reference_material));
				}
			};

			continue;

		}
		//	
		// END SPECIAL CASE: $bottommaterial
		//

		if !VMT_TEXTURE_PARAMETERS.contains(&param_key.to_string().to_lowercase().as_str()) {
			continue;
		}

		let mut source_file_path = format!("materials\\{}", param_value)
			.replace("/", "\\")
			.to_lowercase();

		if !source_file_path.ends_with(".vtf") {
			source_file_path.push_str(".vtf");
		}

		// Special case: $envmap can be set to "env_cubemap" which will be replaced dynamically by a built cubemap by the engine
		if source_file_path == VMT_ENVMAP_DEFAULT_SOURCE_PATH {
			continue;
		}

		// Check if source file exists and add it to used_textures or missing_textures accordingly
		match source_files.get(&source_file_path) {
			Some(source_file) => {
				collection.used_textures.insert(source_file_path, source_file.to_owned());
			},
			None => {
				collection.missing_textures.insert(source_file_path, format!("Used by material \"{}\" in texture parameter {}", logging_reference_material, param_key));
			}
		};

	}

	return Ok(collection);

}

pub fn hashmap_remove_game_content(map: &mut HashMap<String, String>, fs: &OpenFileSystem) -> i32 {

	let mut removed_count = 0;

	map.retain(|file_local_path, _| {

		// plumber_core only allows "/" slashes and lowercase characters
		let game_file_location = file_local_path.replace("\\", "/").to_lowercase();

		// We need to use plumber_core::vpk::Path because only this way plumber_core looks in the *game* file system instead of the OS file system
		// It checks if a std library Path is provided or its custom one.
		let game_file_path = match plumber_core::vpk::Path::try_from_str(&game_file_location.as_str()) {
			Some(path) => path,
			None => {
				warn!("Failed to create game file path for \"{}\"", file_local_path);
				return true;
			}
		};

		// Try to open material in game file system
		// The path is all lowercase but that is working and explicitly allowed (and required above) by plumber_core
		match fs.open_file(game_file_path) {
			Ok(_) => {
				removed_count += 1;
				return false
			},
			Err(_) => {
				// warn!("Failed to open \"{}\" in game file system: {}", material, err.to_string());
				return true;
			}
		}

	});

	return removed_count;

}

pub fn log_missing_files_hashmap(name: &str, map: &HashMap<String, String>) {

	warn!("Missing <red>{}</> {} in source files:", map.len(), name);

	for (file_local_path, error_message) in map {

		warn!("\t<red>-</> {}", file_local_path);
		warn!("\t  ↳ {}", error_message);

	}

}

pub const VMT_TEXTURE_PARAMETERS: [&str; 19] = [
	"$basetexture",
	"$basetexture2",
	"$detail",
	"$detail1",
	"$detail2",
	"$bumpmap",
	"$bumpmap2",
	"$bumpmask",
	"$selfillummask",
	"$selfillumtexture",
	"$AmbientOcclTexture",
	"$lightmap",
	"$phongexponenttexture",
	"$phongwarptexture",
	"$envmap",
	"$envmapmask",
	"$tintmasktexture",
	"$blendmodulatetexture",
	"$normalmap",
];

pub const VMT_ENVMAP_DEFAULT_SOURCE_PATH: &str = "materials\\env_cubemap.vtf";

pub fn copy_files_to_output(source_files: &HashMap<String, SourceContentFile>, output_path: &PathBuf, copy_additional_extensions: Option<&Vec<&str>>) {

	for (_, source_file) in source_files {

		let output_file_path = output_path.join(&source_file.local_path);
		let output_file_dir_path = match output_file_path.parent() {
			Some(path) => path,
			None => {
				warn!("Failed to get parent directory of \"{}\"", output_file_path.display());
				continue
			}
		};

		match fs::create_dir_all(&output_file_dir_path) {
			Ok(_) => {

				let source_file_path = Path::new(&source_file.full_path);

				match fs::copy(&source_file_path, &output_file_path) {
					Ok(_) => {},
					Err(err) => warn!("Failed to copy \"{}\" to \"{}\": {}", source_file.full_path, output_file_path.display(), err.to_string())
				}

				if let Some(copy_additional_extensions) = copy_additional_extensions {
					for extension in copy_additional_extensions {
						let source_file_path_ext = source_file_path.with_extension(extension);
						let output_file_path_ext = output_file_path.with_extension(extension);
						match fs::copy(&source_file_path_ext, &output_file_path_ext) {
							Ok(_) => {},
							Err(err) => warn!("Failed to copy \"{}\" to \"{}\": {}", source_file_path_ext.display(), output_file_path_ext.display(), err.to_string())
						}
					}
				}

			},
			Err(err) => warn!("Failed to create directory \"{}\": {}", output_file_dir_path.display(), err.to_string())
		}

	}

}
