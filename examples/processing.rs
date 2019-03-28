use std::io::Read;
use rgsl::statistics::correlation;
use log::{info,debug};
use std::{env, fs};

const BLOCKSIZE: usize = 128;
const THRESHOLD:   f64 = 0.9;

fn remove_and_push(val: &u8, pos: u8, vec: &mut Vec<f64>) {
    let num = *val & (1 << pos) > 0;
    let minus = vec.remove(0);
    for element in vec.iter_mut() {
        *element -= minus;
    }
    vec.push(vec.last().unwrap() + num as i32 as f64);
}


fn first_block(bytes: &Vec<u8>) -> Vec<f64> {
    let mut result = vec![0f64;BLOCKSIZE];
    for val in bytes.into_iter().take(BLOCKSIZE/8) {
        for j in 0..8 {
            remove_and_push(val, j, &mut result);
        }
    }
    result
}

fn calculate_correlation(one: &Vec<f64>, other: &Vec<f64>) -> f64 {
    correlation(one.as_slice(), 1, other.as_slice(), 1, one.len())
}

fn to_relative(vec: &Vec<f64>) -> Vec<f64> {
    vec.iter().map(|&a| a / BLOCKSIZE as f64).collect()
}

fn read_binary(filename: &String) -> Vec<u8> {
    let mut file = fs::File::open(filename).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut bytes).unwrap();
    bytes
}

fn main() {
    env_logger::init();
    let args : Vec<String> = env::args().collect();
    let bytes = read_binary(&args[1]);

    let mut current = first_block(&bytes);
    let first_candidate = to_relative(&current);

    let mut candidates : Vec<Vec<f64>> = Vec::new();
    candidates.push(first_candidate);

    for (i,val) in bytes.iter().enumerate().skip(BLOCKSIZE/8) {
        if i % 100_000 == 0 {
            info!("{} of {}", i, bytes.len());
        }
        'out: for j in 0..8 {
            remove_and_push(val, j, &mut current);
            let mut iter = candidates.iter().rev().skip_while(|x| calculate_correlation(x, &current) < THRESHOLD);
            if iter.next() == None {
                debug!("Adding: {}, because", i*8+j as usize);
                candidates.push(to_relative(&current));
            }
        }
    }

    formatted_output(&args[1], candidates);
}

fn formatted_output(filename: &String, candidates: Vec<Vec<f64>>) {
    println!("File {}", filename);
    let mut titles : Vec<String> = Vec::new();
    titles.push("ix".to_string());
    for i in 0..candidates.len() {
        titles.push(format!("unit{:03}", i));
    }
    println!("{}", titles.join(","));
    for i in 0..BLOCKSIZE {
        let mut line = i.to_string();
        for k in 0..candidates.len() {
            line = format!("{},{}", line, candidates[k][i])
        }
        println!("{}", line)
    }
}


#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_with_python() {
         let bytes : Vec<u8> = vec![201,124,53,204,149,106,17,228,241,130,205,118,242,181,14,205,74,17,
             96,135,188,213,219,7,94,203,61,245,43,205,243,50,149,246,126,154,
             99,71,137,26,96,143,206,203,66,254,3,147,231,114,23,210,20,196,
             66,64,123,239,71,29,126,228,14,86,29,164,40,27,54,110,114,64,
             139,110,197,75,0,227,68,216,108,186,82,172,219,79,210,80,239,246,
             8,51,68,114,73,136,0,4,6,83,180,0,50,6,233,234,214,10,
             250,3,153,209,209,65,127,41,225,202,23,165,181,136,226,58,251,159,
             8,245,33,134,51,19,39,227,17,2,24,255,84,14,77,166,32,7,
             118,238,77,240,245,229,128,250,5,25,86,58,167,121,157,13,89,28,
             74,228,14,196,163,25,119,233,214,201,145,22,208,185,147,255,95,86,
             23,204,9,112,239,7,60,240,4,188,150,241,138,33,82,238,36,0,
             173,98,233,50,193,92,92,254,211,5,229,6,103,55,239,246,227,18,
             222,46,254,152,130,141,42,175,85,133,215,120,130,158,121,78,139,188,
             115,192,33,68,68,206,143,163,249,160,144,65,77,210,186,193,101,3,
             61,200,183,227,94,178,188,46,185,47,187,248,216,147,225,186,122,14,
             223,196,193,143,35,57,194,218,11,214,127,52,149,11,143,16,125,197,
             171,77,211,124,126,207,55,27,239,135,55,121,198,66,109,20,165,207,
             234,16,196,233,191,87,39,192,205,92,163,130,144,252,171,47,117,14,
             65,70,15,96,13,52,62,131,71,64,94,251,64,104,23,67,251,195,
             228,226,196,78,86,95,227,178,98,28,154,205,22,64,224,251,88,128,
             209,61,71,23,226,71,198,136,178,209,2,153,57,206,192,255,247,51,
             40,222,208,19,129,25,39,126,148,52,137,195,184,58,188,127,215,49,
             121,26,129,199,149,119,238,165,66,59,45,149,129,239,74,248,15,235,
             255,22,28,153,51,151,158,86,67,249,16,227,216,23,182,110,94,163,
             222,4,91,147,1,81,170,184,154,199,153,112,15,227,22,255,151,236,
             234,250,177,197,103,31,108,58,89,113,55,72,75,33,139,13,18,247,
             140,2,87,248,246,114,209,101,99,185,193,69,171,203,141,101,246,203,
             113,105,247,137,66,244,64,104,254,77,49,148,171,204,194,80,8,146,
             217,14,26,89,234,234,254,4,147,196,252,34,227,26,35,247,63,236,
             24,189,184,6,92,90,165,202,219,54,205,123,18,184,40,190,188,172,
             125,34,153,137,63,91,59,172,172,222,124,57,126,242,85,60,28,150,
             224,79,209,205,128,86,118,164,152,1,247,41,170,221,188,33,181,153,
             182,45,75,124,196,173,179,254,95,209,45,216,175,164,125,223,17,162,
             71,109,27,36,235,128,15,24,76,64,215,169,47,23,201,173,234,185,
             111,84,128,180,150,189,231,91,202,80,243,180,178,152,107,65,174,157,
             135,160,85,150,107,93,18,89,118,121,73,57,249,160,151,212,99,118,
             42,77,61,33,182,199,32,118,45,31,34,108,164,246,102,215,228,54,
             156,61,90,115,137,191,219,193,82,80,222,36,219,171,33,40,233,14,
             210,65,77,84,134,215,135,134,62,211,133,31,151,64,90,248,253,148,
             63,7,254,187,148,67,140,204,110,169,42,167,146,11,117,163,0,183,
             98,146,212,219,228,81,152,159,105,86,33,219,40,129,29,82,2,90,
             41,89,198,130,244,70,253,201,191,24,144,90,93,42,241,69,130,74,
             244,174,34,108,95,68,211,51,118,95,81,29,140,33,55,146,100,17,
             34,78,171,241,239,226,192,110,45,120,115,100,43,103,49,195,2,219,
             191,215,141,216,68,33,75,51,15,146,106,175,183,44,79,164,16,18,
             209,51,55,179,24,63,157,122,15,129,76,78,145,76,196,65,20,92,
             128,119,242,77,203,107,175,134,4,96,219,132,56,24,230,101,151,106,
             189,67,205,167,183,155,56,106,100,213,147,145,226,71,130,207,24,20,
             46,23,191,55,228,120,29,79,175,39,22,111,59,98,114,227,151,99,
             16,171,243,1,59,151,77,12,132,169,92,152,198,89,43,212,100,65,
             255,165,97,221,54,48,229,248,233,197,244,154,170,184,37,201,210,240,
             107,145,22,67,193,167,23,112,52,60,54,148,34,161,248,138,105,106,
             54,219,106,126,164,15,108,65,121,11,111,89,221,9,130,227,172,249,
             232,45,185,227,199,42,202,92,3,48,212,179,148,19,151,209,79,236,
             52,105,80,49,139,103,221,16,35,193,212,245,159,198,17,184,133,225,
             2,212,96,162,57,97,123,161,124,148];
        assert!(false);
    }
}
