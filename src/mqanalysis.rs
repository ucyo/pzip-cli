use super::graycodeanalysis::read_u32;
use pzip::toolbelt::to_bitplanes_irregular_u32;
use pzip::transform::{Compact, CompactMapping};
use std::fs::File;
use std::io::{BufWriter, Write};
use byteorder::{LittleEndian, WriteBytesExt};

pub fn mqanalysis(matches: &clap::ArgMatches) {
    let input = String::from(matches.value_of("input").unwrap());
    let data = read_u32(&input);

    let data_bitplanes_residual = to_bitplanes_irregular_u32(&data);
    let data_bitplanes_residual = from_vec_u32_to_vec_u8(data_bitplanes_residual.1);
    let bitplanes = String::from(matches.value_of("bitplanes").unwrap());
    let mut output = BufWriter::new(File::create(bitplanes).unwrap());
    output.write_all(data_bitplanes_residual.as_slice()).unwrap();

    let data_only_residual = Compact::NoLZC.compact_u32(data);
    let data_only_residual = from_vec_u32_to_vec_u8(data_only_residual);
    let only_residual = String::from(matches.value_of("nolzc").unwrap());
    let mut output = BufWriter::new(File::create(only_residual).unwrap());
    output.write_all(data_only_residual.as_slice()).unwrap();
}

pub fn from_vec_u32_to_vec_u8(data: Vec<u32>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for n in data {
        let _ = result.write_u32::<LittleEndian>(n);
    }
    result
}
