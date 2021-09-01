use derive_more::From;
use memmap::MmapMut;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, From)]
pub enum Error {
    Io(io::Error),
    NoSpaceLeft,
    InvalidIndex,
}

#[derive(Debug)]
pub struct Log {
    file: File,
    base_offset: usize,
    max_size: usize,
    offset: usize,
    mmap: MmapMut,
}

impl Log {
    pub fn new(
        path: PathBuf,
        base_offset: usize,
        max_size: usize,
        suffix: &str,
    ) -> Result<Log, io::Error> {
        fs::create_dir_all(&path).unwrap();
        let segment_path = path.join(format!("{:020}.log{}", base_offset, suffix));
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(&segment_path)
            .unwrap();

        file.set_len(max_size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };
        let offset = 0;

        Ok(Log {
            file,
            base_offset,
            max_size,
            offset,
            mmap,
        })
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.mmap.flush_async()?;
        Ok(())
    }

    pub fn fit(&mut self, size: usize) -> bool {
        (self.max_size - self.offset) >= size
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let buf_size = buf.len();
        if !self.fit(buf_size) {
            return Err(Error::NoSpaceLeft);
        }

        self.offset += buf_size;
        let size = (&mut self.mmap[(self.offset - buf_size)..(self.offset)]).write(buf)?;
        Ok(size)
    }

    pub fn read_at(&mut self, offset: usize, size: usize) -> Result<&[u8], Error> {
        if (offset + size) > self.mmap.len() {
            return Err(Error::InvalidIndex);
        }

        Ok(&self.mmap[offset..(offset + size)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate tempfile;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create() {
        let tmp_dir = tempdir().unwrap().path().to_owned();

        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        let l = Log::new(tmp_dir.clone(), 0, 10, "").unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(l.offset(), 0);
    }

    #[test]
    fn test_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        let mut l = Log::new(tmp_dir.clone(), 0, 50, "").unwrap();
        l.write(b"boom!-big-reveal!-i-turned-myself-into-a-pickle!")
            .unwrap();
        l.flush().unwrap();

        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("boom!-big-reveal!-i-turned-myself-into-a-pickle!\u{0}\u{0}")
        );

        assert_eq!(l.offset(), 48);
    }

    #[test]
    fn test_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 20, "").unwrap();
        l.write(b"juca-bala").unwrap();
        l.flush().unwrap();

        assert_eq!(l.read_at(0, 9).unwrap(), b"juca-bala");
        assert_eq!(l.read_at(1, 7).unwrap(), b"uca-bal");
    }
}
