use crate::datastructures::{Competitor, Event};
use crate::wcoerror::WCOError;
use reqwest::blocking::get;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct Person {
    rank: Rank,
}

#[derive(Deserialize)]
struct Rank {
    averages: Vec<PR>,
    singles: Vec<PR>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PR {
    best: u32,
    eventId: String,
}

pub fn retrieve_competitor_pr(competitor: &mut Competitor, event: Event) -> Result<(), WCOError> {
    if let Some(id) = &mut competitor.wca_id {
        let url = format!("https://raw.githubusercontent.com/robiningelbrecht/wca-rest-api/master/api/persons/{}.json", id);
        let json: Person = serde_json::from_str(&get(url)?.text()?)?;
        if let Some(avg) = parse_pr_json(&json, event) {
            competitor.personal_records.insert(event, avg);
        }
    }
    Ok(())
}

fn parse_pr_json(competitor_json: &Person, event: Event) -> Option<Duration> {
    let results = match event.use_average() {
        true => &competitor_json.rank.averages,
        false => &competitor_json.rank.singles,
    };
    results
        .iter()
        .filter(|pr| pr.eventId == event.code_name())
        .map(|pr| pr.best)
        .map(|time| Duration::new(time as u64 / 100, (time % 100) * 10 * 1000 * 1000))
        .next()
}
