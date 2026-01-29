use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use paris::{error, info, warn};
use plumber_core::{
	fs::{FileSystem, OpenFileSystem},
	steam::App,
	uncased::UncasedStr,
};
use simple_error::{bail, SimpleError};
use walkdir::WalkDir;
use crate::library::validation::validate_path_is_directory;

// Garry's Mod Steam App ID
pub const GMOD_APP_ID: u32 = 4_000;

/// Represents a content file found in source paths
#[derive(Debug, Clone)]
pub struct SourceContentFile {
	pub full_path: String,
	pub local_path: String,
}

/// Collected material data including textures and referenced materials
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

// VMT parameters that contain texture paths
pub const VMT_TEXTURE_PARAMETERS: [&str; 20] = [
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
	"$ambientoccltexture",
	"$lightmap",
	"$phongexponenttexture",
	"$phongwarptexture",
	"$envmap",
	"$envmapmask",
	"$tintmasktexture",
	"$blendmodulatetexture",
	"$normalmap",
	"$lightwarptexture",
];

// $envmap default value that should be skipped (engine-generated cubemap)
pub const VMT_ENVMAP_DEFAULT_SOURCE_PATH: &str = "materials\\env_cubemap.vtf";

/// Locates the Garry's Mod installation directory via Steam
pub fn locate_gmod_install() -> Option<(steamlocate::SteamDir, PathBuf)> {
	let mut steam_dir = steamlocate::SteamDir::locate()?;
	let game_dir = steam_dir.app(&GMOD_APP_ID)?.path.clone();
	return Some((steam_dir, game_dir));
}

/// Creates a game filesystem for Garry's Mod
pub fn create_game_filesystem(game_dir: &Path) -> Result<FileSystem, SimpleError> {
	let game_app = App {
		app_id: GMOD_APP_ID,
		name: String::from("Garry's Mod"),
		install_dir: game_dir.to_owned(),
	};
	return FileSystem::from_app(&game_app)
		.map_err(|err| SimpleError::new(format!("Failed to create game file system: {}", err)));
}

/// Validates and collects source paths into a list of PathBufs
pub fn collect_source_paths(source_path_strings: Vec<String>) -> Vec<PathBuf> {
	let mut source_paths: Vec<PathBuf> = vec![];
	for source_path_string in source_path_strings {
		match validate_path_is_directory(&source_path_string) {
			Ok(path) => source_paths.push(path),
			Err(err) => warn!("Skipping provided source path \"{}\": {}", source_path_string, err)
		}
	}
	return source_paths;
}

/// Builds a hashmap of all files in the source paths
/// Key is lowercased path with backslashes
/// This is the "standardized" path used throughout the content collection commands
pub fn build_source_files_map(source_paths: &[PathBuf]) -> HashMap<String, SourceContentFile> {
	let mut source_files: HashMap<String, SourceContentFile> = HashMap::new();

	for source_path in source_paths {
		info!("Reading source path \"<green>{}</>\"...", &source_path.display());

		for entry in WalkDir::new(source_path).follow_links(true) {
			let entry = match entry {
				Ok(entry) => entry,
				Err(err) => {
					error!("Failed to read entry in source path \"{}\": {}", &source_path.display(), err);
					continue;
				}
			};

			if entry.file_type().is_dir() {
				continue;
			}

			let entry_path = entry.path();
			let entry_path_string = match entry_path.to_str() {
				Some(path) => path.to_string(),
				None => {
					error!("Failed to get full path to entry \"{}\"", entry_path.display());
					continue;
				}
			};

			let local_path = match entry_path.strip_prefix(source_path) {
				Ok(path) => path,
				Err(err) => {
					error!("Failed to make local path for entry \"{}\": {}", entry_path.display(), err);
					continue;
				}
			};

			let local_path_string = match local_path.to_str() {
				Some(path) => path.to_string(),
				None => {
					error!("Failed to get local path to entry \"{}\"", entry_path.display());
					continue;
				}
			};

			// Standardize path format: lowercase with backslashes
			let hashmap_key = local_path_string.replace("/", "\\").to_lowercase();
			
			// Insert into hashmap if not already present
			if !source_files.contains_key(&hashmap_key) {
				source_files.insert(hashmap_key, SourceContentFile {
					full_path: entry_path_string,
					local_path: local_path_string,
				});
			}
		}
	}

	return source_files;
}

/// Creates a standardized material path from a raw material name
pub fn make_material_path(material_name: &str) -> String {
	let mut path = format!("materials\\{}", material_name)
		.replace("/", "\\")
		.to_lowercase();
	if !path.ends_with(".vmt") {
		path.push_str(".vmt");
	}
	return path;
}

/// Creates a standardized texture path from a raw texture name
pub fn make_texture_path(texture_name: &str) -> String {
	let mut path = format!("materials\\{}", texture_name)
		.replace("/", "\\")
		.to_lowercase();
	if !path.ends_with(".vtf") {
		path.push_str(".vtf");
	}
	return path;
}

/// Creates a standardized model path from a raw model name
pub fn make_model_path(model_name: &str) -> String {
	return model_name.replace("/", "\\").to_lowercase();
}

/// Collects materials used by a model file
pub fn collect_model_materials(
	model_path: &str,
	source_files: &HashMap<String, SourceContentFile>,
	game_fs: &OpenFileSystem,
) -> (HashMap<String, SourceContentFile>, HashMap<String, String>) {
	let mut used_materials: HashMap<String, SourceContentFile> = HashMap::new();
	let mut missing_materials: HashMap<String, String> = HashMap::new();

	// Only process .mdl files (no vtx / phy / vvd)
	if !model_path.ends_with(".mdl") {
		return (used_materials, missing_materials);
	}

	// Read model
	let model = match plumber_core::mdl::Model::read(Path::new(model_path), game_fs) {
		Ok(model) => model,
		Err(err) => {
			warn!("Failed to read model \"{}\": {}", model_path, err);
			return (used_materials, missing_materials);
		}
	};

	// Verify model
	let verified = match model.verify() {
		Ok(v) => v,
		Err(err) => {
			warn!("Failed to verify model \"{}\": {}", model_path, err);
			return (used_materials, missing_materials);
		}
	};

	// Get materials
	let materials = match verified.mdl_header.iter_textures() {
		Ok(m) => m,
		Err(err) => {
			warn!("Failed to get materials of model \"{}\": {}", model_path, err);
			return (used_materials, missing_materials);
		}
	};

	// Get cdmaterials / texture_paths
	let cdmaterials = match verified.mdl_header.texture_paths() {
		Ok(p) => p,
		Err(err) => {
			warn!("Failed to get cdmaterials of model \"{}\": {}", model_path, err);
			return (used_materials, missing_materials);
		}
	};

	// Add materials to used_materials / missing_materials
	for material in materials {

		// Get material name
		let material_name = match material.name() {
			Ok(name) => name,
			Err(err) => {
				warn!("Failed to get material name from model \"{}\": {}", model_path, err);
				continue;
			}
		};

		// Try to find material in source_files in any of its cdmaterials paths
		for cdmaterial_path in &cdmaterials {
			let source_path = format!("materials\\{}{}.vmt", cdmaterial_path, material_name)
				.replace("/", "\\")
				.to_lowercase();

			match source_files.get(&source_path) {
				Some(source_file) => {
					used_materials.insert(source_path, source_file.to_owned());
				}
				None => {
					missing_materials.insert(
						source_path,
						format!("Used by model \"{}\"", model_path),
					);
				}
			}
		}
		
	}

	return (used_materials, missing_materials);
}

/// Reads and parses material data from a VMT file
pub fn read_material_data(
	full_path: &str,
	source_files: &HashMap<String, SourceContentFile>,
	open_fs: &OpenFileSystem,
) -> Result<SourceMaterialData, SimpleError> {
	let material_file = fs::read(full_path)
		.map_err(|err| SimpleError::new(format!("Failed to read material file \"{}\": {}", full_path, err)))?;

	let material_parsed = plumber_core::vmt::from_bytes(&material_file)
		.map_err(|err| SimpleError::new(format!("Failed to parse material file \"{}\": {}", full_path, err)))?;

	return get_material_data(material_parsed, source_files, open_fs, full_path);
}

/// Extracts texture and material references from a parsed VMT
pub fn get_material_data(
	vmt: plumber_core::vmt::Vmt,
	source_files: &HashMap<String, SourceContentFile>,
	open_fs: &OpenFileSystem,
	logging_ref: &str,
) -> Result<SourceMaterialData, SimpleError> {
	let mut collection = SourceMaterialData::new();

	// Resolve shader with patch material support
	let shader = match vmt.resolve_shader_os(open_fs, |patch_path_local| {
		// SPECIAL CASE: Patch material
		// Try to find the material this patch material is patching
		let mut patch_source_path = patch_path_local.replace("/", "\\").to_lowercase();
		if !patch_source_path.ends_with(".vmt") {
			patch_source_path.push_str(".vmt");
		}

		match source_files.get(&patch_source_path) {
			Some(source_file) => {
				// Add patch material *itself* to the collection
				collection.used_materials.insert(patch_source_path.clone(), source_file.to_owned());

				// Read patch material and add its data to the collection
				// This is necessary since plumber_core will actually apply the patch, while the engine still needs the material to patch it itself
				if let Ok(patch_data) = read_material_data(&source_file.full_path, source_files, open_fs) {
					collection.extend(patch_data);
				}

				Ok(PathBuf::from(&source_file.full_path))
			}
			None => Err(plumber_core::vmt::ShaderResolveError::Io {
				path: String::from(patch_path_local),
				error: String::from("Did not find source file for material to be patched"),
			})
		}
	}) {
		Ok(shader) => shader,
		Err(err) => bail!("Failed to parse shader: {}", err),
	};

	// Iterate material parameters and add their value to used_textures / missing_textures if it is a texture parameter
	for (param_key, param_value) in shader.parameters {
		// SPECIAL CASE: $bottommaterial
		// This is a material parameter that takes a material as input, so we need to add it to the material collection
		if &param_key == UncasedStr::new("$bottommaterial") {
			let source_path = make_material_path(&param_value);
			match source_files.get(&source_path) {
				Some(source_file) => {
					collection.used_materials.insert(source_path, source_file.to_owned());
				}
				None => {
					collection.missing_materials.insert(
						source_path,
						format!("Used by material \"{}\" in $bottommaterial", logging_ref),
					);
				}
			}
			continue;
		}

		// Skip non-texture parameters
		if !VMT_TEXTURE_PARAMETERS.contains(&param_key.to_string().to_lowercase().as_str()) {
			continue;
		}

		let source_path = make_texture_path(&param_value);

		// Special case: $envmap can be set to "env_cubemap" which will be replaced dynamically by a built cubemap by the engine
		if source_path == VMT_ENVMAP_DEFAULT_SOURCE_PATH {
			continue;
		}

		match source_files.get(&source_path) {
			Some(source_file) => {
				collection.used_textures.insert(source_path, source_file.to_owned());
			}
			None => {
				collection.missing_textures.insert(
					source_path,
					format!("Used by material \"{}\" in {}", logging_ref, param_key),
				);
			}
		}
	}

	return Ok(collection);
}

/// Removes entries from a hashmap if they exist in the game filesystem
/// Returns the count of removed entries
pub fn remove_game_content(map: &mut HashMap<String, String>, fs: &OpenFileSystem) -> i32 {
	let mut removed = 0;
	map.retain(|file_path, _| {

		// plumber_core only allows "/" slashes and lowercase characters
		let game_path = file_path.replace("\\", "/").to_lowercase();

		// We need to use plumber_core::vpk::Path because only this way plumber_core looks in the *game* file system instead of the OS file system
		// It checks if a std library Path is provided or its custom one.
		let Some(path) = plumber_core::vpk::Path::try_from_str(&game_path) else {
			warn!("Failed to create game file path for \"{}\"", file_path);
			return true;
		};

		// Try to open material in game file system
		// The path is all lowercase but that is working and explicitly allowed (and required above) by plumber_core
		match fs.open_file(path) {
			Ok(_) => {
				removed += 1;
				return false;
			}
			Err(_) => true,
		}

	});
	return removed;
}

/// Logs missing files from a hashmap
pub fn log_missing_files(name: &str, map: &HashMap<String, String>) {
	warn!("Missing <red>{}</> {} in source files:", map.len(), name);
	for (path, reason) in map {
		warn!("\t<red>-</> {}", path);
		warn!("\t  ↳ {}", reason);
	}
}

/// Copies collected content files to the output directory
pub fn copy_files_to_output(
	source_files: &HashMap<String, SourceContentFile>,
	output_path: &Path,
	additional_extensions: Option<&[&str]>,
) {
	for source_file in source_files.values() {
		let output_file_path = output_path.join(&source_file.local_path);
		let Some(output_dir) = output_file_path.parent() else {
			warn!("Failed to get parent directory of \"{}\"", output_file_path.display());
			continue;
		};

		if let Err(err) = fs::create_dir_all(output_dir) {
			warn!("Failed to create directory \"{}\": {}", output_dir.display(), err);
			continue;
		}

		let source_path = Path::new(&source_file.full_path);
		if let Err(err) = fs::copy(source_path, &output_file_path) {
			warn!("Failed to copy \"{}\" to \"{}\": {}", source_file.full_path, output_file_path.display(), err);
		}

		// Copy additional file extensions (e.g., .vvd, .phy for models)
		if let Some(extensions) = additional_extensions {
			for ext in extensions {
				let source_ext = source_path.with_extension(ext);
				let output_ext = output_file_path.with_extension(ext);
				if let Err(err) = fs::copy(&source_ext, &output_ext) {
					warn!("Failed to copy \"{}\" to \"{}\": {}", source_ext.display(), output_ext.display(), err);
				}
			}
		}
	}
}

/// A tuple of found content files and missing content files with usage context.
/// - First element: File path -> Found content files in source directories
/// - Second element: Missing file path -> Descriptions of where they're referenced
type ContentSummary<'a> = (&'a HashMap<String, SourceContentFile>, &'a HashMap<String, String>);

/// Prints a content summary to the console
pub fn print_content_summary(
	source_files_count: usize,
	materials: ContentSummary,
	models: Option<ContentSummary>,
	textures: ContentSummary,
) {
	info!("<magenta>CONTENT SUMMARY:</>");
	info!("\t<magenta>↳</> Source files: Total <cyan>{}</>", source_files_count);
	info!("\t<magenta>↳</> Materials: Found <green>{}</>; Missing <red>{}</>", materials.0.len(), materials.1.len());
	if let Some((used, missing)) = models {
		info!("\t<magenta>↳</> Models: Found <green>{}</>; Missing <red>{}</>", used.len(), missing.len());
	}
	info!("\t<magenta>↳</> Textures: Found <green>{}</>; Missing <red>{}</>", textures.0.len(), textures.1.len());
}
