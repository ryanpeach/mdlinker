use mdlinker::config;
use mdlinker::lib;
use miette::{miette, Report, Result};

/// Really just a wrapper that loads the config and passes it to the main library function
fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let config = config::Config::new().map_err(|e| miette!(e))?;

    let mut nb_errors = 0;
    match lib(&config) {
        Err(e) => {
            eprintln!("{:?}", miette!(e));
            return Err(miette!("Something went wrong during linting"));
        }
        Ok(e) => {
            println!();
            for error in e.similar_filenames {
                nb_errors += 1;
                eprintln!("{:?}", Report::from(error));
            }
            for error in e.duplicate_aliases {
                nb_errors += 1;
                eprintln!("{:?}", Report::from(error));
            }
            for error in e.broken_wikilinks {
                nb_errors += 1;
                eprintln!("{:?}", Report::from(error));
            }
            for error in e.unlinked_texts {
                nb_errors += 1;
                eprintln!("{:?}", Report::from(error));
            }
        }
    }

    if nb_errors > 0 {
        Err(miette!("Lint rules violated: {nb_errors}"))
    } else {
        Ok(())
    }
}
