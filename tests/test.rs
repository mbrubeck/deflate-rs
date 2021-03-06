extern crate deflate;
extern crate flate2;

fn get_test_file_data(name: &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;
    let mut input = Vec::new();
    let mut f = File::open(name).unwrap();

    f.read_to_end(&mut input).unwrap();
    input
}

fn get_test_data() -> Vec<u8> {
    use std::env;
    let path = env::var("TEST_FILE").unwrap_or_else(|_| "tests/pg11.txt".to_string());
    get_test_file_data(&path)
}

// A test comparing the compression ratio of the library with flate2
#[test]
fn test_file_zlib_compare_output() {
    use flate2::Compression;
    use std::io::{Write, Read};
    use deflate::{CompressionOptions, deflate_bytes_zlib_conf};
    let test_data = get_test_data();
    let flate2_compressed = {
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), Compression::Best);
        e.write_all(&test_data).unwrap();
        e.finish().unwrap()
    };

    let deflate_compressed = deflate_bytes_zlib_conf(&test_data, CompressionOptions::high());

    // {
    //     use std::fs::File;
    //     use std::io::Write;
    //     {
    //         let mut f = File::create("out.deflate").unwrap();
    //         f.write_all(&deflate_compressed).unwrap();
    //     }
    //     {
    //         let mut f = File::create("out.flate2").unwrap();
    //         f.write_all(&flate2_compressed).unwrap();
    //     }
    // }

    println!("libflate: {}, deflate: {}",
             flate2_compressed.len(),
             deflate_compressed.len());
    let decompressed = {
        let mut d = flate2::read::ZlibDecoder::new(deflate_compressed.as_slice());
        let mut out = Vec::new();
        d.read_to_end(&mut out).unwrap();
        out
    };


    assert!(decompressed == test_data);
}
