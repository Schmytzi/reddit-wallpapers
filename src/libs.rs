extern crate reqwest;
extern crate serde_json;

pub mod reddit {
    use reqwest;
    use serde_json;
    use serde_json::{Map, Value};
    use std::error::Error;

    fn get_subreddit(subreddit: &str) -> Result<Value, String> {
        let mut url = String::from("https://reddit.com/r/");
        url.push_str(subreddit);
        url.push_str(".json");
        let mut response = match reqwest::get(&url) {
            Ok(mut r) => r,
            Err(e) => return Err(e.description().to_owned()),
        };
        println!("Parsing JSON");
        match response.json() {
            Ok(v) => Ok(v),
            Err(e) => { println!("{:?}", e);
                Err(e.description().to_owned())},
        }
    }

    #[derive(Debug)]
    pub struct Link {
        pub url: String,
        pub title: String,
        pub author: String,
    }

    pub fn get_subreddit_links(subreddit: &str) -> Result<Vec<Link>, String> {
        let body = match get_subreddit(subreddit) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };
        println!("Parsed JSON");
        let listing = match body.as_object() {
            Some(list) => match list.get("data") {
                Some(d) => d.as_object().unwrap(),
                None => return Err("There was no data".to_owned()),
            },
            None => return Err("There was no listing".to_owned()),
        };
        let children = match listing.get("children") {
            Some(c) => c.as_array().expect("children was not an array"),
            None => return Err("Did not find children element".to_owned()),
        };
        let mut result = Vec::new();
        for child in children {
            // Skip children that don't match expectations
            let elements: &Map<String, Value> = match child.as_object() {
                Some(obj) => match obj.get("data") {
                    Some(d) => match d.as_object() {
                        Some(d_obj) => d_obj,
                        None => continue,
                    },
                    None => continue,
                },
                None => continue,
            };
            let url = match elements.get("url") {
                Some(u) => match u.as_str() {
                    Some(s) => s,
                    None => continue,
                },
                None => continue,
            };
            let title = match elements.get("title") {
                Some(u) => match u.as_str() {
                    Some(s) => s,
                    None => continue,
                },
                None => continue,
            };
            let author = match elements.get("author") {
                Some(u) => match u.as_str() {
                    Some(s) => s,
                    None => continue,
                },
                None => continue,
            };
            result.push(Link {
                url: url.to_owned(),
                title: title.to_owned(),
                author: author.to_owned(),
            });
        }
        Ok(result)
    }

}
