use crate::datastructures::Competitor;
use crate::wcoerror::WCOError;
use regex::Regex;
use scraper::{selectable::Selectable, Html, Selector};

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
