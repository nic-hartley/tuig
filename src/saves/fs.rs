//! Saving and loading to files, specifically.

use std::{path::{PathBuf, Path}, io, time::SystemTime, env};

use tokio::{fs::{read_dir, File, ReadDir}, io::AsyncReadExt};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SaveData {
    pub name: String,
    pub date: SystemTime,
    pub prog_str: String,
}

pub struct SaveDir(ReadDir);

impl SaveDir {
    /// Try to figure out the default directory, based on things like the target platform and 
    pub fn default_dir() -> Option<PathBuf> {

        if cfg!(linux) {
            if let Some(cfg) = env::var_os("CONFIG") {
                return Some(Path::new(&cfg).join("redshell/saves"));
            } else if let Some(home) = env::var_os("HOME") {
                return Some(Path::new(&home).join(".redshell/saves"));
            }
        }
        None
    }

    /// Attempt to open a directory as a save directory.
    /// 
    /// Note that this succeeding does **not** imply that reading any individual save will succeed -- this only checks
    /// that the directory itself is accessible, as of when you opened it. The files could be owned by another user,
    /// or stored on a network share that disconnects, or whatever else. There are many reasons iteration could fail.
    pub async fn open(dir: impl AsRef<Path>) -> io::Result<Self> {
        read_dir(dir).await.map(Self)
    }

    /// Load a single save file at an exact filepath.
    /// 
    /// If it's successful, this function returns the save file's metadata and a [`tokio::fs::File`], which you can
    /// read the actual save data from. It doesn't parse the save itself because this function is also used to build
    /// the *list* of saves, and parsing and storing every save would be an enormous amount of memory consumed for
    /// basically no reason.
    pub async fn load_one(path: impl AsRef<Path>) -> io::Result<(SaveData, File)> {
        // test file metadata first since that's fastest
        match path.as_ref().extension() {
            Some(ext) if ext == "rse" => (),
            // TODO: io::ErrorKind::InvalidFilename
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "extension must be .rse")),
        }
        let md = tokio::fs::metadata(path.as_ref()).await?;
        if md.is_dir() {
            // TODO: io::ErrorKind::IsADirectory
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "is a directory"))
        }

        // open the file for initial parsing
        let mut file = File::open(path.as_ref()).await?;

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic).await?;
        if &magic != b"RDSHSAVE" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "missing magic bytes"));
        }

        // The file has two parts after the magic: one short, up to 65535 bytes long, with what's shown in the saves
        // menu, then the bulk of the actual save data. The first bit is delimited so we can pull and parse just that
        // part without having to touch the rest until the user chooses to load it, saving a *lot* of time and memory.
        let mut header_len_b = [0u8; 2];
        file.read_exact(&mut header_len_b).await?;
        let header_len = u16::from_le_bytes(header_len_b) as usize;

        let mut header_b = vec![0u8; header_len];
        file.read_exact(&mut header_b).await?;

        let header = bincode::deserialize(&header_b)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(_) => unreachable!("didn't deserialize from Read"),
                bincode::ErrorKind::Custom(m) => unreachable!("mysterious serde error: {}", m),
                bincode::ErrorKind::DeserializeAnyNotSupported => unreachable!("not using deserialize_any"),
                bincode::ErrorKind::SequenceMustHaveLength => unreachable!("mysterious sudden unsized iterable"),
                other => io::Error::new(io::ErrorKind::InvalidData, format!("invalid file data: {}", other)),
            })?;

        Ok((header, file))
    }

    /// Attempt to load all of the available saves in the directory. Any items which failed to load for any reason are
    /// silently omitted and if iteration of the directory fails (e.g. due to intermittent errors) the list will just
    /// end right then.
    pub async fn load_all(self) -> Vec<(SaveData, File)> {
        let mut res = self.load_all_verbose().await.into_iter()
            .filter_map(|(_, res)| res.ok())
            .collect::<Vec<_>>();
    
        // order by the date of the save file (default ascending order)
        res.sort_unstable_by_key(|(sd, _)| sd.date);
        // newest first (descending order)
        res.reverse();
    
        res
    }

    /// Similar to [`Self::load_all`] but returns more information. In particular, returns every file it attempted to
    /// load, as well as the result of trying to load it. If directory iteration fails, the list will just end.
    pub async fn load_all_verbose(mut self) -> Vec<(PathBuf, io::Result<(SaveData, File)>)> {
        let mut res = vec![];
        loop {
            let entry = match self.0.next_entry().await {
                Ok(Some(entry)) => entry,
                _ => break,
            };
            res.push((entry.path(), Self::load_one(entry.path()).await));
        }
        res
    }
}
