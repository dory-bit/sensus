use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::fs;
use chrono::{Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleEvent {
    pub summary: Option<String>,
    pub start: GoogleEventTime,
    pub end: GoogleEventTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleEventTime {
    pub date_time: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCalendarResponse {
    pub items: Option<Vec<GoogleEvent>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
}

pub struct GoogleCalendarClient {
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub token_path: String,
}

impl GoogleCalendarClient {
    pub fn new(token_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let token_content = match fs::read_to_string(token_path) {
            Ok(content) => content,
            Err(_) => return Err("AUTH_REQUIRED".into()),
        };
        let token_json: serde_json::Value = serde_json::from_str(&token_content).map_err(|_| "TOKEN_MALFORMED")?;
        
        Ok(Self {
            access_token: token_json["token"].as_str().ok_or("Token not found")?.to_string(),
            refresh_token: token_json["refresh_token"].as_str().ok_or("Refresh token not found")?.to_string(),
            client_id: token_json["client_id"].as_str().ok_or("Client ID not found")?.to_string(),
            client_secret: token_json["client_secret"].as_str().ok_or("Client secret not found")?.to_string(),
            token_path: token_path.to_string(),
        })
    }

    pub async fn refresh_access_token(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", self.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ];

        let resp = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        self.access_token = resp.access_token;

        let mut token_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&self.token_path)?)?;
        token_json["token"] = serde_json::Value::String(self.access_token.clone());
        fs::write(&self.token_path, serde_json::to_string_pretty(&token_json)?)?;

        Ok(())
    }

    pub async fn fetch_events(&self) -> Result<Vec<GoogleEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let url = "https://www.googleapis.com/calendar/v3/calendars/primary/events";
        
        let now = Utc::now();
        let time_min = now.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339();
        
        let tomorrow = now + chrono::Duration::days(1);
        let time_max = tomorrow.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339();

        println!("Sensus API: Buscando eventos entre {} e {}", time_min, time_max);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token))?,
        );

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .headers(headers)
            .query(&[
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
                ("timeMin", &time_min),
                ("timeMax", &time_max),
            ])
            .send()
            .await?;

        println!("Sensus API: Resposta da API: {}", response.status());

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err("TOKEN_EXPIRED".into());
        }

        let body = response.json::<GoogleCalendarResponse>().await?;
        let count = body.items.as_ref().map(|v| v.len()).unwrap_or(0);
        println!("Sensus API: {} eventos encontrados no período.", count);
        
        Ok(body.items.unwrap_or_default())
    }
}
