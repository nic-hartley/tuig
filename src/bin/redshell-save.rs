use std::{path::{PathBuf, Path}, io::{self, Write, BufWriter, BufReader}, fs::File};

use chrono::Utc;
use redshell::{saves::{fs::Directory, SaveSystem, Metadata, SaveHandle}, GameState};

const USAGE: &str = "redshell-save <create/show/list/edit> <filepath..>";

fn print_metadata(md: Metadata) {
    println!("Save {}: {}", md.name, md.created().format("%c"));
    println!("Summary: {}", md.progress);
}

fn yaml_data(md: Metadata, d: GameState, out: &mut dyn Write) -> io::Result<()> {
    let mut out = BufWriter::new(out);
    writeln!(out, "# Save: {} ({})", md.name, md.created().format("%c"))?;
    writeln!(out, "# Summary: {}", md.progress)?;
    writeln!(out)?;
    serde_yaml::to_writer(out, &d).unwrap();
    Ok(())
}

async fn create(path: &Path) -> io::Result<()> {
    let default = GameState::default();
    let file = tokio::fs::File::create(path).await?;
    let md = Metadata { name: "manual".into(), created: Utc::now().timestamp(), progress: "none!".into() };
    let slot = Directory::save_to(file, md).await?;
    slot.save(&default).await?;
    Ok(())
}

async fn show(path: &Path) -> io::Result<()> {
    let (md, handle) = Directory::list_one(path).await?;
    yaml_data(md, handle.load().await?, &mut std::io::stdout().lock())?;
    Ok(())
}

async fn list(path: &Path) -> io::Result<()> {
    for (save, _) in Directory::open(path).list().await? {
        print_metadata(save);
    }
    Ok(())
}

async fn edit(path: &Path) -> io::Result<()> {
    let (md, handle) = Directory::list_one(path).await?;
    let data = handle.load().await?;

    let mut orig_file = tempfile::NamedTempFile::new()?;
    // TODO: io::ErrorKind::InvalidFilename
    yaml_data(md, data, &mut orig_file)?;
    let (_, edit_path) = orig_file.into_parts();

    // much as I'd rather default to nano, too many people expect a `vi` default, so...
    let editor = std::env::var("EDITOR").unwrap_or("vi".into());
    let edit_arg = edit_path.to_str()
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "non-utf8 temp filepath"))?
        .to_owned();
    let mut proc = std::process::Command::new(editor)
        .arg(edit_arg)
        .spawn()?;
    match proc.wait() {
        Ok(s) if s.success() => (),
        Ok(s) => panic!("Command errored: {}", s),
        Err(e) => panic!("Spawning failed: {}", e),
    }

    let edited_file = BufReader::new(File::open(edit_path)?);
    let data: GameState = serde_yaml::from_reader(edited_file).unwrap();
    let (_, handle) = Directory::list_one(path).await?;
    handle.save(&data).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let verb = match args.next() {
        Some(v) => v,
        None => {
            println!("{}", USAGE);
            return;
        }
    };
    for file in args {
        let file = PathBuf::from(file);
        let res = match verb.as_str() {
            "create" => create(&file).await,
            "show" => show(&file).await,
            "list" => list(&file).await,
            "edit" => edit(&file).await,
            _ => panic!("{}", USAGE),
        };
        if let Err(e) = res {
            println!("failed: {}", e);
            return;
        }
    }
}
