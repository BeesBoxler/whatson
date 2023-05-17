use std::env;
use std::path::Path;
use std::{collections::HashSet, process::exit};

use git2::Repository;
use regex::Regex;
use reqwest::{blocking, Method};
use serde::Deserialize;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CLEAR: &str = "\x1b[0m";

fn main() {
    let token = env::var("JIRA_TOKEN").unwrap_or_default();
    let args: Vec<String> = env::args().collect();
    let target_branch = match args.get(1) {
        Some(i) => i,
        None => {
            eprintln!("{RED}Please provide a branchname.{CLEAR}");
            exit(1);
        }
    };
    let path = match args.get(2) {
        Some(p) => Path::new(p),
        None => Path::new("."),
    };

    if token.is_empty() {
        println!(
            "{RED}JIRA_TOKEN environment variable not set. Showing only ticket keys.{CLEAR}\n"
        );
    }

    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open: {e}"),
    };

    let mut history = repo.revwalk().unwrap();

    history.push_range(&format!("origin/master..origin/{target_branch}")).ok();

    let re = Regex::new(r"^([A-Z]{3,4}-[0-9]+)").unwrap();

    let history: HashSet<_> = history
        .filter_map(|r| {
            let msg = repo
                .find_commit(r.unwrap())
                .unwrap()
                .summary()
                .unwrap()
                .to_string();
            let caps = re.captures(&msg);
            caps.map(|a| a.get(0).unwrap().as_str().to_string())
        })
        .collect();

    let client = blocking::Client::new();

    history.iter().for_each(|key| {
        if !token.is_empty() {
                let request = client
                .request(
                    Method::GET,
                    format!("https://jira.wrenkitchens.com/rest/api/2/issue/{key}?fields=summary,self"),
                )
                .bearer_auth(&token);

                let response = request.send();
                match response {
                    Ok(d) => {
                        let issue = d.json::<Issue>().unwrap();
                        println!(
                            "\t{GREEN}{key}{CLEAR}: {summary}\n\t\thttps://jira.wrenkitchens.com/browse/{key}",
                            key = issue.key,
                            summary = issue.fields.summary
                        );
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }else {
                println!("\t{GREEN}{key}{CLEAR}");
        };
    });
}

#[derive(Deserialize, Debug)]
struct Issue {
    pub fields: Fields,
    pub key: String,
}

#[derive(Deserialize, Debug)]
struct Fields {
    pub summary: String,
}
