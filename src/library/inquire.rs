use inquire::{Text, required, Select, Confirm};

pub fn required_text(prompt: &str) -> String {
	return Text::new(prompt)
		.with_validator(required!("This field is required!"))
		.prompt()
		.unwrap();
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