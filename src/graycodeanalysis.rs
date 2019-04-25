pub struct GrayCodeAnalysis {
    pub num: u32,
    pub lzc: usize,
    pub zeros: usize,
    pub ones: usize,
    pub remaining: usize,
    pub residuallength: usize,
}

impl GrayCodeAnalysis{
    pub fn new(num: &u32) -> Self {
        let mut lzc  = num.leading_zeros() as usize;
        let ones = calculate_msb_ones(num);
        let zeros = calculate_msb_zeros(num);
        let mut remaining = if lzc != 32 {
            32 - (lzc + ones + zeros)
        } else {0};
        if lzc == 0 && remaining == 1 {
            lzc = 1; remaining = 0;
        }
        let residuallength = 32 - lzc;
        GrayCodeAnalysis{num:*num, lzc, ones, zeros, remaining, residuallength}
    }
}

impl std::fmt::Display for GrayCodeAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:032b}, {}, {}, {}, {}, {}", self.num, self.lzc, self.ones, self.zeros, self.remaining, self.residuallength)
    }
}


fn calculate_msb_ones(num: &u32) -> usize {
    let next =  if (*num).leading_zeros() == 0 {
        1 << 31
    } else {
        (*num).next_power_of_two()
    };
    let mut pos = 1;
    while ((next >> pos) & num) != 0 {
        pos += 1;
    }
    // println!("Ones {}", pos - 1);
    pos - 1
}


fn calculate_msb_zeros(num: &u32) -> usize {
    if *num == 0 {
        return 0
    }
    let next =  if (*num).leading_zeros() == 0 {
        1 << 31
    } else {
        (*num).next_power_of_two()
    };
    let mut pos = 1;

    while ((next >> pos) & num) != 0 {
        pos += 1;
    }
    let tmp = pos;
    if next >> pos == 0 {
        return 0
    }
    // println!("TMP {}", tmp);
    while pos < 32 && ((next >> pos) & num) == 0 {
        if pos != 32 && next >> pos == 0 {
            break
        }
        pos += 1;
        // println!("pos {}", pos);
    }
    // println!("Zeros {}", pos - tmp);
    pos - tmp
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

pub fn analyse_file(file: &String) {
    println!("FILE: {}", file);
    let data = read_u32(file);
    let data : Vec<u32> = data.iter().filter(|x| ((**x) != 0u32)).map(|&x| x).collect();
    // println!("num, lzc, foc, fzc, remaining, residuallength");
    // for x in data.iter() {
    //     let v = GrayCodeAnalysis::new(x);
    //     println!("{}", v);
    // }
    positions_by_length(&data, 1, 7);
    positions(&data);
    println!("SIZE: {}", data.len());
}

use std::collections::{HashMap, BTreeMap};
fn positions_by_length(data: &Vec<u32>, min: u32, max: u32) {
    let mut counter = [0usize; 1 << 6];
    for n in min..max {
        for value in data.iter() {
            let v  = get_value_first(value, n) as usize;
            counter[v] += 1;
        }
    }
    let countermap : BTreeMap<usize, usize> = counter.iter().enumerate().map(|(u,&k)| (u,k)).collect();
    println!("BINARY: {:?}", countermap);
    // for (k,v) in countermap.into_iter() {
    //     println!("{:b}: {} ({:.2}%)", k, v, ((100*v) as f64 / data.len() as f64))
    // }
}


pub fn positions(data: &Vec<u32>) {
    let mut counter = [0usize; 32];
    for value in data.iter() {
        for i in 0..32 {
            counter[i] +=  ((*value & 1 << i) > 0) as usize
        }
    }
    let countermap : BTreeMap<usize, usize> = counter.iter().enumerate().map(|(u,&k)| (u,k)).collect();
    println!("POSITIONS: {:?}", countermap);
}

/// This will return the first n bits of the value in binary representation
/// If the value is smaller than 1 << n , than addtional zeros will be added
/// at the LSB positions
pub fn get_value_first(value: &u32, n: u32) -> u32 {
    let val = if *value <= (1 << n) { *value << n + 1 } else {*value};
    let filter = ((1<<n) - 1) << 32 - val.leading_zeros() - n;
    (val & filter) >> (32 - val.leading_zeros() - n)
}

/// This wil return the residual plus an additional bit at the front.
/// This is necessary for getting all the first zeros
pub fn get_value_after(value: &u32, n: u32) -> Option<u32> {
    if *value <= (1 << n) {
        return None
    }
    let filter = 32 - (*value).leading_zeros() - n;
    let range = (1 << filter) - 1;
    Some((range & *value) + range + 1)
}

/// This will return the number of bits in the value after the first n bits.
/// If the value is smaller than 1 << n, the result will be 0
pub fn get_residual_size_after(value: &u32, n: u32) -> u32 {
    if *value <= (1 << n) {
        return 0
    }
    32 - (*value).leading_zeros() - n - 1
}
