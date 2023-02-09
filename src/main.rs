use clap::{Parser, Subcommand};

// cli
mod cli {
	pub mod addon;
	pub mod entity;
}
use cli::addon;
use cli::entity;

mod templates {
	pub mod addon;
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
				entity::Actions::Create { name } => {
					entity::create(name);
				}

			}
		}

	}

}
