use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub listen: String,
    pub port: u16,
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wiki {
    pub repository: String,
    pub path: std::path::PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub secret: String,
    pub wiki: Wiki,
    pub blog: Blog,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blog {
    pub repository: String,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IndexPage {
    pub slogan: Option<String>,
    pub title: Option<String>,
    pub skills: Option<Vec<String>>,
    pub github: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Contact {
    pub mail: Option<String>,
    pub matrix: Option<String>,
    pub threema: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub content: Content,
    pub contact: Contact,
    pub index: IndexPage,
}

impl Config {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Config, serde_yaml::Error> {
        let path = path.as_ref();
        let config_content = std::fs::File::open(path).expect("Error loading config file");
        let config_data: Config = serde_yaml::from_reader(config_content)?;

        let server = config_data.server;
        let content = config_data.content;
        let contact = config_data.contact;
        let index = config_data.index;

        Ok(Config {
            server,
            content,
            contact,
            index,
        })
    }
}
