use mdlinker::config;
use mdlinker::lib;
use mdlinker::rules::Report as MdReport;
use mdlinker::rules::ThirdPassReport;
use miette::{miette, Report, Result};

/// Really just a wrapper that loads the config and passes it to the main library function
fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let config = config::Config::new().map_err(|e| miette!(e))?;

    let mut nb_errors = 0;
    match lib(&config) {
        Err(e) => {
            return Err(Report::from(e));
        }
        Ok(e) => {
            println!();
            for report in e.reports {
                match report {
                    MdReport::SimilarFilename(e) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e));
                    }
                    MdReport::DuplicateAlias(e) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e));
                    }
                    MdReport::ThirdPass(ThirdPassReport::BrokenWikilink(e)) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e));
                    }
                    MdReport::ThirdPass(ThirdPassReport::UnlinkedText(e)) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e));
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
