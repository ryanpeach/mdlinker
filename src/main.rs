use mdlinker::config;
use mdlinker::lib;
use miette::{miette, Result};

/// Really just a wrapper that loads the config and passes it to the main library function
fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let config = config::Config::new().map_err(|e| miette!(e))?;

    let mut nb_errors = 0;
    match lib(&config) {
        Err(e) => Err(e)?,
        Ok(e) => {
            for error in e.similar_filenames {
                nb_errors += 1;
                log::error!("{}", miette!(error));
            }
            for error in e.duplicate_aliases {
                nb_errors += 1;
                log::error!("{}", miette!(error));
            }
            for error in e.broken_wikilinks {
                nb_errors += 1;
                log::error!("{}", miette!(error));
            }
        }
    }

    if nb_errors > 0 {
        Err(miette!("Lint rules violated: {nb_errors}"))
    } else {
        Ok(())
    }
}
