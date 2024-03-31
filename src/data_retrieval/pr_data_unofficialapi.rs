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
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct PR {
    best: u32,
    eventId: String,
}

pub fn retrieve_competitor_prs(competitor: &mut Competitor) -> Result<(), WCOError> {
    if let Some(id) = &mut competitor.wca_id {
        let url = format!("https://raw.githubusercontent.com/robiningelbrecht/wca-rest-api/master/api/persons/{}.json", id);
        let json: Person = serde_json::from_str(&get(url)?.text()?)?;
        if let Some(avg) = parse_pr_3x3_avg_json(&json) {
            competitor.personal_records.insert(Event::Ev333, avg);
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
