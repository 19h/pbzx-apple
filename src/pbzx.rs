use std::cmp::Ordering;
use byteorder::{ReadBytesExt, BigEndian};

use lzma::LzmaReader;
use std::io::Read;

const LZMA_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]; // ²7zXZ..
const LZMA_MAGIC_FOOTER: &[u8] = &[0x59, 0x5A];                  // YZ

#[inline]
fn read_vec<R>(file: &mut R, len: usize) -> Result<Vec<u8>, std::io::Error>
    where R: std::io::Read,
{
    let mut buf = [0u8].repeat(len);

    file.read(&mut *buf)?;

    Ok(buf)
}

#[inline]
fn cmp_str(buf: &Vec<u8>, sstr: &[u8]) -> bool {
    buf.cmp(&sstr.to_vec()) == Ordering::Equal
}

#[derive(Debug)]
pub struct PbzxEntry {
    pub flags: u64,
    pub data: Vec<u8>,
    pub lzma: bool,
}

pub struct PbzxFile {
    pub flags: u64,
    pub entries: Vec<PbzxEntry>,
}

pub fn lzma_unpack_item_data_if_needed(
    item_data: Vec<u8>,
) -> (Vec<u8>, bool) {
    if !item_data.starts_with(LZMA_MAGIC) {
        return (item_data, false);
    }

    if !item_data.ends_with(LZMA_MAGIC_FOOTER) {
        println!(
            "Found an item with valid LZMA magic, but without footer. Not unpacking.",
        );

        return (item_data, false);
    }

    let mut out_data = Vec::new();

    {
        let decom = LzmaReader::new_decompressor(&*item_data);

        if decom.is_err() {
            println!(
                "Found an object that has a valid lzma header and footer magic, but failed to be decompressed. Returning file as-is.",
            );

            return (item_data, false);
        }

        decom
            .unwrap()
            .read_to_end(&mut out_data);
    }

    (out_data, true)
}

pub fn proces<R>(
    src: &mut R,
    file_len_hint: u64,
) -> std::io::Result<PbzxFile>
    where R: std::io::Read,
{
    let pbzx_magic =
        read_vec(src, 4)?; // p  b  z  x (??)

    assert!(
        cmp_str(&pbzx_magic, b"pbzx"),
        "Archive magic could not be found. (expected 'pbzx' at offset 0x00)",
    );

    let file_flags = src.read_u64::<BigEndian>()?;

    println!("flags {:x?}", file_flags);

    let mut entries: Vec<PbzxEntry> = Vec::new();

    let mut offset: u64 = 4 + 8;

    let mut index = 0;

    loop {
        let item_flags = src.read_u64::<BigEndian>()?;
        let item_length = src.read_u64::<BigEndian>()?;

        println!("flags {:x?} size {:x?}", item_flags, item_length);

        assert!(
            item_length < (file_len_hint - offset + 8 + 8),
            format!(
                "Item at index {} has invalid length. Found {} bytes, but there are only {} bytes left in this file.",
                index,
                item_length,
                offset,
            ),
        );

        offset += 8 + 8;

        let item_data_raw = read_vec(
            src,
            item_length as usize,
        )?;

        let (item_data, is_lzma) =
            lzma_unpack_item_data_if_needed(
                item_data_raw,
            );

        offset += item_length;

        let entry = PbzxEntry {
            flags: item_flags,
            data: item_data,
            lzma: is_lzma,
        };

        println!(
            "flags {:x?} size {:x?} lzma? {}",
            &entry.flags,
            entry.data.len(),
            &entry.lzma,
        );

        entries.push(entry);

        index += 1;

        let has_continuation_flag = 0 != item_flags & (0x800000 /* new flag */ | 0x01000000 /* legacy */);
        let size_is_exhausted = (file_len_hint - offset) <= 8 /* flag and size tag */;

        if !has_continuation_flag || size_is_exhausted {
            break;
        }
    }

    Ok(
        PbzxFile {
            flags: file_flags,
            entries,
        }
    )
}
