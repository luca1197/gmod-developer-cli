use inquire::{Text, required, Select, Confirm};

pub fn text_required(prompt: &str) -> String {

	let res_string = Text::new(prompt)
		.with_validator(required!("This field is required!"))
		.prompt()
		.unwrap();

	return res_string;

}

pub fn text_optional(prompt: &str, default: &str) -> String {

	let res_string = Text::new(prompt)
		.with_default(default)
		.prompt()
		.unwrap();

	return res_string;

}

pub fn selector(prompt: &str, options: &Vec<&str>) -> String {

	let res_string = Select::new(prompt, options.to_vec())
		.prompt()
		.unwrap();

	return res_string.to_string();

}

pub fn selector_index<'a>(prompt: &str, options: &Vec<&str>) -> usize {

	let res_string = Select::new(prompt, options.to_vec())
		.prompt()
		.unwrap();

	let res_index = options.iter().position(
		|&s| s == res_string
	).unwrap();

	return res_index;

}

pub fn confirm_no(prompt: &str) -> bool {
	return Confirm::new(prompt)
		.with_default(false)
		.prompt()
		.unwrap();
}

pub fn confirm_yes(prompt: &str) -> bool {
	return Confirm::new(prompt)
		.with_default(true)
		.prompt()
		.unwrap();
}