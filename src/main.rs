use clap::{Parser, Subcommand};

// cli
mod cli {
	pub mod addon;
	pub mod entity;
	pub mod vmf;
}
use cli::addon;
use cli::entity;
use cli::vmf;

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
	},
	VMF {
		#[command(subcommand)]
		action: vmf::Actions,
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

		// vmf <action>
		Commands::VMF { action } => {
			match action {

				// vmf collect-content <vmf-path>
				vmf::Actions::CollectContent { vmf_path, source_path, output_path } => {
					vmf::content_collector::collect_content(&vmf_path, source_path, &output_path);
				}

				vmf::Actions::Stats { vmf_path } => {
					vmf::stats::output_vmf_stats(&vmf_path);
				}

			}
		}

	}

}
