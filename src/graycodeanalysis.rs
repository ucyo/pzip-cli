pub struct GrayCodeAnalysis {
    pub num: u32,
    pub lzc: usize,
    pub zeros: usize,
    pub ones: usize,
    pub remaining: usize,
}

impl GrayCodeAnalysis{
    pub fn new(num: &u32) -> Self {
        let ones = calculate_msb_ones(num);
        let zeros = calculate_msb_zeros(num);
        let lzc  = num.leading_zeros() as usize;
        let remaining = if lzc != 32 {
            32 - (lzc + ones + zeros)
        } else {0};
        GrayCodeAnalysis{num:*num, lzc, ones, zeros, remaining}
    }
}

impl std::fmt::Display for GrayCodeAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:032b}, {}, {}, {}, {}", self.num, self.lzc, self.ones, self.zeros, self.remaining)
    }
}


fn calculate_msb_ones(num: &u32) -> usize {
    let next = num.next_power_of_two();
    let mut pos = 1;

    while ((next >> pos) & num) != 0 {
        pos += 1;
    }
    pos - 1
}


fn calculate_msb_zeros(num: &u32) -> usize {
    if *num == 0 {
        return 32
    }
    let next = num.next_power_of_two();
    let num_without_msb = num - (next >> 1);
    let zeros = num_without_msb.leading_zeros() - num.leading_zeros();

    (zeros - 1) as usize
}

use std::fs;
use std::io::Read;
use byteorder::{LittleEndian, ByteOrder};
pub fn read_u32(file: &String) -> Vec<u32> {
    let mut file = fs::File::open(file).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    let size = file.read_to_end(&mut bytes).unwrap();

    if size % 4 != 0 {
        panic!("Can not be read into f32");
    }


    let mut data = vec![0_u32; size / 4];
    LittleEndian::read_u32_into(&bytes, &mut data);
    data
}

pub fn analyse_file(file: &String) -> Vec<GrayCodeAnalysis> {
    let data = read_u32(file);
    let result = data.iter().map(|x| GrayCodeAnalysis::new(x)).collect();
    result
}
