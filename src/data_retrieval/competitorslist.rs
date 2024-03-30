use crate::datastructures::{Competitor, Event};
use crate::wcoerror::WCOError;
use regex::Regex;
use scraper::{selectable::Selectable, ElementRef, Html, Selector};

pub fn get_competition_title(html: &Html) -> Result<String, WCOError> {
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

pub fn parse_competitors(competitors_list: &Html) -> Vec<Competitor> {
    let selector =
        Selector::parse(r#"td[class="name"]"#).expect("Parsing known selector should not fail");
    let name_selector = &Selector::parse("a").expect("Parsing known selector should not fail");
    let event_selector = Selector::parse("i").expect("Parsing known selector should not fail");

    competitors_list
        .select(&selector)
        .map(|element| {
            let name = element.text().collect::<String>().trim().to_owned();
            let wca_id = element
                .select(&name_selector)
                .next()
                .and_then(|elem| elem.value().attr("href").and_then(parse_wca_id));
            let all_classes = element
                .next_siblings()
                .filter_map(|node| ElementRef::wrap(node))
                .flat_map(|elem| {
                    elem.select(&event_selector)
                        .flat_map(|elem| elem.value().classes().collect::<Vec<_>>())
                })
                .collect::<Vec<_>>();
            let events = extract_events(&all_classes);
            Competitor::new(name, wca_id, events)
        })
        .collect()
}

fn parse_wca_id(profile_url: &str) -> Option<String> {
    let re = Regex::new(r"/persons/([0-9]{4}[A-Z]{4}[0-9]{2})").unwrap();
    re.captures(profile_url).map(|cap| cap[1].to_owned())
}

fn extract_events(strs: &[&str]) -> Vec<Event> {
    let re = Regex::new(r"event-(.+)").unwrap();
    strs.iter()
        .filter_map(|str| re.captures(str))
        .filter_map(|cap| Event::try_from(&cap[1]).ok())
        .collect()
}
