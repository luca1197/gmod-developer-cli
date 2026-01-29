use regex::Regex;
use std::path::{Path, PathBuf};

pub fn validate_input_dirname(path: &str, input: &str, fs_check: bool) -> Result<String, String> {

	let dirname: String = input.parse().unwrap();

	let regex = Regex::new(r"[^\w\d_-]").unwrap();
	if regex.is_match(&dirname) {
		return Err("The directory name should only contain letters, numbers, dashes and underscores! Example: my_new_name".to_string());
	}

	if fs_check && Path::new(format!("{path}/{dirname}").as_str()).exists() {
		return Err("Directory with specified name already exists!".to_string())
	}

	return Ok(dirname);

}

pub fn validate_path_is_directory(path: &str) -> Result<PathBuf, String> {

	let path = Path::new(path);

	// Check if path is a directory
	if path.is_dir() {
		return Ok(path.into());
	} else {
		return Err("Provided path is not a directory".to_owned());
	}

}

pub fn validate_input_file_exists(path_to_file: &str, expected_extension: &str) -> Result<PathBuf, String> {

	let path = Path::new(path_to_file);

	// Check file extension
	match path.extension() {
		None => return Err("Missing file extension in provided path".to_owned()),
		Some(extension) => if extension != expected_extension { return Err("Missing file extension in provided path".to_owned()) }
	}

	// Check if file exists
	match path.try_exists() {
		Err(error) => return Err(format!("File existence check failed: {}", error)),
		Ok(exists) => if !exists { return Err("File does not exist".to_owned()) }
	}

	return Ok(path.into());

}
