use super::graycodeanalysis::read_u32;
use pzip_huffman::hufbites::encode_itself_to_bytes;
use super::foc::vec_diff;
use std::collections::HashMap;

pub fn mtf(matches: &clap::ArgMatches) {
    let ifile = String::from(matches.value_of("input").unwrap());
    let data = read_u32(&ifile);

    let mut base : HashMap<String, Vec<u8>> = HashMap::new();
    base.insert("foc".to_string(), get_foc(&data));
    base.insert("lzc".to_string(), get_lzc(&data));
    base.insert("lzcfoc".to_string(), get_lzc_and_foc(&data));

    let mut compressed : HashMap<String, usize> = HashMap::new();
    for k in base.iter() {
        let c = encode_itself_to_bytes(&k.1).0.len();
        let mut name = k.0.clone();
        name.push_str("_huff");
        compressed.insert(name, c);
    }
    for k in base.iter() {
        let c = encode_itself_to_bytes(&vec_diff(k.1)).0.len();
        let mut name = k.0.clone();
        name.push_str("_diff_huff");
        compressed.insert(name, c);
    }

    let count_vec: Vec<_> = compressed.iter().collect();
    println!("{:#?}", count_vec)

    // for (h,d) in
    // let huff_lzc = encode_itself_to_bytes(&lzc).0.len();
    // let huff_lzcfoc = encode_itself_to_bytes(&lzcfoc).0.len();

    // let huff_foc = encode_itself_to_bytes(&foc).0.len();
    // let huff_lzc = encode_itself_to_bytes(&lzc).0.len();
    // let huff_lzcfoc = encode_itself_to_bytes(&lzcfoc).0.len();


    // println!("              Huff");
    // println!("foc:    {:>10}", huff_foc);
    // println!("lzc:    {:>10}", huff_lzc);
    // println!("lzcfoc: {:>10}", huff_lzcfoc);

}

fn get_foc(data: &Vec<u32>) -> Vec<u8> {
    data.iter().filter(|&&x| x != 0).map(|&x| _foc(&x)).collect::<Vec<u8>>()
}

fn get_fzc(data: &Vec<u32>) -> Vec<u8> {
    data.iter().filter(|&&x| x != 0).map(|&x| _fzc(&x)).collect::<Vec<u8>>()
}

fn get_lzc(data: &Vec<u32>) -> Vec<u8> {
    data.iter()
        .map(|&x| x.leading_zeros() as u8)
        .collect::<Vec<u8>>()
}

fn get_lzc_and_foc(data: &Vec<u32>) -> Vec<u8> {
    data.iter()
        .map(|&x| {
            let gf = _foc(&x);
            if gf > 0 {
                (x.leading_zeros() + 33 + _foc(&x) as u32) as u8
            } else {
                x.leading_zeros() as u8
            }
        })
        .collect::<Vec<u8>>()
}

fn get_lzc_and_fzc(data: &Vec<u32>) -> Vec<u8> {
    data.iter()
        .map(|&x| {
            let gf = _fzc(&x);
            if gf > 0 {
                (x.leading_zeros() + 33 + _fzc(&x) as u32) as u8
            } else {
                x.leading_zeros() as u8
            }
        })
        .collect::<Vec<u8>>()
}

fn _foc(val: &u32) -> u8 {
    let mut result = 32 - (*val).leading_zeros();
    let mut ix = 0;
    while result > 0 && !((*val >> ix) + 1).is_power_of_two() {
        ix += 1;
        result -= 1
    }
    result as u8
}

fn _fzc(val: &u32) -> u8 {
    if *val == 0 {
        return 0;
    }
    let mut value = *val;
    let mut ix = 0u8;
    while !value.is_power_of_two() {
        ix += 1;
        value >>= 1
    }
    32 - val.leading_zeros() as u8 - ix - 1
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_fzc() {
        let data: Vec<u32> = vec![32, 5345, 0, 21321, 0696, 3837, 1 << 31, 1 << 3, 9283];
        let result: Vec<u8> = data.iter().map(|&x| _fzc(&x)).collect();
        let expected: Vec<u8> = vec![5, 1, 0, 1, 1, 0, 31, 3, 2];

        assert_eq!(result, expected);
    }
}
