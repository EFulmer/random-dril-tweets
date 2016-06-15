extern crate ansi_term;
extern crate rand;

use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

use ansi_term::Colour::Blue;

const DEFAULT_GREPTWEETS_URL: &'static str = "http://greptweet.com/u/dril/dril.txt";
const DEFAULT_TWEET_FILE: &'static str = "dril.txt";

fn get_dril_tweets(url: &str, out_file_name: &str) -> io::Result<()> {
    let output = try!(Command::new("curl")
        .arg("--compressed")
        .arg("-s")
        .arg(url)
        .output());

    let mut f = File::create(out_file_name).unwrap();

    try!(f.write(output.stdout.as_slice()));
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

    // would use fs::Metadata::modified, but that's currently flagged as unstable.
    if !(Path::new(tweets_txt).exists()) {
        if let Err(e) = get_dril_tweets(tweets_url, tweets_txt) {
            println!("Error encountered retrieving tweets: {:?}", e);
        }
    }

    match random_tweet(tweets_txt) {
        Err(e) => println!("Error encountered reading tweet file: {:?}", e),
        Ok(tweet) => println!("{}", Blue.paint(tweet)),
    };
}
