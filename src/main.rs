use once_cell::sync::Lazy;
use serenity::{
    http::Http,
    model::id::{ChannelId, MessageId},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

struct Globals {
    http: Http,
    channel: ChannelId,
    wd: PathBuf,
}

static GLOBALS: Lazy<Globals> = Lazy::new(|| {
    use std::{env::var, fs::DirBuilder};

    dotenv::dotenv().unwrap();

    let token = var("DISCORD_FS_TOKEN").unwrap();
    let http = Http::new(&token);

    let channel = ChannelId(var("DISCORD_FS_CHANNEL_ID").unwrap().parse().unwrap());

    let mut wd_s = "discord_fs".to_owned();
    let mut wd = PathBuf::from(&wd_s);
    while wd.exists() {
        wd_s.push('_');
        wd.set_file_name(&wd_s);
    }
    DirBuilder::new().create(&wd).unwrap();

    Globals { http, channel, wd }
});

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    #[derive(Parser)]
    struct Args {
        /// Upload the file
        #[clap(short, long, value_parser, value_name = "FILE")]
        upload: Option<String>,
        /// Download the file from the message with the given ID
        #[clap(short, long, value_parser, value_name = "ID")]
        download: Option<u64>,
    }

    let args = Args::parse();
    match (args.upload, args.download) {
        (None, None) => return Err("Use --upload or --download".into()),
        (Some(_), Some(_)) => return Err("Use either --upload or --download, not both".into()),
        (Some(file), None) => println!("{}", upload_file(file).await?),
        (None, Some(id)) => println!("{}", download_file(id).await?.to_string_lossy()),
    }

    std::fs::remove_dir_all(&GLOBALS.wd)?;
    Ok(())
}

async fn upload_file(
    file: impl AsRef<std::path::Path>,
) -> Result<MessageId, Box<dyn std::error::Error>> {
    let filename = file
        .as_ref()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    const MAX_SIZE: usize = 8 * 2usize.pow(20);

    let mut file = File::open(file)?;
    let mut n_files = 0;
    for i in 0.. {
        let mut buf = vec![0u8; MAX_SIZE].into_boxed_slice();
        let bytes_read = file.read(&mut *buf)?;
        if bytes_read == 0 {
            break;
        }
        n_files += 1;
        let mut new_file = File::create((&GLOBALS.wd).join(format!("{i}")))?;
        new_file.write_all(&buf[..bytes_read])?;
        if bytes_read < MAX_SIZE {
            break;
        }
    }

    let files = (0..n_files)
        .map(|i| {
            GLOBALS
                .wd
                .join(format!("{i}"))
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let message = GLOBALS
        .channel
        .send_files(&GLOBALS.http, files.iter().map(String::as_str), |m| {
            m.content(filename)
        })
        .await?;

    Ok(message.id)
}

async fn download_file(msg: impl Into<MessageId>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let msg = GLOBALS.channel.message(&GLOBALS.http, msg).await?;

    let mut file_name = msg.content;
    let mut file_path = PathBuf::from(&file_name);
    while file_path.exists() {
        file_name.push('_');
        file_path.set_file_name(&file_name);
    }

    let mut file = File::create(&file_path)?;

    for at in msg.attachments {
        let bytes = at.download().await?;
        file.write_all(&bytes)?;
    }

    Ok(file_path)
}
