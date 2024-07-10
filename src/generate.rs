use crate::cli::Cli;
use clap::Args;
use clap::CommandFactory;
use clap::ValueEnum;
use clap_mangen::Man;
use std::fs;
use std::io;
use std::io::Write;

#[derive(Debug, Args)]
pub struct GenerateSubcommand {
    pub generate: Generate,

    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Generate {
    /// generate manual doc
    Manual,
    /// generate sheel complete
    Shell,
}

fn generate_man_doc(output: &mut dyn Write) {
    let man = Man::new(Cli::command());
    man.render(output).unwrap();
}

impl GenerateSubcommand {
    pub async fn run(self) {
        let output: &mut dyn Write = match self.output {
            Some(path) => &mut fs::File::create(path).unwrap(),
            None => &mut io::stdout(),
        };

        match self.generate {
            Generate::Manual => generate_man_doc(output),
            Generate::Shell => todo!(),
        }
    }
}
