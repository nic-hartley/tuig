use std::sync::Arc;

use dashmap::{DashMap, mapref::entry::Entry as DMEntry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub contents: String,
}

#[derive(Clone, Debug)]
pub enum Entry {
    File(File),
    Directory(Arc<DashMap<String, Entry>>),
}

impl Entry {
    pub fn file(self) -> Option<File> {
        match self {
            Self::File(f) => Some(f),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::File(_) => true,
            _ => false,
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Self::Directory(_) => true,
            _ => false,
        }
    }

    pub fn dir(self) -> Option<Arc<DashMap<String, Entry>>> {
        match self {
            Self::Directory(d) => Some(d),
            _ => None,
        }
    }
}

// this shouldn't be needed outside of tests so let's enforce that
#[cfg(test)]
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Entry::File(sf), Entry::File(of)) => sf == of,
            (Entry::Directory(sd), Entry::Directory(od)) => {
                for sitem in sd.iter() {
                    let oitem = match od.get(sitem.key()) {
                        Some(i) => i,
                        None => return false,
                    };
                    if sitem.value() != oitem.value() {
                        return false;
                    }
                }
                for oitem in od.iter() {
                    let sitem = match od.get(oitem.key()) {
                        Some(i) => i,
                        None => return false,
                    };
                    if sitem.value() != oitem.value() {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
#[cfg(test)]
impl Eq for Entry {}

/// A single machine, somewhere in cyberspace. Possibly even the player's own.
#[derive(Default, Clone, Debug)]
pub struct Machine {
    /// The files on this machine
    // TODO: Cleaner abstraction for this
    pub root: Arc<DashMap<String, Entry>>,
}

impl Machine {
    fn dir(&self, path: &str) -> Result<Arc<DashMap<String, Entry>>, String> {
        if path.is_empty() {
            return Ok(self.root.clone());
        }
        if !path.starts_with('/') {
            return Err(format!("absolute path {} doesn't start with /", path));
        }
        let mut dir = self.root.clone();
        for comp in path.split('/') {
            if comp.is_empty() {
                // ignore consecutive slashes
                continue;
            }
            let into = match dir.entry(comp.to_owned()) {
                DMEntry::Occupied(p) => match p.get() {
                    Entry::File(_) => return Err(format!("{} is a file", comp)),
                    Entry::Directory(dm) => dm.clone(),
                }
                DMEntry::Vacant(p) => {
                    let dm = Arc::new(DashMap::new());
                    p.insert(Entry::Directory(dm.clone()));
                    dm
                }
            };
            dir = into;
        }
        Ok(dir)
    }

    /// Write a file to the machine's disk at the absolute path.
    /// 
    /// Will overwrite any files already there, but will not replace files with directories.
    /// 
    /// Returns Ok(()) if everything worked, or Err(msg) if not.
    pub fn write(&mut self, path: &str, contents: String) -> Result<(), String> {
        // valid: /foo/bar
        //        /foo//bar
        // not:   foo/bar
        //        /foo/bar/
        let (parent, file) = path.rsplit_once('/').unwrap_or(("", path));
        if file.is_empty() {
            return Err(format!("filepaths cannot end with trailing slash"));
        }
        let dir = self.dir(parent)?;
        let new = Entry::File(File {
            contents
        });
        match dir.entry(file.to_owned()) {
            DMEntry::Occupied(mut p) => {
                if p.get().is_file() {
                    p.insert(new);
                } else {
                    return Err(format!("path is a directory"));
                }
            }
            DMEntry::Vacant(p) => {
                p.insert(new);
            }
        }

        Ok(())
    }

    /// Will get any kind of [`Entry`] from the machine's disk at the absolute path.
    /// 
    /// Returns Ok(entry) if everything worked, or Err(msg) if not.
    pub fn entry(&self, path: &str) -> Result<Entry, String> {
        if path == "/" {
            // special-case for root: there isn't really an entry but we can fake one
            return Ok(Entry::Directory(self.root.clone()));
        }
        let (parent, file) = path.trim_end_matches('/').rsplit_once('/').unwrap_or(("", path));
        let dir = self.dir(parent)?;
        let entry = dir.get(file)
            .ok_or(format!("no such entry: {}", path))?;
        Ok(entry.value().clone())
    }

    /// Read a file from the machine's disk at the absolute path.
    /// 
    /// Can only read files; if you try to read a directory this fails.
    /// 
    /// Returns Ok(file) if everything worked, or Err(msg) if not.
    /// 
    /// See also [`Self::readdir`] and [`Self::entry`].
    pub fn read(&self, path: &str) -> Result<File, String> {
        let entry = self.entry(path)?;
        let file = entry.file()
            .ok_or(format!("cannot read non-files"))?;
        Ok(file)
    }

    /// Iterate over the contents of a directory.
    /// 
    /// Returns Ok(iter) if everything worked, or Err(msg) if not.
    /// 
    /// See also [`Self::read`] and [`Self::entry`].
    pub fn readdir(&self, path: &str) -> Result<impl Iterator<Item=(String, Entry)>, String> {
        let entry = self.entry(path)?;
        let dir = entry.dir()
            .ok_or(format!("cannot readdir non-directory {}", path))?;
        Ok(dir.as_ref().clone().into_iter())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_machine_fs_empty() {
        let mach = Machine::default();
        assert!(mach.root.is_empty());
    }

    #[test]
    fn machine_writes_file_to_root() {
        let mut mach = Machine::default();
        mach.write("/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");

        let f = mach.read("/spooky")
            .expect("failed to read file just written");
        assert_eq!(f.contents, "ghost");
    }

    #[test]
    fn machine_overwrites_file_to_root() {
        let mut mach = Machine::default();
        mach.write("/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");
        mach.write("/spooky", "zombie".into())
            .expect("failed to overwrite file");

        let f = mach.read("/spooky")
            .expect("failed to read file just written");
        assert_eq!(f.contents, "zombie");
    }

    #[test]
    fn machine_writes_file_in_subdir() {
        let mut mach = Machine::default();
        mach.write("/things/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");

        let f = mach.read("/things/spooky")
            .expect("failed to read file just written");
        assert_eq!(f.contents, "ghost");
    }

    #[test]
    fn machine_overwrites_file_in_subdir() {
        let mut mach = Machine::default();
        mach.write("/things/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");
        mach.write("/things/spooky", "zombie".into())
            .expect("failed to overwrite file");

        mach.read("/things").expect_err("/things should be a directory");
        mach.read("/spooky").expect_err("/spooky should not exist");
        let f = mach.read("/things/spooky")
            .expect("failed to read file just written");
        assert_eq!(f.contents, "zombie");
    }

    #[test]
    fn machine_wont_overwrite_file_with_dir() {
        let mut mach = Machine::default();
        mach.write("/things", "many".into())
            .expect("failed to write to empty filesystem");
        mach.write("/things/spooky", "ghost".into())
            .expect_err("didn't return error on attempted file overwrite with dir");

        let f = mach.read("/things")
            .expect("could not read file that should be there");
        assert_eq!(f.contents, "many");
    }

    #[test]
    fn machine_wont_overwrite_dir_with_file() {
        let mut mach = Machine::default();
        mach.write("/things/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");
        mach.write("/things", "ghost".into())
            .expect_err("didn't return error on attempted dir overwrite with file");

        let f = mach.read("/things/spooky")
            .expect("could not read file that should be there");
        assert_eq!(f.contents, "ghost");
    }

    #[test]
    fn machine_entry_reads_file() {
        let mut mach = Machine::default();
        mach.write("/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");

        let e = mach.entry("/spooky")
            .expect("coud not read entry that should be there");
        assert_eq!(e, Entry::File(File { contents: "ghost".into() }));
    }

    #[test]
    fn machine_entry_reads_dir() {
        let mut mach = Machine::default();
        mach.write("/thing/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");

        let e = mach.entry("/thing")
            .expect("coud not read entry that should be there");
        assert!(matches!(e, Entry::Directory(_)));
    }

    #[test]
    fn machine_readdir_reads_dir() {
        let mut mach = Machine::default();
        mach.write("/thing/spooky", "ghost".into())
            .expect("failed to write to empty filesystem");
        mach.write("/thing/cute", "me".into())
            .expect("failed to write to empty filesystem");

        let es = mach.readdir("/thing")
            .expect("could not read dir that should be there");
        let mut es: Vec<_> = es.collect();
        es.sort_by(|l, r| l.0.cmp(&r.0));
        assert_eq!(es[0], ("cute".to_owned(), Entry::File(File { contents: "me".into() })));
        assert_eq!(es[1], ("spooky".to_owned(), Entry::File(File { contents: "ghost".into() })));
    }
}
