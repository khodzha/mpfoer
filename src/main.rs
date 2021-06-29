use anyhow::Context as AnyContext;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    Client,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Got message: {:?}", msg);

        for attachment in &msg.attachments {
            if let Some(mime) = &attachment.content_type {
                if mime.starts_with("video/") {
                    if attachment.height.is_none() || attachment.width.is_none() {
                        // lets transcode it to h264
                        match transcode_vid(&attachment.url).await {
                            Ok(None) => {
                                println!("Something went wrong")
                            }
                            Err(e) => {
                                println!("Something went wrong, err = {:?}", e)
                            }
                            Ok(Some(output)) => {
                                let send = msg.channel_id.send_message(&ctx.http, |m| {
                                    m.add_file(output.as_str());
                                    m.reference_message(&msg);
                                    m.allowed_mentions(|am| {
                                        am.empty_parse()
                                    });
                                    m
                                });
                                if let Err(why) = send.await {
                                    println!("Error sending message: {:?}", why);
                                }
                                if let Err(e) = tokio::fs::remove_file(output).await {
                                    println!("Error deleting output file: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
        /*
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
        */
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    tokio::fs::create_dir_all("/tmp/mpfoer")
        .await
        .expect("Failed to create /tmp/mpfoer");

    let token = read_token();
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Failed to start client");

    if let Err(why) = client.start().await {
        println!("Err with client: {:?}", why);
    }
}

fn read_token() -> String {
    use std::{fs::File, io::Read};

    let mut f = File::open("token.txt").expect("Failed to open token.txt");
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)
        .expect("Failed to read token.txt");

    buffer
}

async fn transcode_vid(url: &str) -> anyhow::Result<Option<String>> {
    let filename = dl_file(url).await?;

    let output_filename = format!("{}-output.mp4", filename);
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(&filename)
        .args(&[
            "-c:a",
            "copy",
            "-c:v",
            "h264",
            "-crf",
            "18",
            "-f",
            "mp4",
            "-y",
            "-abort_on",
            "empty_output",
        ])
        .arg(&output_filename);

    let mut child = cmd.spawn().expect("failed to spawn");

    // Await until the command completes
    let status = child.wait().await.context("ffmpeg")?;

    match status.success() {
        true => {
            tokio::fs::remove_file(filename).await?;

            Ok(Some(output_filename))
        },
        false => Ok(None),
    }
}

async fn dl_file(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let body = response.bytes().await.context("reqwest")?;

    let filename = filename(url);
    eprintln!("filename: {:?}, bodylen: {}", filename, body.len());
    // let bytes = base64::decode(body).context("base64")?;
    let mut file = File::create(&filename).await?;
    file.write_all(&body.as_ref()).await?;
    file.sync_all().await?;
    drop(file);

    Ok(filename)
}

fn filename(url: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    let url_fname = url.split('/').last().unwrap_or("missing_fname");

    let mut rng = thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(7)
        .collect();

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    format!(
        "/tmp/mpfoer/vid-{}-{}-{}",
        url_fname,
        chars,
        now.as_micros()
    )
}
