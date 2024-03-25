use reqwest::blocking::get;
use std::fs;

use scraper::{selectable::Selectable, Html, Selector};

fn main() {
    let list_document = Html::parse_document(
        &fs::read_to_string("example_html_files/competitors_list.html").unwrap(),
    );
    dbg!(get_competitors_and_profiles(&list_document));

    let competitor_document = Html::parse_document(
        &fs::read_to_string("example_html_files/single_competitor.html").unwrap(),
    );
    dbg!(get_best_3x3_avg_from_html(&competitor_document));
    dbg!(get_best_3x3_avg("/persons/2023BEIE01"));
    for (name, time) in get_competitors_and_times(
        "https://www.worldcubeassociation.org/competitions/HessenMiniOpen2024/registrations",
    ) {
        if let Some(t) = time {
            println!("{}: {}", name, t);
        } else {
            println!("{}", name);
        }
    }
}

fn get_competitors_and_times(url: &str) -> Vec<(String, Option<String>)> {
    let html = Html::parse_document(&get(url).unwrap().text().unwrap());
    get_competitors_and_times_from_html(&html)
}

fn get_competitors_and_times_from_html(competitors_list: &Html) -> Vec<(String, Option<String>)> {
    get_competitors_and_profiles(competitors_list)
        .into_iter()
        .map(|(name, profile)| (name, profile.and_then(|p| get_best_3x3_avg(&p))))
        .collect()
}

fn get_competitors_and_profiles(competitors_list: &Html) -> Vec<(String, Option<String>)> {
    let selector = Selector::parse(r#"td[class="name"]"#).unwrap();
    let name_selector = &Selector::parse("a").unwrap();

    competitors_list
        .select(&selector)
        .map(|element| {
            (
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
