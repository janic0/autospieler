use chrono::Datelike;
use office::{cancel_event, update_event_time};
use reqwest::blocking::Client;
use std::env;
pub mod office;

#[derive(PartialEq, Eq)]
enum Attendance {
    ACCEPTED,
    UNSURE,
    DECLINED,
}

fn parse_sp_timestring(input: &str) -> Option<String> {
    if input == "-:-" {
        return None;
    }

    let toplevel_parts: Vec<&str> = input.split_whitespace().collect();
    let time_parts: Vec<&str> = toplevel_parts[0].split(":").collect();

    let mut hours: u8 = time_parts[0].parse().ok()?;

    match toplevel_parts[1] {
        "AM" => hours = hours,
        "PM" => hours = hours + 12,
        _ => return None,
    }

    Some(format!("{:02}:{}", hours, time_parts[1]))
}

fn set_attendence(
    client: &Client,
    user_id: &str,
    event_id: &str,
    event_type: &str,
    reason: &str,
    participation_type: Attendance,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = client
        .post("https://www.spielerplus.de/events/ajax-participation-form")
        .form(&[
            (
                "Participation[participation]",
                match participation_type {
                    Attendance::ACCEPTED => "1",
                    Attendance::UNSURE => "2",
                    Attendance::DECLINED => "0",
                },
            ),
            ("Participation[reason]", reason),
            ("Participation[type]", event_type),
            ("Participation[typeid]", event_id),
            ("Participation[user_id]", user_id),
        ])
        .send()?;

    if res.status() != reqwest::StatusCode::from_u16(200).unwrap() {
        return Err("/events/ajax-participation-form response status is not '200 OK'".into());
    }

    println!("{}", res.status());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entra_client_id = env::var("ENTRA_CLIENT_ID")
        .map_err(|e| format!("Could not read environment variable ENTRA_CLIENT_ID: {e}"))?;
    let entra_client_secret = env::var("ENTRA_CLIENT_SECRET")
        .map_err(|e| format!("Could not read environment variable ENTRA_CLIENT_SECRET: {e}"))?;
    let entra_tenant_id = env::var("ENTRA_TENANT_ID")
        .map_err(|e| format!("Could not read environment variable ENTRA_TENANT_ID: {e}"))?;
    let outlook_user_principal_name = env::var("OUTLOOK_USER_PRINCIPAL_NAME").map_err(|e| {
        format!("Could not read environment variable OUTLOOK_USER_PRINCIPAL_NAME: {e}")
    })?;
    let outlook_calendar_id = env::var("OUTLOOK_CALENDAR_ID")
        .map_err(|e| format!("Could not read environment variable OUTLOOK_CALENDAR_ID: {e}"))?;

    let microsoft_token =
        office::get_microsoft_token(&entra_client_id, &entra_client_secret, &entra_tenant_id)
            .unwrap();

    let current_date = chrono::Utc::now();
    let current_year = current_date.year();
    let last_month = current_date.month() - 1;
    let date_string = format!(
        "{}-{}-{}",
        current_year,
        current_date.month(),
        current_date.day()
    );

    let user_id_variable = env::var("DAUERZUSAGE_ID")
        .map_err(|e| format!("Could not read environment variable DAUERZUSAGE_ID: {e}"))?;
    let user_ids = user_id_variable.split(",").collect::<Vec<&str>>();
    let user_mail = env::var("DAUERZUSAGE_EMAIL")
        .map_err(|e| format!("Could not read environment variable DAUERZUSAGE_EMAIL: {e}"))?;
    let user_password = env::var("DAUERZUSAGE_PASSWORT")
        .map_err(|e| format!("Could not read environment variable DAUERZUSAGE_PASSWORT: {e}"))?;

    let outlook_events = office::list_outlook_events(
        &outlook_user_principal_name,
        &outlook_calendar_id,
        &date_string,
        &microsoft_token,
        &user_mail,
    )?;

    let url = "https://www.spielerplus.de/events";

    log::info!("Fetching {:?}...", url);

    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()?;

    let res = client.get(url).send()?;
    log::info!("/events response: {:?} {}", res.version(), res.status());

    let text = res.text()?;

    let document_events = scraper::Html::parse_document(&text);

    let title_selector = scraper::Selector::parse("title").unwrap();

    let mut titles = document_events
        .select(&title_selector)
        .map(|x| x.inner_html());

    //log::info!("{:?}", titles.next().map(|x| x.inner_html()));
    let mut title = titles.next().ok_or("missing title")?;
    if title == "Einloggen" {
        log::info!("Login required");
        let csrf_token_selector =
            scraper::Selector::parse("form#login-form input[name=\"_csrf\"]").unwrap();
        let csrf_token_element = document_events
            .select(&csrf_token_selector)
            .next()
            .ok_or("missing csrf token")?;
        let csrf_token = csrf_token_element
            .value()
            .attr("value")
            .ok_or("missing value in csrf token")?;
        let res = client
            .post("https://www.spielerplus.de/site/login")
            .form(&[
                ("_csrf", csrf_token),
                ("LoginForm[email]", &user_mail),
                ("LoginForm[password]", &user_password),
            ])
            .send()?;

        log::info!("/site/login response: {:?} {}", res.version(), res.status());

        let text = res.text()?;
        let document = scraper::Html::parse_document(&text);
        let mut titles = document.select(&title_selector).map(|x| x.inner_html());
        if let Some(title) = titles.next() {
            if title == "Team auswählen" || title == "Select team" {
                // alright!
            } else {
                return Err(format!(
                    "title is not 'Team auswählen' or 'Select team', but '{}'",
                    title
                )
                .into());
            }
        } else {
            return Err("title missing".into());
        }
    }

    // Defining selectors
    let event_selector = scraper::Selector::parse(".event").unwrap();
    let panel_selector = scraper::Selector::parse(".panel").unwrap();

    let panel_heading_text_selector = scraper::Selector::parse(".panel-heading-text").unwrap();
    let panel_heading_info_selector = scraper::Selector::parse(".panel-heading-info").unwrap();
    let panel_event_time_item_value_selector =
        scraper::Selector::parse(".event-time-value").unwrap();
    let panel_title_selector = scraper::Selector::parse(".panel-title").unwrap();
    let panel_subtitle_selector = scraper::Selector::parse(".panel-subtitle").unwrap();
    let participation_widget_buttons_selector =
        scraper::Selector::parse(".participation-widget-buttons").unwrap();
    let selected_selector = scraper::Selector::parse(".selected").unwrap();

    // let events = user_ids.iter().filter_map(
    //     |user_id: &&str| -> Result<Vec<EnrichedHTMLEvent>, Box<dyn Error>> {
    //         if user_id.starts_with("pat") {
    //             return Ok(vec![EnrichedHTMLEvent { user_id: user_id }]);
    //         }
    //         Err("i'm a tree".into())
    //     },
    // );

    // Vec<EnrichedHTMLEvent>

    let mut handled_training_ids = Vec::new();

    for user_id in user_ids {
        client
            .get(format!(
                "https://www.spielerplus.de/site/switch-user?id={user_id}"
            ))
            .send()?;

        let res = client.get(url).send()?;
        let text = res.text()?;
        let document_events = scraper::Html::parse_document(&text);

        let mut titles = document_events
            .select(&title_selector)
            .map(|x| x.inner_html());

        title = titles.next().ok_or("missing title")?;

        if title != "Termine" && title != "Events" {
            return Err(format!("title is not 'Termine' or 'Events', but '{}'", title).into());
        }

        let events = document_events.select(&event_selector);

        for event in events {
            log::debug!("Handling event {}", event.inner_html());
            let panel = event
                .select(&panel_selector)
                .next()
                .ok_or("missing .panel")?;
            let heading_text = event
                .select(&panel_heading_text_selector)
                .next()
                .ok_or("missing .panel-heading-text")?;
            let event_title_html = heading_text
                .select(&panel_title_selector)
                .next()
                .ok_or("missing .panel-title")?
                .inner_html();

            // set subtitle or default to empty string
            let event_subtitle_html = heading_text
                .select(&panel_subtitle_selector)
                .next()
                .and_then(|v| Some(v.inner_html()))
                .or(Some("".into()))
                .unwrap();

            let heading_info = event
                .select(&panel_heading_info_selector)
                .next()
                .ok_or("missing .panel-heading-info")?;
            let event_date_html = heading_info
                .select(&panel_subtitle_selector)
                .next()
                .ok_or("missing .panel-subtitle")?
                .inner_html();
            let widget_buttons = event
                .select(&participation_widget_buttons_selector)
                .next()
                .ok_or("missing .participation-widget-buttons")?;
            let event_time_values: Vec<String> = panel
                .select(&panel_event_time_item_value_selector)
                .map(|time| time.inner_html())
                .collect();

            let event_start_ts = parse_sp_timestring(&event_time_values[0])
                .or_else(|| parse_sp_timestring(&event_time_values[1]))
                .ok_or("no event start found")?;

            let event_type_parts = panel
                .value()
                .id()
                .ok_or("no panel id found")?
                .split("-")
                .collect::<Vec<_>>();

            let event_type_sp = *event_type_parts
                .get(1)
                .ok_or("panel id is misformed (no event on position 1)")?;

            let training_id = *event_type_parts.get(2).ok_or("no id found in event id")?;

            handled_training_ids.push(training_id.to_string());

            let event_end_ts =
                parse_sp_timestring(&event_time_values[2]).ok_or("no event end found")?;

            let event_date_parts: Vec<&str> = event_date_html.split("/").collect();
            let event_date_month: u8 = event_date_parts[1].parse()?;
            let event_date_year: i32 = match last_month <= event_date_month.into() {
                true => current_year,
                false => current_year + 1,
            };

            let event_start_ts_iso = format!(
                "{}-{}-{}T{}:00",
                event_date_year, event_date_parts[1], event_date_parts[0], event_start_ts
            );
            let event_end_ts_iso = format!(
                "{}-{}-{}T{}:00",
                event_date_year, event_date_parts[1], event_date_parts[0], event_end_ts
            );

            println!(
                "{}-{} {} ({})",
                event_start_ts_iso, event_end_ts_iso, event_title_html, event_type_sp
            );

            match outlook_events.get(training_id) {
                Some(event) => {
                    if !event.start.dateTime.starts_with(&event_start_ts_iso)
                        || !event.end.dateTime.starts_with(&event_end_ts_iso)
                    {
                        update_event_time(
                            &outlook_user_principal_name,
                            &outlook_calendar_id,
                            &microsoft_token,
                            &event.id,
                            &event_start_ts_iso,
                            &event_end_ts_iso,
                        )?;
                    }

                    let outlook_event_attendence = event
                        .attendees
                        .last()
                        .ok_or("no attendees found")?
                        .status
                        .response
                        .as_str();
                    if outlook_event_attendence == "none" {
                    } else {
                        let new_attendance = match outlook_event_attendence {
                            "accepted" => Attendance::ACCEPTED,
                            "declined" => Attendance::DECLINED,
                            "tentativelyAccepted" | _ => Attendance::UNSURE,
                        };

                        let needs_update = match widget_buttons.select(&selected_selector).next() {
                            Some(selected_button) => {
                                let previous_attendance = match selected_button
                                    .value()
                                    .attr("title")
                                    .ok_or("missing selected_button attr 'title'")?
                                {
                                    "Zugesagt" | "Confirmed" => Attendance::ACCEPTED,
                                    "Unsicher" | "Unsure" => Attendance::UNSURE,
                                    "Absagen / Abwesend" | "Declined / Absent" => {
                                        Attendance::DECLINED
                                    }
                                    other => {
                                        return Err(format!(
                                            "Unknown selected Zusage button title '{other}'"
                                        )
                                        .into());
                                    }
                                };

                                new_attendance != previous_attendance
                            }
                            None => true,
                        };

                        if needs_update {
                            set_attendence(
                                &client,
                                &user_id,
                                &training_id.to_string(),
                                event_type_sp,
                                "-",
                                new_attendance,
                            )?;
                        }
                    }
                }
                None => {
                    office::create_outlook_event(
                        &outlook_user_principal_name,
                        &outlook_calendar_id,
                        &microsoft_token,
                        &event_title_html,
                        "New training found in Spielerplus. Please accept/decline this event.",
                        &event_start_ts_iso,
                        &event_end_ts_iso,
                        &event_subtitle_html,
                        &user_mail,
                        training_id,
                    )?;
                }
            }
        }
    }

    for event in outlook_events {
        if handled_training_ids.contains(&event.0) {
            continue;
        }
        println!("didn't handle {}, deleting...", event.0);
        cancel_event(
            &outlook_user_principal_name,
            &outlook_calendar_id,
            &microsoft_token,
            &event.1.id,
        )?;
    }

    Ok(())
}
