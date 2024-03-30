use crate::datastructures::Competitor;
use crate::wcoerror::WCOError;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::{ops::Add, time::Duration};

pub fn retrieve_competitor_pr_avg_html(competitor: &mut Competitor) -> Result<(), WCOError> {
    if let Some(id) = &mut competitor.wca_id {
        let url = format!("https://www.worldcubeassociation.org/persons/{}", id);
        let html = Html::parse_document(&get(url)?.text()?);
        competitor.pr_3x3_avg = parse_pr_3x3_avg_html(&html)?;
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
