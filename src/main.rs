#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

extern crate ansi_term;
extern crate rand;

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::time::Duration;

use ansi_term::Colour::Blue;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

const DEFAULT_GREPTWEETS_URL: &'static str = "http://greptweet.com/u/dril/dril.txt";
const DEFAULT_TWEET_FILE: &'static str = "dril.txt";

fn get_tweets(url: &str, out_file_name: &str) -> io::Result<()> {
    let output = try!(Command::new("curl")
        .arg("--compressed")
        .arg("-s")
        .arg(url)
        .output());

    let mut f = File::create(out_file_name).unwrap();

    try!(f.write(output.stdout.as_slice()));
    Ok(())
}

fn add_tweets_mongo(file_name: &str) -> io::Result<()> {
    let mut buf = String::new();
    let mut f = File::open(file_name).unwrap();
    try!(f.read_to_string(&mut buf));

    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let coll = client.db("dril").collection("tweets");
    for line in buf.lines() {
        let tweet = line.split('|').last().unwrap();
        let doc = doc!{"tweet" => tweet};
        let mut cursor = coll.find(Some(doc), None).unwrap();
        if cursor.count() == 0 {
            // TODO errhand
            coll.insert_one(doc!{"tweet" => tweet}, None).unwrap();
        } else { 
            break;
        }
    }

    Ok(())
}

fn random_tweet(tweet_file: &str) -> io::Result<String> {
    let mut tweets = Vec::new();
    let mut buf = String::new();
    let mut f = File::open(tweet_file).unwrap();
    try!(f.read_to_string(&mut buf));

    for line in buf.lines() {
        let tweet = line.split('|').last().unwrap();
        tweets.push(tweet.to_owned());
    }

    let which_tweet: usize = rand::random::<usize>() % tweets.len();
    Ok(tweets[which_tweet].to_owned())
}

fn random_tweet_mongo() -> io::Result<String> {
    let client = Client::connect("localhost", 27017).ok().expect("Failed to initialize client.");
    let coll = client.db("dril").collection("tweets");

    let mut cursor = coll.find(None, None).unwrap();
    let num_tweets = coll.find(None, None).unwrap().count();

    let which_tweet: usize = rand::random::<usize>() % num_tweets;

    let tweet = cursor.skip(which_tweet).next().unwrap().unwrap();
    Ok(tweet.get_str("tweet").unwrap().to_owned())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let tweets_url: &str;
    let tweets_txt: &str;

    if let (Some(arg1), Some(arg2)) = (args.get(1), args.get(2)) {
        tweets_url = arg1;
        tweets_txt = arg2;
    } else {
        tweets_url = DEFAULT_GREPTWEETS_URL;
        tweets_txt = DEFAULT_TWEET_FILE;
    }

    if let Ok(metadata) = fs::metadata(tweets_txt) {
        if let Ok(last_modified) = metadata.modified() {
            if let Ok(diff) = SystemTime::now().duration_since(last_modified) {
                if diff >= Duration::from_secs(172800) {
                    if let Err(e) = get_tweets(tweets_url, tweets_txt) {
                        println!("Error encountered retrieving tweets: {:?}", e);
                    }
                }
            }
        }
    } else if let Err(e) = get_tweets(tweets_url, tweets_txt) {
        println!("Error encountered retrieving tweets: {:?}", e);
    }

    match random_tweet(tweets_txt) {
        Err(e) => println!("Error encountered reading tweet file: {:?}", e),
        Ok(tweet) => println!("{}", Blue.paint(tweet)),
    };

    add_tweets_mongo(tweets_txt);
}
