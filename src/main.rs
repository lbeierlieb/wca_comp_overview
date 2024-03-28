use maud::html;
use rand::prelude::*;
use reqwest::blocking::get;
use std::{fs, ops::Add, time::Duration};
use thiserror::Error;

use clap::Parser;
use scraper::{selectable::Selectable, Html, Selector};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the competition's competitor list page
    #[arg(index = 1)]
    url: String,

    /// Directory where to save the report (default: current directory)
    #[arg(short, long, default_value_t = String::new())]
    destination_directory: String,

    /// Do not retrieve times from user profile, instead generate random times
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Do not open generated report in system default browser
    #[arg(short, long, default_value_t = false)]
    no_browser: bool,
}

struct Competitor {
    name: String,
    profile: Option<String>,
    pr_3x3_avg: Option<Duration>,
}

impl Competitor {
    fn new(name: String, profile: Option<String>) -> Self {
        Competitor {
            name,
            profile,
            pr_3x3_avg: None,
        }
    }
}

fn main() -> Result<(), WCOError> {
    let args = Args::parse();

    let path = format!("{}test.html", args.destination_directory);

    generate_report(&args.url, &path, args.debug)?;
    if !args.no_browser {
        webbrowser::open(&path)?;
    }
    Ok(())
}

fn generate_report(competitors_url: &str, out_path: &str, debug: bool) -> Result<(), WCOError> {
    let competitors_html = Html::parse_document(&get(competitors_url)?.text()?);
    let competition_title = get_competition_title(&competitors_html)?;
    let mut competitors = parse_competitors(&competitors_html);
    if !debug {
        retrieve_competitor_pr_avgs(&mut competitors)?;
    } else {
        set_random_competitor_pr_avgs(&mut competitors);
    }
    let report = generate_report_html(&competition_title, &competitors);
    fs::write(out_path, report)?;
    Ok(())
}

fn get_competition_title(html: &Html) -> Result<String, WCOError> {
    let selector = Selector::parse(r#"h3"#).expect("Parsing known selector should not fail");

    let h3_headlines: Vec<_> = html.select(&selector).collect();
    match h3_headlines.len() {
        1 => Ok(h3_headlines[0].text().collect::<String>().trim().to_owned()),
        len => Err(WCOError::ParsingError(format!(
            "Expected competition title to be the only h3 element, but there were {}",
            len
        ))),
    }
}

fn parse_competitors(competitors_list: &Html) -> Vec<Competitor> {
    let selector =
        Selector::parse(r#"td[class="name"]"#).expect("Parsing known selector should not fail");
    let name_selector = &Selector::parse("a").expect("Parsing known selector should not fail");

    competitors_list
        .select(&selector)
        .map(|element| {
            Competitor::new(
                element.text().collect::<String>().trim().to_owned(),
                element
                    .select(&name_selector)
                    .next()
                    .and_then(|elem| elem.value().attr("href").map(|str| str.to_owned())),
            )
        })
        .collect()
}

fn retrieve_competitor_pr_avgs(competitors: &mut [Competitor]) -> Result<(), WCOError> {
    for competitor in competitors {
        if let Some(profile) = &mut competitor.profile {
            let url = format!("https://www.worldcubeassociation.org/{}", profile);
            let html = Html::parse_document(&get(url)?.text()?);
            competitor.pr_3x3_avg = parse_pr_3x3_avg(&html)?;
        }
    }
    Ok(())
}

fn parse_pr_3x3_avg(competitor_html: &Html) -> Result<Option<Duration>, WCOError> {
    let selector = Selector::parse(r#"a[href="/results/rankings/333/average"]"#)
        .expect("Parsing known selector should not fail");

    match competitor_html
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned())
    {
        Some(time_str) => Ok(Some(parse_time(&time_str)?)),
        None => Ok(None),
    }
}

fn set_random_competitor_pr_avgs(competitors: &mut [Competitor]) {
    let mut rng = rand::thread_rng();
    for competitor in competitors {
        if let Some(_) = &mut competitor.profile {
            competitor.pr_3x3_avg = Some(
                parse_time(&format!(
                    "{}.{}",
                    rng.gen_range(7..40),
                    rng.gen_range(10..100)
                ))
                .expect("Generated times should not fail to parse"),
            )
        }
    }
}

fn generate_report_html(competition_title: &str, competitor_data: &[Competitor]) -> String {
    let markup = html! {
        html {
            head {
                title { "My Table" }
                link rel="stylesheet" type="text/css" href="styles.css" {}
            }
            body {
                h1 {
                    (competition_title)
                }
                table {
                    @for competitor in competitor_data {
                        tr {
                            @if let Some(profile) = &competitor.profile {
                                td {
                                    a target="_blank" href=(format!("https://www.worldcubeassociation.org/{}", profile)) {
                                        (competitor.name)
                                    }
                                }
                            } @ else {
                                td { (competitor.name) }
                            }
                            td { (match &competitor.pr_3x3_avg { Some(time) => format_time(time), None => "".to_string()}) }
                        }
                    }
                }
            }
        }
    };
    markup.into_string()
}

fn parse_time(text: &str) -> Result<Duration, WCOError> {
    let msg = format!("Cannot parse time from string \"{}\"", text);
    let splits: Vec<_> = text.split(":").collect();
    match splits.len() {
        1 => parse_sub_minute_time(text),
        2 => {
            let sub_min = parse_sub_minute_time(splits[1])?;
            match splits[0].parse::<u64>() {
                Ok(mins) if mins < 60 => Ok(Duration::from_secs(mins * 60).add(sub_min)),
                _ => Err(WCOError::ParsingError(msg)),
            }
        }
        _ => Err(WCOError::ParsingError(msg)),
    }
}
fn parse_sub_minute_time(text: &str) -> Result<Duration, WCOError> {
    let msg = format!("Cannot parse sub-minute time from string \"{}\"", text);
    let splits: Vec<_> = text.split(".").collect();
    if splits.len() != 2 {
        return Err(WCOError::ParsingError(msg));
    }
    let secs = splits[0].parse::<u64>();
    let subsec = splits[1].parse::<u32>();
    match (secs, subsec) {
        (Ok(secs), Ok(subsec)) if secs < 60 && subsec < 100 => {
            Ok(Duration::new(secs, subsec * 1000 * 1000 * 10))
        }
        _ => Err(WCOError::ParsingError(msg)),
    }
}
fn format_time(time: &Duration) -> String {
    let subsec = time.subsec_millis() / 10;
    let sec = time.as_secs() % 60;
    let min = time.as_secs() / 60;
    match min {
        0 => format!("{}.{:0>2}", sec, subsec),
        _ => format!("{}:{:0>2}.{:0>2}", min, sec, subsec),
    }
}

#[derive(Error, Debug)]
pub enum WCOError {
    #[error("Invalid input: {0}")]
    ParsingError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
