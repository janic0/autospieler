use std::{collections::HashMap, error::Error};

#[derive(serde::Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
}

pub fn get_microsoft_token<'a>(
    client_id: &str,
    client_secret: &str,
    tenant_id: &str,
) -> Result<String, Box<dyn Error>> {
    let response = reqwest::blocking::Client::new()
        .post(format!(
            "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token",
        ))
        .form(&[
            ("grant_type", "client_credentials"),
            ("scope", "https://graph.microsoft.com/.default"),
        ])
        .basic_auth(client_id, Some(client_secret))
        .send()?;

    let response_status = response.status().as_u16();
    if response_status != 200 {
        let response_text = response.text()?;
        return Err(
            format!("request failed with status code {response_status}\n{response_text}").into(),
        );
    }

    let data: MicrosoftTokenResponse = response.json::<MicrosoftTokenResponse>()?;
    Ok(data.access_token)
}

pub fn create_outlook_event(
    user_principal_name: &str,
    calendar_id: &str,
    access_token: &str,
    event_name: &str,
    body: &str,
    start_datetime: &str,
    end_datetime: &str,
    location_name: &str,
    email_address: &str,
    spielerplus_id: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let response = reqwest::blocking::Client::new()
        .post(format!(
            "https://graph.microsoft.com/v1.0/users/{}/calendars/{}/events",
            user_principal_name, calendar_id,
        ))
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "subject": event_name,
                "body": {
                    "contentType": "text",
                    "content": body
                },
                "start": {
                    "dateTime": start_datetime,
                    "timeZone": "Europe/Zurich"
                },
                "end": {
                    "dateTime": end_datetime,
                    "timeZone": "Europe/Zurich"
                },
                "location": {
                    "displayName": location_name
                },
                "singleValueExtendedProperties": [
                    {
                        "id": "String {78ba6e36-6d32-4157-83aa-e36c77df0418} Name SP_ID",
                        "value": spielerplus_id,
                    },
                    {
                        "id": "String {abdd660a-9ce1-4aa3-b4e7-eca89ccfedea} Name SP_USER_EMAIL",
                        "value": email_address,
                    },
                ],
                "attendees": [
                    {
                        "emailAddress": {
                            "address": email_address,
                            "name": email_address
                        },
                        "type": "required"
                    }
                ]
            })
            .to_string(),
        )
        .bearer_auth(access_token)
        .send()?;

    if response.status() != reqwest::StatusCode::from_u16(201).unwrap() {
        return Err(format!("request failed with status code {}", response.status()).into());
    }

    Ok(response.json()?)
}

#[derive(serde::Deserialize)]
pub struct MicrosoftGetEventsResponseEventTimestamp {
    pub dateTime: String,
    pub timeZone: String,
}

#[derive(serde::Deserialize)]
pub struct MicrosoftGetEventsResponseEventAttendeeStatus {
    pub response: String,
    pub time: String,
}

#[derive(serde::Deserialize)]
pub struct MicrosoftGetEventsResponseEventAttendee {
    pub r#type: String,
    pub status: MicrosoftGetEventsResponseEventAttendeeStatus,
}
#[derive(serde::Deserialize)]
pub struct SingleValueExtendedProperties {
    pub id: String,
    pub value: String,
}
#[derive(serde::Deserialize)]
pub struct MicrosoftGetEventsResponseEvent {
    pub id: String,
    pub subject: String,
    pub start: MicrosoftGetEventsResponseEventTimestamp,
    pub end: MicrosoftGetEventsResponseEventTimestamp,
    pub singleValueExtendedProperties: Vec<SingleValueExtendedProperties>,
    pub attendees: Vec<MicrosoftGetEventsResponseEventAttendee>,
}

#[derive(serde::Deserialize)]
pub struct MicrosoftGetEventsResponse {
    value: Vec<MicrosoftGetEventsResponseEvent>,
}

pub type ProcessedOutlookEventMap = HashMap<String, MicrosoftGetEventsResponseEvent>;

pub fn list_outlook_events(
    user_principal_name: &str,
    calendar_id: &str,
    current_date: &str,
    access_token: &str,
    email_address: &str,
) -> Result<ProcessedOutlookEventMap, Box<dyn Error>> {
    let response = reqwest::blocking::Client::new()
        .get(format!(
            "https://graph.microsoft.com/v1.0/users/{}/calendars/{}/events",
            user_principal_name, calendar_id,
        ))
        .bearer_auth(access_token)
        .header("Prefer", "outlook.timezone=\"Europe/Zurich\"")
        .query(
            &[
                ("$expand", "singleValueExtendedProperties($filter=id eq 'String {78ba6e36-6d32-4157-83aa-e36c77df0418} Name SP_ID')"),
                ("$filter", &format!("singleValueExtendedProperties/Any(ep: ep/id eq 'String {{abdd660a-9ce1-4aa3-b4e7-eca89ccfedea}} Name SP_USER_EMAIL' and ep/value eq '{}') and start/dateTime ge '{}'", email_address, current_date)),
                ("$select", "id, subject, singleValueExtendedProperties, start, end, attendees")
            ]
        )
        .send()?;

    let response_status = response.status().as_u16();
    if response_status != 200 {
        let response_text = response.text()?;

        return Err(format!(
            "/events request failed with status code {response_status}\n {response_text}"
        )
        .into());
    }

    let data = response.json::<MicrosoftGetEventsResponse>()?;

    let mut processed_outlook_event_map = HashMap::new();
    for event in data.value {
        processed_outlook_event_map
            .insert(event.singleValueExtendedProperties[0].value.clone(), event);
    }

    Ok(processed_outlook_event_map)
}
pub fn update_event_time(
    user_principal_name: &str,
    calendar_id: &str,
    access_token: &str,
    event_id: &str,
    new_start_time: &str,
    new_end_time: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let response = reqwest::blocking::Client::new()
        .patch(format!(
            "https://graph.microsoft.com/v1.0/users/{}/calendars/{}/events/{}",
            user_principal_name, calendar_id, event_id
        ))
        .header("content-type", "application/json")
        .body(
            serde_json::json!({

                "start": {
                    "dateTime": new_start_time,
                    "timeZone": "Europe/Zurich"
                },
                "end": {
                    "dateTime": new_end_time,
                    "timeZone": "Europe/Zurich"
                },
            })
            .to_string(),
        )
        .bearer_auth(access_token)
        .send()?;

    if response.status() != reqwest::StatusCode::from_u16(200).unwrap() {
        return Err(format!("request failed with status code {}", response.status()).into());
    }

    Ok(response.json()?)
}

pub fn cancel_event(
    user_principal_name: &str,
    calendar_id: &str,
    access_token: &str,
    event_id: &str,
) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::Client::new()
        .post(format!(
            "https://graph.microsoft.com/v1.0/users/{}/calendars/{}/events/{}/cancel",
            user_principal_name, calendar_id, event_id
        ))
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "comment": "This event no longer exists in Spielerplus."
            })
            .to_string(),
        )
        .bearer_auth(access_token)
        .send()?;

    if response.status() != reqwest::StatusCode::from_u16(202).unwrap() {
        return Err(format!("request failed with status code {}", response.status()).into());
    }

    Ok(())
}
