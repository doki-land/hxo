use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use console::style;
use hxo_compiler::{CompileOptions, Compiler};
use hxo_lsp::run_server;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "hxo")]
#[command(about = "HXO Framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a HXO project or component
    Build {
        /// Input file or directory
        #[arg(default_value = ".")]
        input: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,

        /// Enable production mode
        #[arg(long)]
        prod: bool,

        /// Enable Server-Side Rendering
        #[arg(long)]
        ssr: bool,

        /// Enable Hydration mode
        #[arg(long)]
        hydrate: bool,

        /// Enable minification
        #[arg(short, long)]
        minify: bool,

        /// Target JavaScript version (e.g., es2022, esnext)
        #[arg(long)]
        target: Option<String>,

        /// Locale for zero-runtime i18n optimization
        #[arg(long)]
        locale: Option<String>,
    },
    /// Initialize a new HXO project
    Init {
        /// Project name
        name: String,
    },
    /// Start development server
    Dev {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Start Language Server
    Lsp,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { input, output, prod, ssr, hydrate, minify, target, locale } => {
            println!("{} Building project...", style("●").blue());

            if input.is_file() {
                let source = fs::read_to_string(&input)?;
                let component_name = input.file_stem().unwrap().to_string_lossy();

                let mut compiler = Compiler::new();
                let options = CompileOptions {
                    ssr,
                    hydrate: if hydrate { true } else { !ssr },
                    minify: minify || prod,
                    is_prod: prod,
                    target,
                    i18n_locale: locale,
                    ..Default::default()
                };

                match compiler.compile_with_options(&component_name, &source, options) {
                    Ok(result) => {
                        fs::create_dir_all(&output)?;
                        let out_file = output.join(format!("{}.js", component_name));
                        fs::write(&out_file, &result.code)?;

                        if let Some(map) = result.source_map {
                            let map_file = output.join(format!("{}.js.map", component_name));
                            fs::write(map_file, map.to_json()?)?;
                        }

                        println!("{} Build complete!", style("✔").green());
                    }
                    Err(e) => {
                        eprintln!("{} Build failed: {:?}", style("✘").red(), e);
                        std::process::exit(1);
                    }
                }
            }
            else {
                println!("{} Multi-file build not yet implemented", style("!").yellow());
            }
        }
        Commands::Init { name } => {
            println!("{} Initializing project {}...", style("●").blue(), name);
            // TODO: Implement project scaffolding
            println!("{} Project {} initialized!", style("✔").green(), name);
        }
        Commands::Dev { port } => {
            println!("{} Starting dev server on port {}...", style("●").blue(), port);
            // TODO: Implement dev server
        }
        Commands::Lsp => {
            run_server().await;
        }
    }

    Ok(())
}
