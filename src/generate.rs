use crate::cli::Cli;
use clap::{Args, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};
use clap_mangen::Man;
use std::io::Write;
use std::{fs, io};

#[derive(Debug, Args)]
pub struct GenerateSubcommand {
    pub generate: Generate,

    /// set output file path
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,

    /// generate complete for which shell
    ///
    /// if no set, use shell from env.
    #[arg(short, long)]
    pub shell: Option<Shell>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Generate {
    /// generate manual doc
    Manual,
    /// generate sheel complete
    Complete,
}

fn generate_man_doc(output: &mut dyn Write) {
    let man = Man::new(Cli::command());
    man.render(output).unwrap();
}

fn generate_shell_complete(output: &mut dyn Write, shell: Shell) {
    generate(shell, &mut Cli::command(), env!("CARGO_PKG_NAME"), output)
}

impl GenerateSubcommand {
    pub async fn run(self) {
        let output: &mut dyn Write = match self.output {
            Some(path) => &mut fs::File::create(path).unwrap(),
            None => &mut io::stdout(),
        };

        match self.generate {
            Generate::Manual => generate_man_doc(output),
            Generate::Complete => {
                generate_shell_complete(output, self.shell.or_else(Shell::from_env).unwrap())
            }
        }
    }
}
