use regex::Regex;
use std::path::Path;

pub fn validate_input_dirname(path: &str, input: &str) -> Result<String, String> {

	let dirname: String = input.parse().unwrap();

	let regex = Regex::new(r"[^\w\d_-]").unwrap();
	if regex.is_match(&dirname) {
		return Err(format!("name should only contain letters, numbers, dashes and underscores!"));
	}

	if Path::new(format!("{path}/{dirname}").as_str()).exists() {
		return Err(format!("Directory with specified name already exists!"))
	}

	return Ok(dirname);

}