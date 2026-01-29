use std::{fs, path::PathBuf};
use paris::{error, info, success, warn};

pub fn output_vmf_stats(vmf_path: &PathBuf) {

	//
	// Read vmf
	//
	info!("Reading vmf \"<green>{}</>\"...", vmf_path.display());
	let vmf_content = match fs::read(vmf_path) {
		Ok(content) => content,
		Err(err) => {
			error!("Failed to read vmf file in \"{}\": {}", vmf_path.display(), err.to_string());
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
			error!("Failed to parse vmf file in \"{}\": {}", vmf_path.display(), err.to_string());
			return;
		}
	};


	let mut count_solid = 0;
	let mut count_faces = 0;
	let mut count_vertices = 0;


	let mut out_positions = vec![];


	for solid in vmf_parsed.world.solids {

		count_solid += 1;

		count_faces += solid.sides.len();
		count_vertices += solid.sides.len() * 3;

		for side in solid.sides {

			out_positions.push(side.plane.0);
			out_positions.push(side.plane.1);
			out_positions.push(side.plane.2);

		}

	}

	info!("Solids: {}", count_solid);
	info!("Faces: {}", count_faces);
	info!("Vertices: {}", count_vertices);



	let mut out_strings_x = vec![];
	let mut out_strings_y = vec![];
	for pos in out_positions {
		let full_str = pos.to_string().replace("[", "").replace("]", "");
		let parts: Vec<&str> = full_str.split(',').collect();
		out_strings_x.push(parts[0].to_string());
		out_strings_y.push(parts[1].to_string());
	}

	let out_string_x = out_strings_x.join("\n");
	fs::write("./positions_x.txt", out_string_x);

	let out_string_y = out_strings_y.join("\n");
	fs::write("./positions_y.txt", out_string_y);

}