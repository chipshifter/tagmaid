//! Manages the [`TagFile`](TagFile) object
use crate::data::ui_util;
use anyhow::{bail, Context, Result};
#[cfg(test)]
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashSet;
use std::fs::{File, Metadata};
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
/// TagFile is a object used to handle user files to TagDatabase. It contains
/// some attributes related to the file, such as file name, path, hash, associated tags
/// (if any present in the database) and a `File` instance for other file operations.
///
/// TagFiles are initialised with [`initialise_from_path`](TagFile::initialise_from_path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFile {
    pub path: PathBuf,
    pub file_name: String,
    // NOTE: `Vec<u8>` is used here because `rusqlite` implement `FromSql`/`ToSql` for it
    // (therefore is easy to handle in `SqliteDatabase`)
    pub file_hash: Vec<u8>,
    pub tags: HashSet<String>,
}

impl TagFile {
    /// Creates a new, completely empty TagFile object.
    pub fn new() -> TagFile {
        TagFile {
            path: PathBuf::new(),
            file_name: String::new(),
            file_hash: Vec::new(),
            tags: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.path == PathBuf::new()
            && self.file_name.is_empty()
            && self.file_hash.is_empty()
            && self.tags.is_empty()
        {
            return true;
        } else {
            return false;
        }
    }

    /// Initialises a TagFile from a given file's path. The file has to exist and be accessible,
    /// since it is opened for the `self.file` attribute, and hashed for the `self.file_hash` attribute.
    ///
    /// ATTENTION: Returns a TagFile in all cases (attention, some attributes may be empty if it fails).
    pub fn initialise_from_path(path: &Path) -> Result<TagFile> {
        let mut tagfile = TagFile::new();
        tagfile.file_name = match path.file_name() {
            Some(file_os_name) => match file_os_name.to_os_string().into_string().ok() {
                Some(file_name) => file_name,
                None => String::new(),
            },
            None => String::new(),
        };
        tagfile.path = path.to_owned();
        tagfile.file_hash = (&tagfile)
            .file_hash()
            .context("Could not get file ID while initialising TagFile")?;

        Ok(tagfile)
    }

    pub fn get_tags(&self) -> &HashSet<String> {
        &self.tags
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn get_file_name_from_path(&self) -> &str {
        match &self.path.as_path().file_name() {
            Some(file_name) => return file_name.to_str().unwrap(),
            None => "unknown",
        }
    }

    /// Adds the given tag to the HashSet from the `tags` attribute.
    ///
    /// ATTENTION: This *does not* update the tags saved in database, only the instance loaded
    /// in the memory. To update/save the tags change, use
    /// [`update_tagfile`](crate::database::tagmaid_database::TagMaidDatabase::update_tagfile)
    pub fn add_tag(&mut self, tag: &str) -> Result<()> {
        if super::tag_util::is_tag_name_valid(tag) {
            self.tags.insert(tag.to_owned());
        }
        Ok(())
    }

    pub fn add_tags(&mut self, tags: &HashSet<String>) -> Result<()> {
        for tag in tags.iter() {
            self.add_tag(&tag).with_context(|| {
                format!(
                    "Couldn't add tag '{}' to file with path '{}'",
                    &tag,
                    &self.path.display()
                )
            })?;
        }
        Ok(())
    }

    /// Removes the given tag to the HashSet from the `tags` attribute.
    ///
    /// ATTENTION: This *does not* update the tags saved in database, only the loaded object
    /// in memory. To update/save the tags change, use
    /// the `update_tagfile()` function in `TagMaidDatabase`.
    pub fn remove_tag(&mut self, tag: &str) -> Result<()> {
        if self.tags.contains(tag) {
            self.tags.remove(tag);
        }
        Ok(())
    }

    pub fn remove_tags(&mut self, tags: HashSet<String>) -> Result<()> {
        for tag in tags {
            self.remove_tag(&tag).with_context(|| {
                format!(
                    "Couldn't remove tag '{}' from file with path '{}'",
                    &tag,
                    &self.path.display()
                )
            })?;
        }
        Ok(())
    }

    pub fn remove_all_tags(&mut self) -> Result<()> {
        self.tags.clear();
        Ok(())
    }

    /// Tries to open file at &self.path and return the file's std::fs::Metadata
    fn get_metadata(&self) -> Result<Metadata> {
        match File::open(&self.get_path()).ok() {
            Some(file) => {
                let file_metadata: Metadata = file.metadata()?;
                Ok(file_metadata)
            }
            None => {
                bail!("Couldn't open file and get metadata")
            }
        }
    }

    /// Calculates a Blake3 digest of the file (using their path). Is used in
    /// `file_hash()`.
    fn blake3_digest(path: &Path) -> Result<blake3::Hash> {
        let input = File::open(path)?;
        let mut reader = BufReader::new(input);

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 1024];

        // don't read the entire file in the memory at once
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hasher.finalize())
    }

    /// Returns a Blake3 hash of the TagFile's associated file (using the `path` attribute)
    fn file_hash(&self) -> Result<Vec<u8>> {
        let path = &self.path;
        let hash_arraystring = Self::blake3_digest(&path)
            .with_context(|| format!("Couldn't get file hash for {}", &self.path.display()))?;
        let hash_bytes = hash_arraystring.as_bytes().to_vec();
        Ok(hash_bytes)
    }

    pub fn get_thumbnail_path(&self) -> PathBuf {
        let file_path = *(&self.get_path());
        // Thumbnail dimensions are hardcoded to 100x100 (maximum, aspect ratio is preserved)
        let thumbnail_path = ui_util::create_image_thumbnail(file_path, 100, 100);
        return thumbnail_path;
    }

    pub fn display(&self) -> String {
        return format!(
            "TagFile{{file_name: {}, path: {}, tags: {:?}, file_hash: {:?}}}",
            &self.get_file_name(),
            &self.get_path().display(),
            &self.get_tags(),
            crate::data::tag_util::bytes_to_hex(&self.file_hash)
        );
    }

    /// Creates a totally random TagFile instance.
    /// File is located in temp directiories. Used for unit testing
    #[cfg(test)]
    pub fn create_random_tagfile() -> TagFile {
        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let tmp_dir = tempfile::tempdir().unwrap();
        // into_path is necessary for tempdir to persist in the file system
        let tmp_file_path = tmp_dir.into_path().as_path().join(random_string);

        let mut temp_file = File::create(&tmp_file_path).unwrap();
        let random_string_2 = Alphanumeric.sample_string(&mut rand::thread_rng(), 256);
        let _ = &temp_file.write_all(random_string_2.as_bytes()).unwrap();

        let tagfile = TagFile::initialise_from_path(&tmp_file_path).unwrap();
        return tagfile;
    }

    /// Creates a random TagFile instance that is uploaded to a random TagDatabase instance.
    /// File and database is located in temp directiories. Used for unit testing
    #[cfg(test)]
    pub fn create_random_tagfile_in_fsdatabase() -> TagFile {
        let tmp_dir = tempfile::tempdir().unwrap();
        let mut tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        tmp_path.push(random_string);

        let db: crate::FsDatabase = crate::FsDatabase::initialise(&tmp_path).unwrap();
        let tagfile = TagFile::create_random_tagfile();
        let uploaded_tagfile = db.upload_file(&tagfile).unwrap();
        return uploaded_tagfile;
    }
}

impl std::fmt::Display for TagFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};

    #[test]
    fn should_tagfile_initialise() {
        use std::collections::HashSet;
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_file_path = temp_dir.path().join("tagfile-test.maid");

        {
            let _temp_file = File::create(&temp_file_path).unwrap();
        }

        let tag_file: TagFile = TagFile::initialise_from_path(&temp_file_path).unwrap();

        assert_eq!(&tag_file.path, &temp_file_path);

        // operations on File since two files can't be directly compared
        {
            let _file: File = File::open(&tag_file.path).unwrap();
        }
        assert_eq!(&tag_file.file_hash, &TagFile::file_hash(&tag_file).unwrap());
        assert_eq!(&tag_file.tags, &HashSet::new());
    }

    #[test]
    fn should_add_and_remove_tag() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_file_path = temp_dir.path().join("tagfile-test.maid");
        {
            let _temp_file = File::create(&temp_file_path).unwrap();
        }

        let mut tag_file: TagFile = TagFile::initialise_from_path(&temp_file_path).unwrap();

        // test: quick check if HashSet created properly

        assert_eq!(&tag_file.tags, &HashSet::new());

        // test: add tag

        tag_file.add_tag(&String::from("hi")).unwrap();

        let mut tags_that_should_be_added = HashSet::new();
        tags_that_should_be_added.insert(String::from("hi"));

        assert_eq!(&tag_file.tags, &tags_that_should_be_added);

        // test: remove tag

        tag_file.remove_tag(&String::from("hi")).unwrap();

        assert_ne!(&tag_file.tags, &tags_that_should_be_added);
        assert_eq!(&tag_file.tags, &HashSet::new());

        // test: add multiple tags at once

        tags_that_should_be_added.insert(String::from("hi2"));
        tags_that_should_be_added.insert(String::from("hi3"));
        tag_file.add_tags(&tags_that_should_be_added).unwrap();

        assert_eq!(&tag_file.tags, &tags_that_should_be_added);
    }

    #[test]
    fn should_blake3_digest_function_work() {
        use crate::TagFile;
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        /* Check small files
         */

        let hash = blake3::hash(b"bird goes coucou");

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("hash-test-1.maid");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "bird goes coucou").unwrap();
        let digest_result = TagFile::blake3_digest(&file_path).unwrap();

        // should be equal
        assert_eq!(hash, digest_result);

        let file_path = dir.path().join("hash-test-2.maid");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "bird goes wahou").unwrap();
        let digest_result = TagFile::blake3_digest(&file_path).unwrap();

        // should not be equal
        assert_ne!(hash, digest_result);

        /*  Check large files
            The hardcoded hash was obtained with a different tool
        */

        let large_file_hash = TagFile::blake3_digest(&Path::new("src/sample/toes.png")).unwrap();
        assert_eq!(
            large_file_hash.to_hex().as_bytes(),
            b"d8fb642056b94106f3fb9916653cf402d84a3f11751c966866299e47cdf23ea9"
        );
    }
}
