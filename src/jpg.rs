use anyhow::{anyhow, ensure, Result};
use time::{macros::format_description, PrimitiveDateTime};

pub fn get_timestamp(data: &[u8]) -> Result<Option<time::PrimitiveDateTime>> {
    ensure!(data[0..2] == [0xff, 0xd8], "Missing SOI marker");

    ensure!(data[2] == 0xff, "Expected start of marker");
    if data[3] != 0xe1 {
        // Image does not contain metadata
        return Ok(None);
    }

    // Find APP1 segment length
    let segment_length = u16::from_be_bytes(data[4..6].try_into().unwrap());

    // Extract APP1 data
    let app1_data = &data[4..(4 + segment_length as usize)];

    ensure!(
        app1_data[2..8] == [0x45, 0x78, 0x69, 0x66, 0x00, 0x00],
        "Invaid exif header"
    );

    ensure!(
        app1_data[8..12] == [0x49, 0x49, 0x2a, 0x00],
        "Invaid tiff header"
    );
    // Get IFD0 offset
    let ifd0_offset = u32::from_le_bytes(app1_data[12..16].try_into().unwrap());

    let ifd0_data = &app1_data[(8 + ifd0_offset as usize)..];
    match parse_ifd0(ifd0_data, app1_data) {
        Some(Ok(timestamp)) => Ok(Some(timestamp)),
        None => Ok(None),
        Some(Err(e)) => Err(e),
    }
}

fn parse_ifd0(data: &[u8], app1_data: &[u8]) -> Option<Result<time::PrimitiveDateTime>> {
    let number_of_entries = u16::from_le_bytes(data[0..2].try_into().unwrap());

    let mut entries = (0..number_of_entries).filter_map(|i| {
        let data_start = 2 + 12 * i as usize;
        let data_end = 14 + 12 * i as usize;
        let entry_data = &data[data_start..data_end];

        parse_ifd_entry(entry_data, app1_data)
    });

    entries.find(|e| e.tag == 0x0132).map(|e| {
        let IFDValue::AsciiStrings(s) = e.data else {
            return Err(anyhow!(
                "DateTime entry contained invalid data format, expected AsciiStrings but got {:?}",
                e.data
            ));
        };

        let date_time_format = format_description!("[year]:[month]:[day] [hour]:[minute]:[second]");

        Ok(PrimitiveDateTime::parse(&s, &date_time_format)?)
    })
}

#[derive(Debug)]
struct IFDEntry {
    tag: u16,
    data: IFDValue,
}

#[derive(Debug)]
#[repr(u16)]
#[allow(dead_code)]
enum IFDValue {
    UnsignedByte(u8),
    AsciiStrings(String),
    UnsignedShort(u16),
    UnsignedLong(u32),
    UnsignedRational,
    SignedByte(i8),
    Undefined(Vec<u8>),
    SignedShort(i16),
    SignedLong(i32),
    SignedRational,
    SingleFloat(f32),
    DoubleFloat(f64),
}

fn parse_ifd_entry(data: &[u8], app1_data: &[u8]) -> Option<IFDEntry> {
    let tag_number = u16::from_le_bytes(data[0..2].try_into().unwrap());
    let data_format = u16::from_le_bytes(data[2..4].try_into().unwrap());
    let number_of_components = u32::from_le_bytes(data[4..8].try_into().unwrap());

    let bytes_per_component = match data_format {
        1 => 1,  // unsigned byte
        2 => 1,  // ascii strings
        3 => 2,  // unsigned short
        4 => 4,  // unsigned long
        5 => 8,  // unsigned rational
        6 => 1,  // signed byte
        7 => 1,  // undefined
        8 => 2,  // signed short
        9 => 4,  // signed long
        10 => 8, // signed rational
        11 => 4, // single float
        12 => 8, // double float
        _ => {
            println!("data format: {data_format} not implemented");
            return None;
        }
    };

    let data_length = bytes_per_component * number_of_components;

    let value_data = if data_length <= 4 {
        &data[8..12]
    } else {
        let offset = u32::from_le_bytes(data[8..12].try_into().unwrap()) + 8;
        let end = offset + data_length;
        &app1_data[(offset as usize)..(end as usize)]
    };

    let value = {
        use IFDValue::*;
        match data_format {
            1 => UnsignedByte(value_data[0]), // unsigned byte
            2 => AsciiStrings(String::from_utf8_lossy(value_data).into_owned()), // ascii strings
            3 => UnsignedShort(u16::from_le_bytes(value_data[0..2].try_into().unwrap())), // unsigned short
            4 => UnsignedLong(u32::from_le_bytes(value_data[0..4].try_into().unwrap())), // unsigned long
            5 => UnsignedRational, // unsigned rational
            6 => SignedByte(i8::from_le_bytes([value_data[0]])), // signed byte
            7 => Undefined(value_data.to_vec()), // undefined
            8 => SignedShort(i16::from_le_bytes(value_data[0..2].try_into().unwrap())), // signed short
            9 => SignedLong(i32::from_le_bytes(value_data[0..4].try_into().unwrap())), // signed long
            10 => SignedRational, // signed rational
            11 => SingleFloat(f32::from_be_bytes(value_data[0..4].try_into().unwrap())), // single float
            12 => DoubleFloat(f64::from_le_bytes(value_data[0..8].try_into().unwrap())), // double float
            _ => unimplemented!(),
        }
    };

    Some(IFDEntry {
        tag: tag_number,
        data: value,
    })
}
