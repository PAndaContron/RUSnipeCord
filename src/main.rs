mod dat;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

use log::{debug, info, warn, error};
use reqwest::Url;
use tokio::time::{Duration, interval, MissedTickBehavior};
use webhook::client::WebhookClient;

use dat::{Config, Course};

const USERNAME: &'static str = "RU SnipeCord";
const AVATAR: &'static str = "https://upload.wikimedia.org/wikipedia/commons/thumb/1/1c/Rifle_scope.svg/240px-Rifle_scope.svg.png";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize logging
    env_logger::init();

    // Read config file
    let mut cfg_file = File::open("config.json").expect("Failed to open config.json");
    let mut config = String::new();
    cfg_file.read_to_string(&mut config).expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config).expect("Failed to parse config.json");
    info!("Parsed config: {:?}", config);

    // Initialize request params
    let params = [
        ("year", &config.year),
        ("term", &config.term),
        ("campus", &config.campus),
        ("level", &config.level),
    ];

    // Initialize reqwest client
    let client = reqwest::Client::new();

    // Get class metadata (used for display purposes)
    let url = Url::parse_with_params("https://sis.rutgers.edu/soc/courses.gz", &params).expect("Failed to parse URL");
    let meta: Vec<Course> = client
        .get(url)
        .send()
        .await
        .expect("Failed to request course data")
        .json()
        .await
        .expect("Failed to parse course data");
    info!("Parsed metadata");
    debug!("Sample of metadata: {:?}", &meta[0..5]);

    // Map of index -> (display name, timeout)
    let mut section_meta = HashMap::new();
    for index in &config.indexes {
        section_meta.insert(index.clone(), ("".to_owned(), 0u32));
    }
    for course in &meta {
        for section in &course.sections {
            if let Some((val, _)) = section_meta.get_mut(&section.index) {
                if *val != "" {
                    warn!("Same index found twice in CSP data; first {}, then {} Section {}; using last one",
                          *val, course.title, section.number);
                }
                *val = format!("{} Section {} (Index {})", course.title, section.number, section.index);
            }
        }
    }
    for (index, (val, _)) in section_meta.iter_mut() {
        if *val == "" {
            error!("Class data for index {} not found; we'll track it anyway but this index is almost definitely not valid", index);
            *val = format!("Unknown Class (Index {})", index);
        }
    }
    debug!("Section metadata: {:?}", section_meta);

    // We don't need the old metadata anymore,
    // and it takes up a ton of space so we should get rid of it
    drop(meta);

    // Initialize webhook
    let webhook = WebhookClient::new(&config.webhook);
    if let Err(err) = webhook.send(|message| {
        message
            .username(USERNAME)
            .avatar_url(AVATAR)
            .content("Ready for action!!")
    }).await {
        error!("Failed to send message through webhook: {:?}", err);
    }

    // Start sniping
    let url = Url::parse_with_params("https://sis.rutgers.edu/soc/openSections.gz", &params).expect("Failed to parse URL");
    let mention = config.mention.map_or("".to_owned(), |m| m + "\n");

    let mut clock = interval(Duration::from_secs(1));
    clock.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        // Wait for the next tick
        clock.tick().await;

        // Get the open sections
        let open_sections = client
            .get(url.clone())
            .send()
            .await;
        let open_sections = match open_sections {
            Err(err) => {
                error!("Failed to query open sections: {:?}", err);
                continue;
            }
            Ok(c) => c
        }.json().await;
        let open_sections: Vec<String> = match open_sections {
            Err(err) => {
                error!("Failed to parse open sections: {:?}", err);
                continue;
            }
            Ok(c) => c
        };

        // Check if any of the courses we're tracking are in here
        let mut seen = HashSet::new();

        for index in &open_sections {
            if let Some((val, time)) = section_meta.get_mut(index) {
                seen.insert(index);
                if *time == 0 {
                    // Print the message to both the log and the discord webhook, just in case
                    let msg = format!("The course {} is open!!! Register with \
                                      http://sims.rutgers.edu/webreg/editSchedule.htm?login=cas&semesterSelection={}{}&indexList={}",
                                      val, config.term, config.year, index);
                    info!("{}", msg);
                    if let Err(err) = webhook.send(|message| {
                        message
                            .username(USERNAME)
                            .avatar_url(AVATAR)
                            .content(&format!("{}\n{}", mention, msg))
                    }).await {
                        error!("Failed to send message through webhook: {:?}", err);
                    }
                    *time = config.repeat_timeout;
                } else {
                    *time -= 1;
                }
            }
        }

        for (index, (_, time)) in section_meta.iter_mut() {
            if !seen.contains(index) {
                *time = 0;
            }
        }
    }
}
