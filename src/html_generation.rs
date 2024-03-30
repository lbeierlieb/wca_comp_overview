use crate::datastructures::Competitor;
use maud::html;
use std::time::Duration;

pub fn generate_report_html(competition_title: &str, competitor_data: &[Competitor]) -> String {
    let mut competitors_no_id = vec![];
    let mut competitors_no_time = vec![];
    let mut competitors_time = vec![];
    for comp in competitor_data {
        match comp {
            Competitor {
                name: _,
                wca_id: None,
                pr_3x3_avg: _,
                ..
            } => competitors_no_id.push(comp),
            Competitor {
                name: _,
                wca_id: Some(_),
                pr_3x3_avg: None,
                ..
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
                    img src="histogram.png" {}
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

fn format_time(time: &Duration) -> String {
    let subsec = time.subsec_millis() / 10;
    let sec = time.as_secs() % 60;
    let min = time.as_secs() / 60;
    match min {
        0 => format!("{}.{:0>2}", sec, subsec),
        _ => format!("{}:{:0>2}.{:0>2}", min, sec, subsec),
    }
}
