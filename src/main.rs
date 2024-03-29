use maud::html;
use rand::prelude::*;
use regex::Regex;
use reqwest::blocking::get;
use serde::Deserialize;
use std::{fs, ops::Add, str::FromStr, time::Duration};
use thiserror::Error;

use clap::Parser;
use scraper::{selectable::Selectable, Html, Selector};

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

#[derive(Deserialize)]
struct Person {
    rank: Rank,
}

#[derive(Deserialize)]
struct Rank {
    averages: Vec<PR>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PR {
    best: u32,
    eventId: String,
}

#[derive(Debug, Clone)]
struct Competitor {
    name: String,
    wca_id: Option<String>,
    pr_3x3_avg: Option<Duration>,
}

impl Competitor {
    fn new(name: String, wca_id: Option<String>) -> Self {
        Competitor {
            name,
            wca_id,
            pr_3x3_avg: None,
        }
    }
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
    match source {
        Source::UnofficialAPI => retrieve_competitor_pr_avgs_json(&mut competitors)?,
        Source::WCAwebsite => retrieve_competitor_pr_avgs_html(&mut competitors)?,
        Source::Debug => set_random_competitor_pr_avgs(&mut competitors),
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
                    .and_then(|elem| elem.value().attr("href").and_then(parse_wca_id)),
            )
        })
        .collect()
}

fn parse_wca_id(profile_url: &str) -> Option<String> {
    let re = Regex::new(r"/persons/([0-9]{4}[A-Z]{4}[0-9]{2})").unwrap();
    re.captures(profile_url).map(|cap| cap[1].to_owned())
}

fn retrieve_competitor_pr_avgs_html(competitors: &mut [Competitor]) -> Result<(), WCOError> {
    for competitor in competitors {
        if let Some(id) = &mut competitor.wca_id {
            let url = format!("https://www.worldcubeassociation.org/persons/{}", id);
            let html = Html::parse_document(&get(url)?.text()?);
            competitor.pr_3x3_avg = parse_pr_3x3_avg_html(&html)?;
        }
    }
    Ok(())
}

fn parse_pr_3x3_avg_html(competitor_html: &Html) -> Result<Option<Duration>, WCOError> {
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

fn retrieve_competitor_pr_avgs_json(competitors: &mut [Competitor]) -> Result<(), WCOError> {
    for competitor in competitors {
        if let Some(id) = &mut competitor.wca_id {
            let url = format!("https://raw.githubusercontent.com/robiningelbrecht/wca-rest-api/master/api/persons/{}.json", id);
            let json: Person = serde_json::from_str(&get(url)?.text()?)?;
            competitor.pr_3x3_avg = parse_pr_3x3_avg_json(&json);
        }
    }
    Ok(())
}

fn parse_pr_3x3_avg_json(competitor_json: &Person) -> Option<Duration> {
    competitor_json
        .rank
        .averages
        .iter()
        .filter(|pr| pr.eventId == "333")
        .map(|pr| pr.best)
        .map(|time| Duration::new(time as u64 / 100, (time % 100) * 10 * 1000 * 1000))
        .next()
}

fn set_random_competitor_pr_avgs(competitors: &mut [Competitor]) {
    let mut rng = rand::thread_rng();
    for competitor in competitors {
        if let Some(_) = &mut competitor.wca_id {
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
    let mut competitors_no_id = vec![];
    let mut competitors_no_time = vec![];
    let mut competitors_time = vec![];
    for comp in competitor_data {
        match comp {
            Competitor {
                name: _,
                wca_id: None,
                pr_3x3_avg: _,
            } => competitors_no_id.push(comp),
            Competitor {
                name: _,
                wca_id: Some(_),
                pr_3x3_avg: None,
            } => competitors_no_time.push(comp),
            _ => competitors_time.push(comp),
        }
    }
    let num_time = competitors_time.len();
    let num_no_time = competitors_no_time.len();
    let num_no_id = competitors_no_id.len();
    competitors_time.sort_by_key(|comp| comp.pr_3x3_avg);
    let mut all_competitors = competitors_time;
    all_competitors.append(&mut competitors_no_time);
    all_competitors.append(&mut competitors_no_id);
    let markup = html! {
        html {
            head {
                title { "My Table" }
                link rel="stylesheet" type="text/css" href="styles.css" {}
            }
            body {
                div class="container" {
                    h1 {
                        (competition_title)
                    }
                    p {
                        "There is a total of " b { (all_competitors.len()) } " competitors registered. They consists of:"
                    }
                    ul {
                        li {
                            b { (num_time) } ", who have competed in 3x3 before"
                        }
                        li {
                            b { (num_no_time) } ", who have competed at WCA events before, but not in 3x3"
                        }
                        li {
                            b { (num_no_id) } ", who have never competed at an WCA event before"
                        }
                    }
                    table {
                        tr {
                            th {
                                "Competitor"
                            }
                            th {
                                "3x3 PR Average"
                            }
                        }
                        @for competitor in all_competitors {
                            tr {
                                @if let Some(id) = &competitor.wca_id {
                                    td {
                                        a target="_blank" href=(format!("https://www.worldcubeassociation.org/persons/{}", id)) {
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

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}
