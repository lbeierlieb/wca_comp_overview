use indicatif::ProgressBar;
use reqwest::blocking::get;
use scraper::Html;
use std::{fs, str::FromStr};

use clap::Parser;

use data_retrieval::competitorslist::get_competition_title;
use data_retrieval::competitorslist::parse_competitors;
use data_retrieval::pr_data_random::set_random_competitor_pr_avg;
use data_retrieval::pr_data_unofficialapi::retrieve_competitor_pr_avg_json;
use data_retrieval::pr_data_wcawebsite::retrieve_competitor_pr_avg_html;
use html_generation::generate_report_html;
use plot::plot;
use wcoerror::WCOError;

mod data_retrieval;
mod datastructures;
mod html_generation;
mod plot;
mod wcoerror;

#[derive(Debug, Clone)]
enum Source {
    UnofficialAPI,
    WCAwebsite,
    Debug,
}
impl FromStr for Source {
    type Err = WCOError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "wcawebsite" => Ok(Source::WCAwebsite),
            "unofficialapi" => Ok(Source::UnofficialAPI),
            "debug" => Ok(Source::Debug),
            _ => Err(WCOError::ParsingError(format!(
                "Invalid source specified: \"{}\"",
                s
            ))),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the competition's competitor list page
    #[arg(index = 1)]
    url: String,

    /// Directory where to save the report (default: current directory)
    #[arg(short, long, default_value_t = String::new())]
    destination_directory: String,

    /// Source where to retrieve PR averages from. Available: UnofficialAPI, WCAwebsite
    #[arg(short, long, default_value = "UnofficialAPI")]
    source: Source,

    /// Do not open generated report in system default browser
    #[arg(short, long, default_value_t = false)]
    no_browser: bool,
}

fn main() -> Result<(), WCOError> {
    let args = Args::parse();

    let path = format!("{}test.html", args.destination_directory);

    generate_report(&args.url, &path, &args.source)?;
    if !args.no_browser {
        webbrowser::open(&path)?;
    }
    Ok(())
}

fn generate_report(competitors_url: &str, out_path: &str, source: &Source) -> Result<(), WCOError> {
    let competitors_html = Html::parse_document(&get(competitors_url)?.text()?);
    let competition_title = get_competition_title(&competitors_html)?;
    let mut competitors = parse_competitors(&competitors_html);
    let num_competitors = competitors.len() as u64;
    println!(
        r#"Found {} competitors for competition "{}""#,
        num_competitors, competition_title
    );
    println!("Retrieving competitor PRs...");
    let bar = ProgressBar::new(num_competitors);
    for competitor in &mut competitors {
        match source {
            Source::UnofficialAPI => retrieve_competitor_pr_avg_json(competitor)?,
            Source::WCAwebsite => retrieve_competitor_pr_avg_html(competitor)?,
            Source::Debug => set_random_competitor_pr_avg(competitor),
        }
        bar.inc(1);
    }
    bar.finish();
    let report = generate_report_html(&competition_title, &competitors);
    fs::write(out_path, report)?;
    match plot(&competitors) {
        Err(e) => return Err(WCOError::PlottingError(e.to_string())),
        _ => {}
    }
    Ok(())
}
