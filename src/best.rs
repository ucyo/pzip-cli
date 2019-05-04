use std::fs;
use std::io::{Read, BufReader};
use super::{parse_shape, parse_ring};
use byteorder::{ByteOrder, LittleEndian};

pub fn best(matches: &clap::ArgMatches) {

    // parse cli
    let ifile = String::from(matches.value_of("input").unwrap());
    let shape = parse_shape(&matches);
    let ring = parse_ring(&matches);
    let size = (shape.x * shape.y * shape.z) as usize;

    // read f32 file
    let mut file = fs::File::open(ifile).unwrap();
    let mut bytes: Vec<u8> = Vec::new(); // TODO: Optimize? we know the size
    let s = BufReader::new(file).read_to_end(&mut bytes).unwrap(); // TODO: Optimize? we know the size
    if s % 4 != 0 {
            panic!("Can not be read into f32");
        }
    let mut data: Vec<f32> = vec![0f32; s / 4];
    LittleEndian::read_f32_into(&bytes, &mut data);

    println!("{:?}", matches);
}
