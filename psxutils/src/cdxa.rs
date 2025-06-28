use bitflags::bitflags;
use bytemuck::cast_slice;
use memmap2::Mmap;
use num_traits::Unsigned;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{self, Cursor, Seek, SeekFrom};
use std::mem;
use std::path::Path;
use thiserror::Error;

/// CRC table for use in error detection calculations.
const CRC_TABLE: [u32; 256] = generate_crc_table();

/// The size of a CD-XA sector in bytes.
const SECTOR_SIZE: usize = 2352;

/// The sync pattern for a CD-XA sector, used to identify the start of a sector.
const SECTOR_SYNC: [u8; 12] = [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0];

bitflags! {
	/// Flags representing the coding information of a CD-XA sector.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	struct CodingInfo: u8 {
		/// Mono audio
		const MONO = 1;
		/// Stereo audio
		const STEREO = 2;
		/// 37800 Hz sampling rate
		const HZ37800 = 4;
		/// 18900 Hz sampling rate
		const HZ18900 = 8;
		/// 4-bit audio
		const BPS4 = 16;
		/// 8-bit audio
		const BPS8 = 32;
		/// Emphasis applied to the audio
		const EMPHASIS = 64;
	}

	/// Flags representing the attributes of a file in a CD-XA directory entry.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	struct FileAttributes: u16 {
		/// Owner read permission
		const OWNER_READ = 1;
		/// Owner execute permission
		const OWNER_EXECUTE = 4;
		/// Group read permission
		const GROUP_READ = 16;
		/// Group execute permission
		const GROUP_EXECUTE = 64;
		/// World read permission
		const WORLD_READ = 256;
		/// World execute permission
		const WORLD_EXECUTE = 1024;
		/// Mode 2 form 1
		const MODE2 = 2048;
		/// Binary data with 2048 bytes per sector
		const BINARY = 3413;
		/// Mode 2 form 2
		const MODE2_FORM2 = 4096;
		/// Video data
		const FADE_TO_BLACK = 5461;
		/// Interleaved data
		const INTERLEAVED = 8192;
		/// Wipeout .AV files
		const WIPEOUT_AV = 9557;
		/// Streaming data
		const STREAM = 15701;
		/// CDDA
		const CDDA = 16384;
		/// CD-DA audio track
		const CCDA_AUDIO_TRACK = 17749;
		/// Directory entry
		const DIRECTORY = 32768;
		/// Indicates that the file is a directory record.
		const DIRECTORY_RECORD = 36181;
	}
}

/// Represents the file flags used in CD-XA directory entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileFlags {
	/// Indicates a file entry.
    File = 0,
    /// Indicates a directory entry.
    Directory = 2,
}

impl From<u8> for FileFlags {
	fn from(value: u8) -> Result<Self, CDXAError> {
		match value {
			0 => Ok(FileFlags::File),
			2 => Ok(FileFlags::Directory),
			_ => Err(CDXAError::FileFlag(value)),
		}
	}
}

/// Represents errors that can occur while reading or parsing CD-XA data.
#[derive(Debug, Error)]
pub enum CDXAError<'a> {
	#[error("Invalid directory record size: expected {0}, got {1}")]
	DirectoryRecordSize(usize, usize),
    #[error("Incorrect EDC: Expected {0}, got {1}")]
    EDCFail(u32, u32),
	#[error("Invalid file flag: {0}")]
	FileFlag(u8),
	#[error("File size is not aligned: {0}")]
	FileSize(u64),
    #[error("Failed to read CD-XA data: {0}")]
    IoError(#[from] io::Error),
	#[error("Offset out of bounds: {0}")]
	OutOfBounds(usize),
    #[error("Could not parse data as integer: {0}")]
    ParseError(&'a [u8]),
	#[error("Invalid sector size: expected {0}, got {1}")]
	SectorSize(usize, usize),
    #[error("Sector subheader mismatch: {0:?} != {1:?}")]
    SectorSubHeaderMismatch(SubHeader, SubHeader),
	#[error("Invalid sector sub-mode: {0}")]
	SectorSubModeInvalid(u8),
    #[error("Invalid sector sync pattern: {0:?}")]
    SectorSyncPattern(&'a [u8]),
}

/// Represents the header of a CD-XA sector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Header {
    adr: u32,
    mode: u8,
}

impl Header {
    /// Parses the header from a byte slice.
    fn parse(data: &[u8]) -> Self {
        Self {
            adr: adr(data[0], data[1], data[2]),
            mode: data[3],
        }
    }
}

/// Represents the sub-modes of a CD-XA sector, indicating the type of data contained in the sector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum SubMode {
	/// Indicates the end of a record.
	EndOfRecord = 0,
	/// Indicates a video sub-mode.
	Video,
	/// Indicates an audio sub-mode.
	Audio,
	/// Indicates a data sub-mode.
	Data,
	/// Indicates a trigger sub-mode.
	Trigger,
	/// Indicates a Form 2 sub-mode.
	Form2,
	/// Indicates a real-time sub-mode.
	RealTime,
	/// Indicates the end of a file.
	EndOfFile,
}

impl From<u8> for SubMode {
	fn from(value: u8) -> Result<Self, CDXAError> {
		match value & !32 {
			0 => Ok(SubMode::EndOfRecord),
			1 => Ok(SubMode::Video),
			2 => Ok(SubMode::Audio),
			3 => Ok(SubMode::Data),
			4 => Ok(SubMode::Trigger),
			5 => Ok(SubMode::Form2),
			6 => Ok(SubMode::RealTime),
			7 => Ok(SubMode::EndOfFile),
			_ => Err(CDXAError::SectorSubModeInvalid(value)),
		}
	}
}

/// Represents the sub-header of a CD-XA sector containing file identification,
/// channel information, sub-mode flags, and coding parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SubHeader {
    file: u8,
    chan: u8,
    submode: SubMode,
    coding_info: CodingInfo,
}

impl SubHeader {
    /// Parses the sub-header from a byte slice.
    fn parse(data: &[u8]) -> Result<Self, CDXAError> {
        Self {
            file: data[0],
            chan: data[1],
            submode: SubMode::from(data[2])?,
            coding_info: CodingInfo::from_bits_truncate(data[3]),
        }
    }
}

/// Represents a CD-XA sector, which includes a header, sub-header, data, and an error detection code (EDC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sector {
    header: Header,
    sub_header: SubHeader,
    data: [u8; 2048],
    edc: u32,
    ecc: [u32; 69],
    gf8_log: [u8; 256],
    gf8_ilog: [u8; 256],
    gf8_product: [[u16; 256]; 43],
}

impl Sector {
    /// Parses a CD-XA sector from a byte slice.
    pub fn parse(data: &[u8]) -> Result<Self, CDXAError> {
        if &data[0..12] != &SECTOR_SYNC {
            return Err(CDXAError::SectorSyncPattern(&data[0..12]));
        }

        let header = Header::parse(&data[12..16]);
        let sub_header = SubHeader::parse(&data[16..20])?;
        let sub_header2 = SubHeader::parse(&data[20..24])?;

        if sub_header != sub_header2 {
            return Err(CDXAError::SectorSubHeaderMismatch(sub_header, sub_header2));
        }

        let mut payload = [0u8; 2048];
        payload.copy_from_slice(&data[24..2072]);

        let edc = parse_le(&data[2072..2076])?;
        let len = parse_le(&data[2056..2060])? as usize;

        /*let crc = calculate_edc(&data[16..], len, edc);
        if crc != edc {
            return Err(CDXAError::EDCFail(edc, crc));
        }*/

        let mut gf8_log = [0u8; 256];
        let mut gf8_ilog = [0u8; 256];
        let mut x = 1u8;

        for exp in 0..255 {
            gf8_log[x as usize] = exp;
            gf8_ilog[exp as usize] = x;
            x = x.wrapping_shl(1) ^ if x & 0x80 != 0 { 13 } else { 0 };
        }

        let mut gf8_product = [[0u16; 256]; 43];
        for i in 0..43 {
            let xx = gf8_ilog[44 - i];
            let yy = gf8_shift(xx ^ 1, 0x19, &gf8_log, &gf8_ilog);
            let xx = gf8_shift(xx, 1, &gf8_log, &gf8_ilog);
            let xx = gf8_shift(xx ^ 1, 0x18, &gf8_log, &gf8_ilog);
            let xx_exp = gf8_log[xx as usize] as usize;
            let yy_exp = gf8_log[yy as usize] as usize;
            for j in 0..256 {
                let x = (xx_exp + gf8_log[j] as usize) % 255;
                let y = (yy_exp + gf8_log[j] as usize) % 255;
                gf8_product[i][j] = gf8_ilog[x] as u16 | (gf8_ilog[y] as u16).wrapping_shl(8);
            }
        }

        let mut mut_data = [0u8; SECTOR_SIZE];
        mut_data.copy_from_slice(&data);
        mut_data[0..12].fill(0);
        parity(&mut mut_data, 0, 43, 19, 86, 2, &gf8_product);
        parity(&mut mut_data, 172, 26, 0, 88, 86, &gf8_product);

        let mut ecc = [0u32; 69];
        ecc.copy_from_slice(cast_slice(&data[2076..SECTOR_SIZE]));

        Ok(Self {
            header,
            sub_header,
            data: mut_data[24..2072].try_into().unwrap(),
            edc: edc,
            ecc,
            gf8_log,
            gf8_ilog,
            gf8_product,
        })
    }

    pub const fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordTime {
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    timezone: u8,
}

impl RecordTime {
    /// Parses a record time from a byte slice.
    fn parse(data: &[u8]) -> Self {
        Self {
            year: data[0],
            month: data[1],
            day: data[2],
            hour: data[3],
            minute: data[4],
            second: data[5],
            timezone: data[6],
        }
    }
}

/// Represents the name of a file or directory in a CD-XA volume descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FileName {
	Name(String),
	ParentDirectory,
	RootDirectory,
}

impl From<&[u8]> for FileName {
	fn from(data: &[u8]) -> Self {
		if &data[0] == 0 {
			FileName::RootDirectory
		} else if &data[0] == 1 {
			FileName::ParentDirectory
		} else {
			let name = String::from_utf8_lossy(data).to_string();
			FileName::Name(name)
		}
	}
}

/// Represents a directory entry in a CD-XA volume descriptor, which contains metadata about files and directories.
#[derive(Debug, Clone)]
struct Directory {
    length: u8,
    ext_attr_length: u8,
    data_logical_block: u32,
    data_size: u32,
    recording_timestamp: RecordTime,
    flags: FileFlags,
    file_unit_size: u8,
    interleave_gap_size: u8,
    volume_sequence_number: u16,
    name: FileName,
}

impl Directory {
    /// Parses a directory entry from a byte slice.
    fn parse(data: &[u8]) -> Result<Self, CDXAError> {
		if data.len() < 34 {
			return Err(CDXAError::DirectoryRecordSize(34, data.len()));
		}

        let length = data[0];
		if length < 34 {
			return Err(CDXAError::DirectoryRecordSize(34, length as usize));
		}

        let ext_attr_length = data[1];
        let data_logical_block: u32 = parse_pair(data[2..10])?;
        let data_size: u32 = parse_pair(data[10..18])?;
        let recording_timestamp = RecordTime::parse(&data[18..25]);
		let flags = FileFlags::from(data[25])?;
        let file_unit_size = data[26];
        let interleave_gap_size = data[27];
        let volume_sequence_number: u16 = parse_pair(data[28..32])?;
        let name_length = data[32] as usize;
        let name = FileName::from(&data[33..33 + name_length]);

		Ok(Self {
            length,
            ext_attr_length,
            data_logical_block,
            data_size,
            recording_timestamp,
            flags,
            file_unit_size,
            interleave_gap_size,
            volume_sequence_number,
            name,
        })
    }
}

/// Represents a CD-XA volume descriptor, which contains metadata about the volume.
#[derive(Debug, Clone)]
struct VolumeDescriptor {
    kind: u8,
    identifier: [u8; 5],
    version: u8,
    system_id: [u8; 32],
    volume_id: [u8; 32],
    volume_space_size: u32,
    volume_set_size: u16,
    volume_sequence_number: u16,
    logical_block_size: u16,
    path_table_size: u32,
    type_l_path_table: u32,
    opt_type_l_path_table: u32,
    type_m_path_table: u32,
    opt_type_m_path_table: u32,
    root_directory: Directory,
    volume_set_id: [u8; 128],
    publisher_id: [u8; 128],
    data_preparer_id: [u8; 128],
    application_id: [u8; 128],
    copyright_file_id: [u8; 37],
    abstract_file_id: [u8; 37],
    bibliographic_file_id: [u8; 37],
    creation_timestamp: [u8; 17],
    modification_timestamp: [u8; 17],
    expiration_timestamp: [u8; 17],
    effective_timestamp: [u8; 17],
    file_structure_version: u8,
    application_use1: [u8; 141],
    cdxa_signature: [u8; 8],
    cdxa_flags: u16,
    cdxa_startup_directory: [u8; 8],
    application_use2: [u8; 345],
}

impl VolumeDescriptor {
	/// Parses a volume descriptor from a byte slice.
	fn parse(data: &[u8]) -> Result<Self, CDXAError> {
		if data.len() < 2048 {
			return Err(CDXAError::ParseError(data));
		}

		let kind = data[0];
		let identifier = data[1..6].try_into().unwrap();
		let version = data[6];
		let system_id = data[8..40].try_into().unwrap();
		let volume_id = data[40..72].try_into().unwrap();
		let volume_space_size: u32 = parse_pair(&data[80..88])?;
		let volume_set_size: u16 = parse_pair(&data[120..124])?;
		let volume_sequence_number: u16 = parse_pair(&data[124..128])?;
		let logical_block_size: u16 = parse_pair(&data[128..132])?;
		let path_table_size: u32 = parse_pair(&data[132..140])?;
		let type_l_path_table: u32 = parse_le(&data[140..144])?;
		let opt_type_l_path_table: u32 = parse_le(&data[144..148])?;
		let type_m_path_table: u32 = parse_be(&data[148..152])?;
		let opt_type_m_path_table: u32 = parse_be(&data[152..156])?;
		let root_directory = Directory::parse(&data[156..190])?;
		let volume_set_id = data[190..318].try_into().unwrap();
		let publisher_id = data[318..446].try_into().unwrap();
		let data_preparer_id = data[446..574].try_into().unwrap();
		let application_id = data[574..702].try_into().unwrap();
		let copyright_file_id = data[702..739].try_into().unwrap();
		let abstract_file_id = data[739..776].try_into().unwrap();
		let bibliographic_file_id = data[776..813].try_into().unwrap();
		let creation_timestamp = data[813..830].try_into().unwrap();
		let modification_timestamp = data[830..847].try_into().unwrap();
		let expiration_timestamp = data[847..864].try_into().unwrap();
		let effective_timestamp = data[864..881].try_into().unwrap();
		let file_structure_version = data[881];
		let application_use1 = data[883..1024].try_into().unwrap();
		let cdxa_signature = data[1024..1032].try_into().unwrap();
		let cdxa_flags = parse_le(&data[1032..1034])?;
		let cdxa_startup_directory = data[1034..1042].try_into().unwrap();
		let application_use2 = data[1050..1395].try_into().unwrap();

		Ok(Self {
			kind,
			identifier,
			version,
			system_id,
			volume_id,
			volume_space_size,
			volume_set_size,
			volume_sequence_number,
			logical_block_size,
			path_table_size,
			type_l_path_table,
			opt_type_l_path_table,
			type_m_path_table,
			opt_type_m_path_table,
			root_directory,
			volume_set_id,
			publisher_id,
			data_preparer_id,
			application_id,
			copyright_file_id,
			abstract_file_id,
			bibliographic_file_id,
			creation_timestamp,
			modification_timestamp,
			expiration_timestamp,
			effective_timestamp,
			file_structure_version,
			application_use1,
			cdxa_signature,
			cdxa_flags,
			cdxa_startup_directory,
			application_use2,
		}
	}
}

/// Represents a path table entry in a CD-XA volume descriptor, which contains information about directories.
#[derive(Debug, Clone)]
struct PathTable {
	directory_logical_block: u32,
	extended_attr_length: u8,
	parent_directory: u16,
	directory_name: FileName,
}

impl PathTable {
	/// Parses a path table entry from a byte slice.
	fn parse(data: &[u8]) -> Result<Self, CDXAError> {
		if data.len() < 8 {
			return Err(CDXAError::ParseError(data));
		}

		let directory_logical_block = parse_le(&data[0..4])?;
		let extended_attr_length = data[4];
		let parent_directory = parse_le(&data[5..7])?;
		let directory_name = FileName::from(&data[7..]);

		Ok(Self {
			directory_logical_block,
			extended_attr_length,
			parent_directory,
			directory_name,
		})
	}
}

pub struct CDXA {
    volume_descriptor: VolumeDescriptor,
    sectors: Vec<Sector>,
	sector_count: usize,
}

impl CDXA {
    /// Creates a new CD-XA instance by reading the data from a specified file path.
    pub fn new(data: &Path) -> Result<Self, CDXAError> {
		let metadata = fs::metadata(data).map_err(CDXAError::IoError)?;

		if metadata.len() % SECTOR_SIZE as u64 != 0 {
			return Err(CDXAError::FileSize(metadata.len()));
		}

		let sector_count = metadata.len() as usize / SECTOR_SIZE;
		let f = File::open(data).map_err(CDXAError::IoError)?;
		let mmap = unsafe { Mmap::map(&f).map_err(CDXAError::IoError)? };
		let mut cursor = Cursor::new(&*mmap);

		seek_sector(&mut cursor, 16)?;
		let volume_descriptor_sector = Sector::parse(&cursor.get_ref()[cursor.position() as usize..cursor.position() as usize + SECTOR_SIZE])?;
		let volume_descriptor = VolumeDescriptor::parse(volume_descriptor_sector.get_data())?;

		Ok(Self {
			volume_descriptor,
			sectors: Vec::with_capacity(sector_count),
			sector_count,
		})
	}
}

/// Calculates the absolute sector address from BCD-encoded minute, second, and frame values.
const fn adr(minute: u8, second: u8, frame: u8) -> u32 {
    (decimal(minute) as u32) * 4500 + (decimal(second) as u32) * 75 + (decimal(frame) as u32)
}

/// Calculates the error detection code (EDC) for the sector data.
const fn calculate_edc(input: &[u8], len: usize, crc: u32) -> u32 {
    let mut crc = crc;
    for i in 0..len {
        crc ^= input[i] as u32;
        crc = crc.wrapping_shr(8) ^ CRC_TABLE[(crc as usize) ^ (input[i] as usize) & 255] as u32;
    }

    crc
}

/// Converts a BCD-encoded byte to its decimal representation.
const fn decimal(x: u8) -> u8 {
    if x > 0x99 {
        return x;
    }
    (x >> 4) * 10 + (x & 15)
}

/// Generates a CRC table for use in error detection calculations.
const fn generate_crc_table() -> [u32; 256] {
    let mut crc_table = [0u32; 256];
    let mut i = 0;
    
    while i < 256 {
        crc_table[i] = (i as u32).wrapping_shl(24);
        let mut j = 0;
        while j < 8 {
            if crc_table[i] & 1 != 0 {
                crc_table[i] = crc_table[i].wrapping_shr(1) ^ 0xD8018001;
            } else {
                crc_table[i] = crc_table[i].wrapping_shr(1);
            }
            j += 1;
        }
        i += 1;
    }
    
    crc_table
}

/// Shifts a Galois field element by a specified number of bits.
const fn gf8_shift(a: u8, shift: u8, log: &[u8], ilog: &[u8]) -> u8 {
    if a == 0 {
        0
    } else {
        let mut exp = log[a as usize] as i16 - shift as i16;
        if exp < 0 {
            exp += 255;
        }
        ilog[exp as usize]
    }
}

/// Calculates the parity for a given data slice using Galois field multiplication.
const fn parity(
    data: &mut [u8],
    offset: usize,
    len: usize,
    j0: usize,
    step1: usize,
    step2: usize,
    gf8_product: &[[u16; 256]; 43],
) {
    let mut src = 12;
    let mut dst = 2076 + offset;
    let srcmax = dst;

    for _ in 0..len {
        let base = src;
        let mut x = 0u16;
        let mut y = 0u16;

        for j in j0..43 {
            x ^= gf8_product[j][data[src] as usize];
            y ^= gf8_product[j][data[src + 1] as usize];
            src += step1;

            if step1 == 88 && src >= srcmax {
                src -= 2 * 1118;
            }
        }

        data[dst + (len << 1)] = x as u8;
        data[dst] = (x >> 8) as u8;
        data[dst + (len << 1) + 1] = y as u8;
        data[dst + 1] = (y >> 8) as u8;

        dst += 2;
        src = base + step2;
    }
}

/// Parses a byte slice into a native-endian numeric type.
fn parse_be<T: Unsigned>(data: &[u8]) -> Result<T, CDXAError> {
    if data.len() < mem::size_of::<T>() {
        return Err(CDXAError::ParseError(data));
    }
    Ok(T::from_be_bytes(
        data[0..mem::size_of::<T>()]
            .try_into()
            .map_err(|_| CDXAError::ParseError(data))?,
    ))
}

/// Parses a byte slice into a native-endian numeric type.
fn parse_le<T: Unsigned>(data: &[u8]) -> Result<T, CDXAError> {
    if data.len() < mem::size_of::<T>() {
        return Err(CDXAError::ParseError(data));
    }
    Ok(T::from_le_bytes(
        data[0..mem::size_of::<T>()]
            .try_into()
            .map_err(|_| CDXAError::ParseError(data))?,
    ))
}

/// Parses a pair of opposite-endian numeric types from a byte slice.
fn parse_pair<T: Unsigned>(data: &[u8]) -> Result<T, CDXAError> {
	if data.len() < mem::size_of::<T>() << 1 {
		return Err(CDXAError::ParseError(data));
	}
	let first = T::from_ne_bytes(
		data[0..mem::size_of::<T>()]
			.try_into()
			.map_err(|_| CDXAError::ParseError(data))?,
	);
	let second = T::from_ne_bytes(
		data[mem::size_of::<T>()..mem::size_of::<T>() << 1]
			.try_into()
			.map_err(|_| CDXAError::ParseError(data))?,
	);
	Ok(if cfg!(target_endian = "little") {
		first
	} else {
		second
	})
}

/// Seeks to a specific sector in the CD-XA data.
fn seek_sector(cursor: &mut Cursor<&[u8]>, sector: usize) -> Result<(), CDXAError> {
	if sector * SECTOR_SIZE > cursor.get_ref().len() {
		return Err(CDXAError::OutOfBounds(sector * SECTOR_SIZE));
	}
	let offset = sector * SECTOR_SIZE;
	cursor.seek(SeekFrom::Start(offset as u64))?;
	Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr() {
        assert_eq!(adr(0, 2, 0), 150);
    }
}
