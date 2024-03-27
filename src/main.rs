use maud::html;
use reqwest::blocking::get;
use std::fs;
use thiserror::Error;

use scraper::{selectable::Selectable, Html, Selector};

struct Competitor {
    name: String,
    profile: Option<String>,
    pr_3x3_avg: Option<String>,
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
    let url = "https://www.worldcubeassociation.org/competitions/HessenMiniOpen2024/registrations";
    let path = "test.html";

    generate_report(&url, &path)?;
    webbrowser::open(path)?;
    Ok(())
}

fn generate_report(competitors_url: &str, out_path: &str) -> Result<(), WCOError> {
    let competitors_html = Html::parse_document(&get(competitors_url)?.text()?);
    let competition_title = get_competition_title(&competitors_html)?;
    let mut competitors = parse_competitors(&competitors_html);
    retrieve_competitor_pr_avgs(&mut competitors)?;
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
            competitor.pr_3x3_avg = parse_pr_3x3_avg(&html);
        }
    }
    Ok(())
}

fn parse_pr_3x3_avg(competitor_html: &Html) -> Option<String> {
    let selector = Selector::parse(r#"a[href="/results/rankings/333/average"]"#)
        .expect("Parsing known selector should not fail");

    competitor_html
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned())
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
                            td { (match &competitor.pr_3x3_avg { Some(time) => format!("{}", time), None => "".to_string()}) }
                        }
                    }
                }
            }
        }
    };
    markup.into_string()
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
