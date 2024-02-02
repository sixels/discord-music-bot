use std::borrow::Cow;

use reqwest::{Client as Http, Url};
use serde::Deserialize;
use tracing::{debug, error};

const PIPED_URL: &str = "https://pipedapi.kavin.rocks";
const USER_AGENT: &str = concat!("piped-rust-sdk/0.0.4");

pub struct PipedClient<'h> {
    instance: Cow<'static, str>,
    http: &'h Http,
}

impl<'h> PipedClient<'h> {
    pub fn new(http: &'h Http) -> Self {
        Self {
            http,
            instance: Cow::Borrowed(PIPED_URL),
        }
    }

    #[allow(dead_code)]
    pub fn set_instance<S: Into<String>>(mut self, url: S) -> Self {
        self.instance = Cow::Owned(url.into());
        self
    }

    pub async fn search_songs(&self, input: &str) -> Result<SearchResults, PipedError> {
        let mut url = Url::parse(format!("{}/search", self.instance).as_str()).expect("bad URL");
        url.query_pairs_mut().append_pair("q", input);
        url.query_pairs_mut()
            .append_pair("filter", "videos");


        debug!("search url: {url:?}");

        let Ok(res) = self
            .http
            .get(url)
            .header("user-agent", USER_AGENT)
            .send()
            .await
        else {
            error!("request to {} failed: ", self.instance);
            return Err(PipedError::Request);
        };

        debug!("search status: {}", res.status());
        let body = res.text().await.unwrap();

        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(err) => {
                error!("results serialization failed: {err:?}");
                debug!("request body: {:?}", body);
                Err(PipedError::Unknown)
            }
        }
    }
}

pub enum PipedError {
    Request,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    pub items: Vec<SearchResult>,
}
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub duration: u64,
    pub title: String,
}
