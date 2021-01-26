use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches};
use flate2::read::GzDecoder;
use log::{error, info};
use reqwest;
use serde_json::Value;
use simple_logger::SimpleLogger;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use tar::Archive;
use tokio;

use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    let m = requirements();
    let url = m.value_of("url").unwrap().to_string();
    Url::parse(&url)?;
    let token = m.value_of("token").unwrap().to_string();
    let policy_path = m.value_of("policy_path").unwrap().to_string();
    if Path::new(&policy_path).exists() == false {
        error!(
            "The provided path {} does not exist. Exiting...",
            policy_path
        );
        process::exit(1);
    }
    info!("The provided path {} does exist.", policy_path);
    let (id_vector, _ret) = list_projects(url.clone(), token.clone()).await;
    let (download_url_vector, _ret) =
        list_packages_per_project(id_vector, url.clone(), token.clone()).await;
    download_bundle(download_url_vector, token.clone(), policy_path).await?;
    Ok(())
}

fn requirements() -> ArgMatches<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("url")
                .long("url")
                .env("URL")
                .help("Default value from env var URL")
                .required(true),
        )
        .arg(
            Arg::with_name("token")
                .long("token")
                .env("TOKEN")
                .help("Default value from env var TOKEN.")
                .required(true),
        )
        .arg(
            Arg::with_name("policy_path")
                .long("policy_path")
                .env("POLICY_PATH")
                .help("Default value from env var POLICY_PATH.")
                .required(true),
        )
        .get_matches()
}

async fn list_packages_per_project(
    id_vector: Vec<i32>,
    url: String,
    token: String,
) -> (Vec<String>, Result<(), reqwest::Error>) {
    let client = reqwest::Client::new();
    let mut download_url_vector: Vec<String> = Vec::new();

    for i in 0..id_vector.len() {
        let url_packages = format!("{}/api/v4/projects/{}/packages", url, id_vector[i]);
        let res = client
            .get(&url_packages)
            .header("PRIVATE-TOKEN", &token)
            .send()
            .await
            .expect("Failed to list the packages from projects")
            .text_with_charset("utf-8")
            .await
            .expect("Failed to list the packages from projects");

        let v: Vec<Value> = serde_json::from_str(&res).unwrap();
        let vers = v[0].get("version").unwrap().to_string();

        let version = vers.trim_matches('"').to_string();

        let download_url = format!(
            "{}/api/v4/projects/{}/packages/generic/bundle/{}/bundle.tar.gz",
            url, id_vector[i], version
        );
        download_url_vector.push(download_url);
    }
    return (download_url_vector, Ok(()));
}

async fn list_projects(url: String, token: String) -> (Vec<i32>, Result<(), reqwest::Error>) {
    let client = reqwest::Client::new();

    let url = format!("{}/api/v4/projects?per_page=500&sort=asc", url);

    let res = client
        .get(&url)
        .header("PRIVATE-TOKEN", &token)
        .send()
        .await
        .expect("Request failed")
        .text_with_charset("utf-8")
        .await
        .expect("Request failed");

    if res == String::from("{\"message\":\"401 Unauthorized\"}") {
        error!("The provided token is unauthorized. Exiting...");
        process::exit(1);
    }

    let v: Vec<Value> = serde_json::from_str(&res).unwrap();

    if v.len() == 0 {
        error!(
            "The provided token has access to {} projects, expected at least 1. Exiting...",
            v.len()
        );
        process::exit(1);
    }

    let mut id_vector: Vec<i32> = Vec::new();

    for i in &v {
        let id = i.get("id").unwrap().to_string();
        let my_int = id.parse::<i32>().unwrap();
        id_vector.push(my_int);
    }

    info!(
        "The provided token has access to {} projects.",
        id_vector.len()
    );

    // println!("THIS IS THE LENGHT::::   {}",id_vector.len());
    // let check = id_vector.len();

    // if &id_vector.len() = 0 {
    //     info!("The provided token has access to {} projects. Exiting...",&id_vector.len());
    // else {
    //     info!("The provided token has access to {} projects",&id_vector.len());
    // }

    return (id_vector, Ok(()));
}

async fn download_bundle(
    download_url_vector: Vec<String>,
    token: String,
    policy_path: String,
) -> Result<()> {
    let client = reqwest::Client::new();
    let file_path = format!("{}/bundle.tar.gz", policy_path);
    println!("{}", file_path);

    for url in download_url_vector {
        let response = client
            .get(&url)
            .header("PRIVATE-TOKEN", &token)
            .send()
            .await?
            .bytes()
            .await?;
        let mut file = File::create(&file_path).expect("Creating file failed");
        let data: Result<Vec<_>, _> = response.bytes().collect();
        let data = data.expect("Unable to read data");
        file.write_all(&data).expect("Unable to write data");
        file.write_all(&data).expect("Writing to the file failed");
        let tar_gz = File::open(&file_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(&policy_path)?;
    }
    Ok(())
}
