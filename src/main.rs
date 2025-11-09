use mdlinker::config;
use mdlinker::lib;
use mdlinker::rules::Report as MdReport;
use mdlinker::rules::ThirdPassReport;
use miette::{miette, Report, Result};

/// Really just a wrapper that loads the config and passes it to the main library function
fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let mut config = config::Config::new().map_err(|e| miette!(e))?;

    let mut nb_errors = 0;
    match lib(&config) {
        Err(e) => {
            return Err(Report::from(e));
        }
        Ok(e) => {
            println!();
            for report in e.reports() {
                match report {
                    MdReport::SimilarFilename(e) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e.clone()));
                        if config.ignore_remaining {
                            config.add_report_to_ignore(e);
                        }
                    }
                    MdReport::DuplicateAlias(e) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e.clone()));
                        if config.ignore_remaining {
                            config.add_report_to_ignore(e);
                        }
                    }
                    MdReport::ThirdPass(ThirdPassReport::BrokenWikilink(e)) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e.clone()));
                        if config.ignore_remaining {
                            config.add_report_to_ignore(e);
                        }
                    }
                    MdReport::ThirdPass(ThirdPassReport::UnlinkedText(e)) => {
                        nb_errors += 1;
                        eprintln!("{:?}", Report::from(e.clone()));
                        if config.ignore_remaining {
                            config.add_report_to_ignore(e);
                        }
                    }
                }
            }
        }
    }

    if nb_errors > 0 && !config.ignore_remaining {
        Err(miette!("Lint rules violated: {nb_errors}"))
    } else if nb_errors > 0 {
        println!("Lint rules ignored: {nb_errors}");
        if config.ignore_remaining {
            config.save_config()?;
        }
        Ok(())
    } else {
        Ok(())
    }
}
