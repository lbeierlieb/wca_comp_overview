use maud::html;
use reqwest::blocking::get;
use std::fs;

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

fn main() {
    //let list_document = Html::parse_document(
    //    &fs::read_to_string("example_html_files/competitors_list.html").unwrap(),
    //);
    //dbg!(get_competitors_and_profiles(&list_document));

    //let competitor_document = Html::parse_document(
    //    &fs::read_to_string("example_html_files/single_competitor.html").unwrap(),
    //);
    //dbg!(get_best_3x3_avg_from_html(&competitor_document));
    //dbg!(get_best_3x3_avg("/persons/2023BEIE01"));
    let competitor_data = get_competitors_and_times(
        "https://www.worldcubeassociation.org/competitions/HessenMiniOpen2024/registrations",
    );
    //let competitor_data = vec![
    //    Competitor::new("Hannah".into(), None),
    //    Competitor::new("Hannah Otto".into(), Some("profile".into())),
    //];
    let path = "test.html";
    let report = generate_report(&competitor_data);

    if let Err(err) = fs::write(path, report) {
        eprintln!("Error writing html file: {:?}", err);
    } else {
        if let Err(err) = webbrowser::open(path) {
            eprintln!("Error opening browser: {:?}", err);
        }
    }
}

fn get_competitors_and_times(url: &str) -> Vec<Competitor> {
    let html = Html::parse_document(&get(url).unwrap().text().unwrap());
    get_competitors_and_times_from_html(&html)
}

fn get_competitors_and_times_from_html(competitors_list: &Html) -> Vec<Competitor> {
    let mut competitors = get_competitors_and_profiles(competitors_list);
    for competitor in &mut competitors {
        competitor.pr_3x3_avg = competitor
            .profile
            .clone()
            .and_then(|profile| get_best_3x3_avg(&profile));
    }
    competitors
}

fn get_competitors_and_profiles(competitors_list: &Html) -> Vec<Competitor> {
    let selector = Selector::parse(r#"td[class="name"]"#).unwrap();
    let name_selector = &Selector::parse("a").unwrap();

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

fn get_best_3x3_avg(profile: &str) -> Option<String> {
    let url = format!("https://www.worldcubeassociation.org/{}", profile);
    let html = Html::parse_document(&get(url).unwrap().text().unwrap());
    get_best_3x3_avg_from_html(&html)
}

fn get_best_3x3_avg_from_html(competitor_html: &Html) -> Option<String> {
    let selector = Selector::parse(r#"a[href="/results/rankings/333/average"]"#).unwrap();

    competitor_html
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned())
}

fn generate_report(competitor_data: &[Competitor]) -> String {
    let markup = html! {
        html {
            head {
                title { "My Table" }

                // Include the link to your CSS file
                link rel="stylesheet" type="text/css" href="styles.css" {}
            }
            body {
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
