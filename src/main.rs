use mdlinker::config;
use mdlinker::lib;
use mdlinker::rules::Report;
use mdlinker::rules::ThirdPassReport;
use miette::{miette, Result};

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
            for report in e.reports {
                match report {
                    Report::SimilarFilename(e) => {
                        nb_errors += 1;
                        eprintln!("{e:?}");
                    }
                    Report::DuplicateAlias(e) => {
                        nb_errors += 1;
                        eprintln!("{e:?}");
                    }
                    Report::ThirdPass(ThirdPassReport::BrokenWikilink(e)) => {
                        nb_errors += 1;
                        eprintln!("{e:?}");
                    }
                    Report::ThirdPass(ThirdPassReport::UnlinkedText(e)) => {
                        nb_errors += 1;
                        eprintln!("{e:?}");
                    }
                }
            }
        }
    }

    if nb_errors > 0 {
        Err(miette!("Lint rules violated: {nb_errors}"))
    } else {
        Ok(())
    }
}
