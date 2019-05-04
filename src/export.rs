use std::io::{BufWriter, Write};
use std::fs::File;
use super::graycodeanalysis::read_u32;
use super::mtf::{get_lzc, get_foc, get_lzc_and_foc, apply_bwt};

pub fn export(matches: &clap::ArgMatches) {
    let ifile = String::from(matches.value_of("input").unwrap());
    let data = read_u32(&ifile);

    let lzcfilename = ifile.clone() + &".lzc".to_string();
    let lzcfilename_bwt = ifile.clone() + &".lzc_bwt".to_string();
    let lzc = get_lzc(&data);
    let lzc_bwt = apply_bwt(&get_lzc(&data));
    write_u8(&lzc, lzcfilename);
    write_u8(&lzc_bwt, lzcfilename_bwt);

    let focfilename = ifile.clone() + &".foc".to_string();
    let focfilename_bwt = ifile.clone() + &".foc_bwt".to_string();
    let foc = get_foc(&data);
    let foc_bwt = apply_bwt(&get_foc(&data));
    write_u8(&foc, focfilename);
    write_u8(&foc_bwt, focfilename_bwt);

    let lzcfocfilename = ifile.clone() + &".lzcfoc".to_string();
    let lzcfocfilename_bwt = ifile.clone() + &".lzcfoc_bwt".to_string();
    let lzcfoc = get_lzc_and_foc(&data);
    let lzcfoc_bwt = apply_bwt(&get_lzc_and_foc(&data));
    write_u8(&lzcfoc, lzcfocfilename);
    write_u8(&lzcfoc_bwt, lzcfocfilename_bwt);
}

fn write_u8(data: &[u8], fname: String) {
    let mut output = BufWriter::new(File::create(fname).unwrap());
    output.write_all(data).unwrap();
}
