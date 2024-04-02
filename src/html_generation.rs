use crate::datastructures::{Competitor, Event};
use maud::html;
use std::time::Duration;

pub fn generate_event_html(
    competition_title: &str,
    competitor_data: &[Competitor],
    event: &Event,
) -> String {
    let mut competitors_no_id = vec![];
    let mut competitors_no_time = vec![];
    let mut competitors_time = vec![];
    let event_participating_competitors = competitor_data
        .iter()
        .filter(|comp| comp.events.contains(event));
    for comp in event_participating_competitors {
        match comp {
            Competitor { wca_id: None, .. } => competitors_no_id.push(comp),
            Competitor {
                wca_id: Some(_),
                personal_records: hmap,
                ..
            } if !hmap.contains_key(event) => competitors_no_time.push(comp),
            _ => competitors_time.push(comp),
        }
    }
    let num_time = competitors_time.len();
    let num_no_time = competitors_no_time.len();
    let num_no_id = competitors_no_id.len();
    competitors_time.sort_by_key(|comp| comp.personal_records.get(event));
    let mut all_competitors = competitors_time;
    all_competitors.append(&mut competitors_no_time);
    all_competitors.append(&mut competitors_no_id);
    let evname = event.pretty_name();
    let markup = html! {
        html {
            head {
                title { (event.code_name()) "@" (competition_title) }
                link rel="stylesheet" type="text/css" href="styles.css" {}
            }
            body {
                div class="container" {
                    h1 {
                        (competition_title) ": " (evname)
                    }
                    p {
                        "Of the " (competitor_data.len()) " competitors, there is a total of " b { (all_competitors.len()) } " participating in " (evname) " registered. They consists of:"
                    }
                    ul {
                        li {
                            b { (num_time) } ", who have competed in " (evname) " before"
                        }
                        li {
                            b { (num_no_time) } ", who have competed at WCA events before, but not in " (evname)
                        }
                        li {
                            b { (num_no_id) } ", who have never competed at a WCA event before"
                        }
                    }
                    img src=(format!("plots/hist{}.png", event.code_name())) {}
                    table {
                        tr {
                            th {}
                            th {
                                "Competitor"
                            }
                            th {
                                (evname) " PR " ( if event.use_average() { "Average" } else { "Single" })
                            }
                        }
                        @for (rank, competitor) in all_competitors.iter().enumerate() {
                            tr {
                                td { (match &competitor.personal_records.get(event) { Some(_) => (rank+1).to_string(), None => "".to_string()}) }
                                @if let Some(id) = &competitor.wca_id {
                                    td {
                                        a target="_blank" href=(format!("https://www.worldcubeassociation.org/persons/{}", id)) {
                                            (competitor.name)
                                        }
                                    }
                                } @ else {
                                    td { (competitor.name) }
                                }
                                td { (match &competitor.personal_records.get(event) { Some(time) => format_time(time), None => "".to_string()}) }
                            }
                        }
                    }
                }
            }
        }
    };
    markup.into_string()
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

pub fn generate_index_html(
    competition_title: &str,
    events: &[Event],
    competitors: &[Competitor],
) -> String {
    let mut sorted_events: Vec<_> = events.iter().collect();
    sorted_events.sort();
    let competitor_count = competitors.len();
    let event_count = sorted_events.len();
    let returner_count = competitors
        .iter()
        .filter(|comp| comp.wca_id.is_some())
        .count();
    let newcomer_count = competitor_count - returner_count;
    let markup = html! {
        html {
            head {
                title { (competition_title) " - Competitor Overview" }
                link rel="stylesheet" type="text/css" href="styles.css" {}
            }
            body {
                div class="container" {
                    h1 { (competition_title) " - Competitor Overview" }
                    p {
                        "There is a total of " b { (competitor_count) } " competitors registered across " b { (event_count) } " events. "
                        "The competitors consists of " b { (newcomer_count) } " newcomers and " b { (returner_count) } " returners."
                    }
                    p {
                        "The following table gives an overview of the offered events. " i { "Total participant count" } " describes "
                        "how many of the competitors are participating in the respective event. "
                        i { "Available personal records" } " indicates how many of the participants have a WCA profile and a personal "
                        " record for this event. Details about all the PRs for a event can be viewed by clicking on the respective "
                        "event name."
                    }
                    table {
                        tr {
                            th {
                                "Event"
                            }
                            th {
                                "Total participant count"
                            }
                            th {
                                "Available personal records"
                            }
                        }
                        @for event in sorted_events {
                            tr {
                                td {
                                    a href=(format!("{}.html", event.code_name())) {
                                        ( event.pretty_name())
                                    }
                                }
                                td {
                                    (competitors.iter().filter(|comp| comp.events.contains(event)).count())
                                }
                                td {
                                    (competitors.iter().filter(|comp| comp.personal_records.contains_key(event)).count())
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    markup.into_string()
}
