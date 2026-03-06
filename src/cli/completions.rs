use clap::CommandFactory;
use clap_complete::Shell;

pub fn run(shell: Shell) {
    let mut cmd = super::Cli::command();
    clap_complete::generate(shell, &mut cmd, "devs", &mut std::io::stdout());
    eprintln!(
        "# Completions generated. See 'devs completions --help' or the README for setup instructions."
    );
}
