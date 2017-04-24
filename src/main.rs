extern crate egg_mode;

#[macro_use]
extern crate serde_derive;

extern crate toml;

mod config {
    use std::fs::File;
    use std::io;
    use std::io::Read;
    use std::path::Path;

    use super::toml;

    #[derive(Deserialize)]
    pub struct Config {
        pub account: String,
        pub interval: i32,
        pub auth: TwitterAuth,
    }

    #[derive(Deserialize)]
    pub struct TwitterAuth {
        pub consumer_key: String,
        pub consumer_secret: String,
        pub access_token: String,
        pub access_token_secret: String,
    }

    impl Config {
        fn read_config_file(path: &str) -> Result<String, io::Error> {
            let path = Path::new(path);
            let mut config_contents = String::new();
            File::open(&path)?.read_to_string(&mut config_contents)?;

            Ok(config_contents)
        }

        pub fn load(path: &str) -> Result<Config, io::Error> {
            let config_contents = Config::read_config_file(path)?;

            Ok(toml::from_str(&config_contents).unwrap())
        }
    }
}

fn get_oldest_own_tweet_wo_media<'a>(config: &config::Config, token: &egg_mode::Token)
                             -> Option<egg_mode::tweet::Tweet> {

    let tweets: Vec<egg_mode::tweet::Tweet> = egg_mode::tweet::user_timeline(
        &config.account,
        true,
        true,
        &token
    )
        .start()
        .unwrap()
        .response;

    let mut filtered_tweets: Vec<egg_mode::tweet::Tweet> = tweets
        .into_iter()
        .filter_map(|t| {
            let has_media = t.entities.media.is_some();
            let retweeted = t.retweeted == Some(false);
            match has_media || retweeted {
                true => Some(t),
                false => None
            }
        })
        .collect();

    filtered_tweets.pop()
}

fn tweet_again_and_delete_oldest() {
    let config = config::Config::load("Config.toml")
        .unwrap();

    let access_token = egg_mode::KeyPair::new(config.auth.access_token.as_str(),
                                              config.auth.access_token_secret.as_str());
    let consumer_token = egg_mode::KeyPair::new(config.auth.consumer_key.as_str(),
                                                config.auth.consumer_secret.as_str());

    let token = egg_mode::Token::Access {
        consumer: consumer_token,
        access: access_token
    };

    get_oldest_own_tweet_wo_media(&config, &token)
        .map(|tweet| {
            egg_mode::tweet::DraftTweet::new(tweet.text.as_str())
                .send(&token)
                .ok()
                .map(|result| {
                    println!("Tweeted: {}", result.response.text);
                    egg_mode::tweet::delete(tweet.id, &token).unwrap();
                });
        });
}

fn main() {
    tweet_again_and_delete_oldest();
}
