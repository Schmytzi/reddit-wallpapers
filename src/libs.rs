extern crate reqwest;
extern crate serde_json;

use serde_json::{Value, Error, Map};
pub mod reddit {
     fn get_subreddit(subreddit: &str) -> Value {
         let mut url = String::from("https://reddit.com/r/");
         url.push_str(subreddit);
         url.push_str(".json");
         let response = reqwest::get(&url).unwrap().text().unwrap();
         serde_json::from_str(&response)
     }

     pub struct Link {
         url: String,
         title: String,
         author: String,
         created: u64,
     }

     pub fn get_subreddit_links(subreddit: &str) -> Vec<Link> {
         let body = get_subreddit(subreddit);
         let listing = body.as_object().unwrap();
         let children = listing.get("children").unwrap().as_array().unwrap();
         let mut result = Vec::new();
         for child in children {
             let elements: &Map<String, Value> = child.as_object().unwrap();
             result.push(Link {
                 url: String::from(elements.get("url").unwrap().as_str().unwrap()),
                 title: String::from(elements.get("title").unwrap().as_str().unwrap()),
                 author: String::from(elements.get("author").unwrap().as_str().unwrap()),
                 created: elements.get("created").unwrap().as_u64().unwrap(),
             });
         }
         result
     }

}