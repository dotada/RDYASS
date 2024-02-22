use qrcode_generator::QrCodeEcc;
use std::fs::{File, OpenOptions};
use std::io::{stdin, Read, Seek, SeekFrom, Write, stdout};
use std::{fs, io};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use num_cpus;
use rayon::ThreadPoolBuilder;
use rxing::BarcodeFormat;

fn splitfile(chunksize: u64, mut file: &File) -> io::Result<()> {
    let fsize = file.metadata()?.len();
    let splits = fsize / chunksize;
    let mut index = 1;
    let mut readfrom = 0;

    while index < splits + 1 {
        file.seek(SeekFrom::Start(readfrom))?;
        let mut chunk = Vec::with_capacity(chunksize as usize);
        file.take(chunksize).read_to_end(&mut chunk)?;
        let mut output = File::create(format!("output_{}.bin", index))?;
        output.write_all(&chunk)?;
        index += 1;
        readfrom += chunksize;
    }

    let lastread = fsize - readfrom;
    file.seek(SeekFrom::End(-(lastread as i64)))?;
    let mut chunk = Vec::with_capacity(lastread as usize);
    file.read_to_end(&mut chunk)?;
    let mut output = File::create(format!("output_{}.bin", index))?;
    output.write_all(&chunk)?;
    println!("Chunks: {}", index);
    Ok(())
}

fn readlinetrim() -> String {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    s = s.trim().parse().unwrap();
    return s;
}
/*
fn combinefile(chunks: u64, outputfile: &String) -> io::Result<()> {
    let mut index = 1;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(outputfile)
        .unwrap();
    while index <= chunks {
        let mut chunkfile = File::open(format!("output_{}.bin", index))?;
        let mut chunk = Vec::new();
        chunkfile.read_to_end(&mut chunk)?;
        file.write_all(&chunk)?;
        fs::remove_file(format!("output_{}.bin", index))?;
        index += 1;
    }
    Ok(())
}
*/
fn split() -> io::Result<()> {
    print("Chunk size: ")?;
    let chunksize = readlinetrim();
    print("Target file: ")?;
    let filepath = readlinetrim();
    let file = File::open(filepath)?;
    let chunksizenum: u64 = chunksize.parse().unwrap();
    splitfile(chunksizenum, &file)?;
    Ok(())
}

fn print(string: &str) -> io::Result<()> {
    print!("{}", string);
    stdout().flush()?;
    Ok(())
}
/*
fn combine() -> io::Result<()> {
    print("Output filename: ")?;
    let outputf = readlinetrim();
    print("Chunks: ")?;
    let chunks = readlinetrim().parse().unwrap();
    combinefile(chunks, &outputf)?;
    Ok(())
}
*/
fn encode() -> io::Result<()> {
    print("Chunks: ")?;
    let chunkss = readlinetrim();
    let chunks = chunkss.parse().unwrap();

    let num_threads = num_cpus::get() - 1;

    ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    (1..=chunks).into_par_iter().for_each(|index| {
        let bytes = fs::read(format!("output_{}.bin", index)).unwrap();
        let file = File::open(format!("output_{}.bin", index)).unwrap();
        qrcode_generator::to_png_to_file(bytes, QrCodeEcc::Low, file.metadata().unwrap().len() as usize, format!("qr_{}.png", index)).unwrap();
        fs::remove_file(format!("output_{}.bin", index)).unwrap();
    });

    Ok(())
}

fn decode() -> io::Result<()> {
    print("Codes: ")?;
    let chunkss = readlinetrim();
    let chunks = chunkss.parse().unwrap();
    print("Output file: ")?;
    let outf = readlinetrim();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(outf)
        .unwrap();
    let mut index = 1;
    println!();
    while index <= chunks {
        print("\r")?;
        let str = format!("Decoded: {}/{}", index, chunks);
        print(str.as_str())?;
        let results = rxing::helpers::detect_in_file(&*format!("qr_{}.png", index), Option::from(BarcodeFormat::QR_CODE));
        file.write_all(results.unwrap().getRawBytes())?;
        fs::remove_file(format!("qr_{}.png", index)).unwrap();
        index += 1;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    print("split, decode or encode: ")?;
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    s = s.trim().parse().unwrap();
    match s.as_str() {
        "split"=>split()?,
        "encode"=>encode()?,
        "decode"=>decode()?,
        _=>println!("Invalid option."),
    }
    Ok(())
}
