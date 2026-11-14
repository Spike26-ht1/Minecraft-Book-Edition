use crate::BookingEngine;
use book_core::{BlockId, BlockState};
use book_world::{
    ChunkColumn, ChunkManager, ChunkPos, ChunkSection, PalettedBlockStorageSnapshot,
    SECTION_VOLUME,
};
use std::collections::BTreeSet;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"BOOKWRLD";
const FORMAT_VERSION: u16 = 1;
const CHECKSUM_SIZE: usize = 8;
const MAX_CHUNKS: usize = 65_536;
const MAX_SECTIONS_PER_CHUNK: usize = 512;
const MAX_SAVE_BYTES: usize = 512 * 1024 * 1024;

/// Errors produced while reading or writing a Booking world save.
#[derive(Debug)]
pub enum WorldSaveError {
    Io(io::Error),
    InvalidData(&'static str),
    UnsupportedVersion(u16),
}

impl fmt::Display for WorldSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "world save I/O error: {error}"),
            Self::InvalidData(message) => write!(formatter, "invalid world save: {message}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported Booking world save version: {version}")
            }
        }
    }
}

impl std::error::Error for WorldSaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for WorldSaveError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl BookingEngine {
    /// Encodes all currently loaded chunk columns using palette-compressed data.
    pub fn encode_world(&self) -> Result<Vec<u8>, WorldSaveError> {
        let chunk_count = self.world().loaded_count();
        if chunk_count > MAX_CHUNKS {
            return Err(WorldSaveError::InvalidData("too many chunk columns"));
        }

        let mut output = Vec::new();
        output.extend_from_slice(MAGIC);
        write_u16(&mut output, FORMAT_VERSION);
        write_u32(&mut output, chunk_count as u32);

        for (position, column) in self.world().iter() {
            write_i32(&mut output, position.x);
            write_i32(&mut output, position.z);

            let section_count = column.section_count();
            if section_count > MAX_SECTIONS_PER_CHUNK {
                return Err(WorldSaveError::InvalidData("too many vertical sections in a chunk"));
            }
            write_u16(&mut output, section_count as u16);

            for (section_y, section) in column.iter_sections() {
                let snapshot = section.snapshot();
                if snapshot.capacity != SECTION_VOLUME
                    || !(4..=16).contains(&snapshot.bits_per_entry)
                    || snapshot.palette.is_empty()
                    || snapshot.palette.len() > (1_usize << snapshot.bits_per_entry)
                    || snapshot.palette.first().copied() != Some(BlockState::AIR)
                {
                    return Err(WorldSaveError::InvalidData("invalid in-memory section palette"));
                }

                write_i32(&mut output, section_y);
                output.push(snapshot.bits_per_entry);
                write_u32(&mut output, snapshot.palette.len() as u32);
                for state in snapshot.palette {
                    write_u16(&mut output, state.block.0);
                    write_u16(&mut output, state.state);
                }
                write_u32(&mut output, snapshot.data.len() as u32);
                for word in snapshot.data {
                    write_u64(&mut output, word);
                }

                if output.len() > MAX_SAVE_BYTES {
                    return Err(WorldSaveError::InvalidData("world save exceeds size limit"));
                }
            }
        }

        let checksum = checksum64(&output);
        write_u64(&mut output, checksum);
        Ok(output)
    }

    /// Replaces the currently loaded world only after the complete save has
    /// been decoded and validated successfully.
    pub fn decode_world(bytes: &[u8]) -> Result<ChunkManager, WorldSaveError> {
        if bytes.len() < MAGIC.len() + 2 + 4 + CHECKSUM_SIZE
            || bytes.len() > MAX_SAVE_BYTES
        {
            return Err(WorldSaveError::InvalidData("invalid file size"));
        }

        let payload_len = bytes.len() - CHECKSUM_SIZE;
        let expected_checksum = u64::from_le_bytes(
            bytes[payload_len..]
                .try_into()
                .map_err(|_| WorldSaveError::InvalidData("missing checksum"))?,
        );
        if checksum64(&bytes[..payload_len]) != expected_checksum {
            return Err(WorldSaveError::InvalidData("checksum mismatch"));
        }

        let mut reader = SaveReader::new(&bytes[..payload_len]);
        if reader.read_bytes(MAGIC.len())? != MAGIC {
            return Err(WorldSaveError::InvalidData("unrecognized file signature"));
        }

        let version = reader.read_u16()?;
        if version != FORMAT_VERSION {
            return Err(WorldSaveError::UnsupportedVersion(version));
        }

        let chunk_count = reader.read_u32()? as usize;
        if chunk_count > MAX_CHUNKS {
            return Err(WorldSaveError::InvalidData("chunk count exceeds limit"));
        }

        let mut world = ChunkManager::new();
        let mut seen_chunks = BTreeSet::new();

        for _ in 0..chunk_count {
            let position = ChunkPos {
                x: reader.read_i32()?,
                z: reader.read_i32()?,
            };
            if !seen_chunks.insert(position) {
                return Err(WorldSaveError::InvalidData("duplicate chunk coordinates"));
            }

            let section_count = reader.read_u16()? as usize;
            if section_count > MAX_SECTIONS_PER_CHUNK {
                return Err(WorldSaveError::InvalidData("section count exceeds limit"));
            }

            let mut column = ChunkColumn::new(position);
            let mut seen_sections = BTreeSet::new();

            for _ in 0..section_count {
                let section_y = reader.read_i32()?;
                if !seen_sections.insert(section_y) {
                    return Err(WorldSaveError::InvalidData("duplicate section height"));
                }

                let bits_per_entry = reader.read_u8()?;
                if !(4..=16).contains(&bits_per_entry) {
                    return Err(WorldSaveError::InvalidData("invalid palette bit width"));
                }

                let palette_count = reader.read_u32()? as usize;
                if palette_count == 0 || palette_count > (1_usize << bits_per_entry) {
                    return Err(WorldSaveError::InvalidData("invalid palette length"));
                }

                let mut palette = Vec::with_capacity(palette_count);
                for _ in 0..palette_count {
                    palette.push(BlockState::new(
                        BlockId(reader.read_u16()?),
                        reader.read_u16()?,
                    ));
                }

                let word_count = reader.read_u32()? as usize;
                let expected_words = (SECTION_VOLUME * bits_per_entry as usize).div_ceil(64);
                if word_count != expected_words {
                    return Err(WorldSaveError::InvalidData("invalid packed data length"));
                }

                let mut data = Vec::with_capacity(word_count);
                for _ in 0..word_count {
                    data.push(reader.read_u64()?);
                }

                let section = ChunkSection::from_snapshot(PalettedBlockStorageSnapshot {
                    capacity: SECTION_VOLUME,
                    bits_per_entry,
                    palette,
                    data,
                })
                .ok_or(WorldSaveError::InvalidData("invalid palette indices or packed storage"))?;

                column.insert_section(section_y, section);
            }

            world.insert_column(column);
        }

        if !reader.is_finished() {
            return Err(WorldSaveError::InvalidData("unexpected trailing data"));
        }
        Ok(world)
    }

    /// Saves to a temporary sibling file, flushes it, then replaces the target.
    pub fn save_world_to_path(&self, path: &Path) -> Result<(), WorldSaveError> {
        let encoded = self.encode_world()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("world.book");
        let temporary_path = path.with_file_name(format!("{file_name}.tmp"));

        let write_result = (|| -> Result<(), WorldSaveError> {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temporary_path)?;
            file.write_all(&encoded)?;
            file.sync_all()?;
            drop(file);

            #[cfg(windows)]
            if path.exists() {
                fs::remove_file(path)?;
            }

            fs::rename(&temporary_path, path)?;
            Ok(())
        })();

        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result
    }

    /// Loads and validates a save before replacing the active chunk manager.
    pub fn load_world_from_path(&mut self, path: &Path) -> Result<(), WorldSaveError> {
        let metadata = fs::metadata(path)?;
        if metadata.len() as usize > MAX_SAVE_BYTES {
            return Err(WorldSaveError::InvalidData("world save exceeds size limit"));
        }
        let mut file = File::open(path)?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut bytes)?;
        let loaded_world = Self::decode_world(&bytes)?;
        *self.world_mut() = loaded_world;
        Ok(())
    }
}

struct SaveReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> SaveReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], WorldSaveError> {
        let end = self.offset.checked_add(count)
            .ok_or(WorldSaveError::InvalidData("file offset overflow"))?;
        let result = self.bytes.get(self.offset..end)
            .ok_or(WorldSaveError::InvalidData("unexpected end of file"))?;
        self.offset = end;
        Ok(result)
    }

    fn read_u8(&mut self) -> Result<u8, WorldSaveError> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, WorldSaveError> {
        Ok(u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }

    fn read_u32(&mut self) -> Result<u32, WorldSaveError> {
        Ok(u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    fn read_i32(&mut self) -> Result<i32, WorldSaveError> {
        Ok(i32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    fn read_u64(&mut self) -> Result<u64, WorldSaveError> {
        Ok(u64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn checksum64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn example_engine() -> BookingEngine {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        assert!(engine.place_block_at([2, 48, 3], BlockId(3)));
        engine
    }

    #[test]
    fn world_round_trip_preserves_chunk_coordinates_and_block_states() {
        let engine = example_engine();
        let encoded = engine.encode_world().expect("encode world");
        let loaded = BookingEngine::decode_world(&encoded).expect("decode world");

        assert_eq!(loaded.loaded_count(), engine.loaded_chunk_count());
        assert_eq!(loaded.get_block(0, -32, 0), Some(BlockId(4)));
        assert_eq!(loaded.get_block(2, 48, 3), Some(BlockId(3)));
        assert_eq!(loaded.get_block(-1, 24, -1), engine.world().get_block(-1, 24, -1));
        assert!(loaded.column(ChunkPos { x: -1, z: -1 }).is_some());
    }

    #[test]
    fn save_checksum_detects_corrupted_world_data() {
        let engine = example_engine();
        let mut encoded = engine.encode_world().expect("encode world");
        encoded[12] ^= 0x40;
        assert!(matches!(
            BookingEngine::decode_world(&encoded),
            Err(WorldSaveError::InvalidData("checksum mismatch"))
        ));
    }

    #[test]
    fn truncated_world_save_is_rejected_without_panicking() {
        let engine = example_engine();
        let encoded = engine.encode_world().expect("encode world");
        assert!(BookingEngine::decode_world(&encoded[..encoded.len() - 3]).is_err());
    }

    #[test]
    fn save_file_round_trip_replaces_loaded_world() {
        let engine = example_engine();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir()
            .join(format!("booking-world-{}-{timestamp}.book", std::process::id()));

        engine.save_world_to_path(&path).expect("save world");

        let mut loaded = BookingEngine::new();
        loaded.load_world_from_path(&path).expect("load world");
        assert_eq!(loaded.world().loaded_count(), 9);
        assert_eq!(loaded.world().get_block(2, 48, 3), Some(BlockId(3)));

        let _ = fs::remove_file(path);
    }
}
