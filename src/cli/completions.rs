use clap::CommandFactory;
use clap_complete::Shell;

pub fn run(shell: Shell) {
    let mut cmd = super::Cli::command();
    clap_complete::generate(shell, &mut cmd, "devs", &mut std::io::stdout());
    eprintln!("# Static completions generated (subcommands and flags only).");
    eprintln!(
        "# For dynamic project name completions, see 'devs completions --help' or the README."
    );
}
