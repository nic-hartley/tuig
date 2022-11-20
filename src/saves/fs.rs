//! Saving and loading to files, specifically.

use std::{path::{PathBuf, Path}, io};

use tokio::{fs::{read_dir, File, OpenOptions, remove_file}, io::{AsyncReadExt, AsyncWriteExt}, task::block_in_place};
use rand::prelude::*;

use crate::GameState;

use super::{SaveSystem, Metadata, SaveHandle};

const EXT: &str = "rse";
const QUICKSAVE: &str = "quicksave";
const MAGIC: &[u8] = b"RDSHSAVE";

fn bc2io_err(e: bincode::Error) -> io::Error {
    match *e {
        // impossible errors
        bincode::ErrorKind::Custom(m) => unreachable!("mysterious serde error: {}", m),
        bincode::ErrorKind::DeserializeAnyNotSupported => unreachable!("not using deserialize_any"),
        bincode::ErrorKind::SequenceMustHaveLength => unreachable!("mysterious sudden unsized iterable"),
        // more possible ones
        bincode::ErrorKind::Io(e) => e,
        other => io::Error::new(io::ErrorKind::InvalidData, format!("invalid file data: {}", other)),
    }
}

/// [`Directory`]'s [`SaveSystem::Handle`] implementation.
pub struct Handle(File);

#[async_trait::async_trait]
impl SaveHandle for Handle {
    type System = Directory;

    async fn load(self) -> Result<GameState, io::Error> {
        let slot_std = self.0.into_std().await;
        block_in_place(|| bincode::deserialize_from(slot_std)).map_err(bc2io_err)
    }

    async fn save(self, data: &GameState) -> Result<(), io::Error> {
        let slot_std = self.0.into_std().await;
        block_in_place(|| bincode::serialize_into(slot_std, data)).map_err(bc2io_err)
    }
}

/// Handle saves out of a directory.
pub struct Directory(PathBuf);

impl Directory {
    /// Read saves from the default location, based on the platform. This will panic if it doesn't know the platform
    /// being targeted.
    pub fn new() -> Self {
        todo!()
    }

    /// Read saves from a specific location.
    pub fn open(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().into())
    }

    /// Get the metadata and handle for a single specific file.
    pub async fn list_one(path: impl AsRef<Path>) -> io::Result<(Metadata, Handle)> {
        // test file metadata first since that's fastest
        match path.as_ref().extension() {
            Some(ext) if ext == EXT => (),
            // TODO: io::ErrorKind::InvalidFilename
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "extension must be .rse")),
        }
        let md = tokio::fs::metadata(path.as_ref()).await?;
        if md.is_dir() {
            // TODO: io::ErrorKind::IsADirectory
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "is a directory"))
        }

        // open the file for initial parsing
        let mut file = OpenOptions::new()
            .read(true).write(true).truncate(false)
            .open(path.as_ref()).await?;

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic).await?;
        if &magic != MAGIC {
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
            .map_err(bc2io_err)?;

        Ok((header, Handle(file)))
    }

    pub async fn save_to(mut file: File, metadata: Metadata) -> io::Result<Handle> {
        let mut data = Vec::with_capacity(10);
        data.extend_from_slice(MAGIC);
        let header_len = bincode::serialized_size(&metadata).unwrap();
        if header_len > u16::MAX as u64 {
            // TODO: io::ErrorKind::FileTooLarge
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "metadata is too long to serialize"));
        }
        data.extend_from_slice(&(header_len as u16).to_le_bytes());
        bincode::serialize_into(&mut data, &metadata).unwrap();
        file.write_all(&data).await?;
        Ok(Handle(file))
    }
}

#[async_trait::async_trait]
impl SaveSystem for Directory {
    type Handle = Handle;
    type Error = io::Error;

    async fn list_verbose(&self) -> Result<Vec<Result<(Metadata, Self::Handle), Self::Error>>, Self::Error> {
        let mut reader = read_dir(&self.0).await?;
        let mut res = vec![];
        loop {
            let entry = match reader.next_entry().await {
                Ok(Some(entry)) => entry,
                _ => break,
            };
            res.push(Self::list_one(entry.path()).await);
        }
        Ok(res)
    }

    async fn new_slot(&self, metadata: Metadata) -> Result<Handle, Self::Error> {
        let file = loop {
            let id = format!("{:08x}.{}", thread_rng().gen::<u32>(), EXT);
            match OpenOptions::new().create_new(true).write(true).open(self.0.join(id)).await {
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                other => break other?,
            }
        };

        Self::save_to(file, metadata).await
    }

    async fn quicksave(&self) -> Result<Self::Handle, Self::Error> {
        // the quicksave slot is a file named `quicksave.rse` in the save directory, but because it requires no save
        // metadata (it's the quicksave) we can just... open it.
        OpenOptions::new().create(true).write(true).truncate(false)
            .open(self.0.join(QUICKSAVE))
            .await.map(Handle)
    }

    async fn cleanup(self) -> Result<(), Self::Error> {
        // all we need to do is delete the quicksave; everything else is handled by `Drop` impls
        remove_file(self.0.join(QUICKSAVE)).await
    }
}
