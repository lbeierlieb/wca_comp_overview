use chrono::prelude::*;
use indicatif::ProgressBar;
use reqwest::blocking::get;
use scraper::Html;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{fs, str::FromStr};

use clap::Parser;

use css_generation::css_content;
use data_retrieval::competitorslist::get_competition_title;
use data_retrieval::competitorslist::parse_competitors;
use data_retrieval::pr_data_random::set_random_competitor_pr;
use data_retrieval::pr_data_unofficialapi::retrieve_competitor_pr;
use data_retrieval::pr_data_wcawebsite::retrieve_competitor_pr_avg_html;
use datastructures::Event;
use html_generation::generate_report_html;
use plot::plot;
use wcoerror::WCOError;

mod css_generation;
mod data_retrieval;
mod datastructures;
mod html_generation;
mod plot;
mod wcoerror;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[arg(short, long, default_value_t = String::from("."))]
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

    let report_index = generate_report(&args)?;
    if !args.no_browser {
        webbrowser::open(
            report_index
                .to_str()
                .ok_or(WCOError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "path to index of generated report is not valid",
                )))?,
        )?;
    }
    Ok(())
}

fn generate_report(args: &Args) -> Result<PathBuf, WCOError> {
    let competitors_html = Html::parse_document(&get(&args.url)?.text()?);
    let competition_title = get_competition_title(&competitors_html)?;
    println!(r#"Found competition "{}""#, competition_title);
    let parent_dir = PathBuf::from(&args.destination_directory).canonicalize()?;
    let report_dir = parent_dir.join(create_foldername(
        &competition_title,
        args.source == Source::Debug,
    ));
    if report_dir.exists() {
        println!(
            "Target folder {:?} already exists, overwriting contents",
            report_dir
        );
    } else {
        println!("Saving report in {:?}", report_dir);
        fs::create_dir(&report_dir)?;
    }
    let mut competitors = parse_competitors(&competitors_html);
    let num_competitors = competitors.len() as u64;
    println!(
        r#"Found {} competitors for competition "{}""#,
        num_competitors, competition_title
    );
    println!("Retrieving competitor PRs...");
    let mut all_events = HashSet::new();
    let bar = ProgressBar::new(num_competitors);
    for competitor in &mut competitors {
        for event in competitor.events.clone().into_iter() {
            match args.source {
                Source::UnofficialAPI => retrieve_competitor_pr(competitor, event)?,
                Source::WCAwebsite => retrieve_competitor_pr_avg_html(competitor, event)?,
                Source::Debug => set_random_competitor_pr(competitor, event),
            }
            all_events.insert(event);
        }
        bar.inc(1);
    }
    bar.finish();
    let report = generate_report_html(&competition_title, &competitors, &Event::Ev333);
    let report_index = report_dir.join("index.html");
    fs::write(&report_index, report)?;
    fs::write(report_dir.join("styles.css"), css_content())?;
    let plot_dir = report_dir.join("plots");
    if !plot_dir.exists() {
        fs::create_dir(&plot_dir)?;
    }
    for event in all_events {
        match plot(
            &competitors,
            &event,
            &plot_dir.join(format!("hist{}.png", event.code_name())),
        ) {
            Err(e) => return Err(WCOError::PlottingError(e.to_string())),
            _ => {}
        }
    }
    Ok(report_index)
}

fn create_foldername(comp_name: &str, debug: bool) -> String {
    let pathfriendly_name = comp_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let now = chrono::Local::now();
    format!(
        "{}-{}_{:0>2}_{:0>2}__{:0>2}_{:0>2}{}",
        pathfriendly_name,
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        match debug {
            true => "_DEBUG",
            false => "",
        }
    )
}
