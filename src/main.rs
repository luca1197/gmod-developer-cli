use clap::{Parser, Subcommand};

// cli
mod cli {
	pub mod addon;
	pub mod entity;
}
use cli::addon;
use cli::entity;

// library
mod library {
	pub mod validation;
	pub mod inquire;
}

// templates
mod templates {
	pub mod addon;
	pub mod entity;
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands
}

#[derive(Subcommand)]
enum Commands {
	Addon {
		#[command(subcommand)]
		action: addon::Actions,
	},
	Entity {
		#[command(subcommand)]
		action: entity::Actions,
	}
}

fn main() {

	let cli = Cli::parse();

	match cli.command {

		// addon <action>
		Commands::Addon { action } => {
			match action {

				// addon init <name>
				addon::Actions::Init { target_directory } => {
					addon::init(target_directory);
				}

			}
		}

		// entity <action>
		Commands::Entity { action } => {
			match action {
				
				// entity create <name>
				entity::Actions::Create { directory_name } => {
					entity::create(directory_name);
				}

			}
		}

	}

}
