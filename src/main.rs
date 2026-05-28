fn main() {
    if let Err(err) = run_main() {
        eprintln!("treefold error: {err}");
        std::process::exit(1);
    }
}

fn run_main() -> anyhow::Result<()> {
    let parsed = treefold::cli::parse_args(std::env::args().skip(1)).map_err(anyhow::Error::msg)?;
    if parsed.help {
        println!("{}", treefold::cli::help_text());
        return Ok(());
    }

    let path = if let Some(path) = parsed.path {
        if !path.exists() {
            anyhow::bail!("invalid path: {}", path.display());
        }
        path
    } else {
        std::env::current_dir()?
    };

    match parsed.mode {
        treefold::cli::Mode::Tui => treefold::app::run_with_path(path),
        treefold::cli::Mode::Gui => {
            treefold::gui::run_with_path(Some(path)).map_err(anyhow::Error::msg)
        }
    }
}
