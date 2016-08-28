use length_encode::EncodedLength;
use length_encode::{encode_lengths, huffman_lengths_from_frequency, COPY_PREVIOUS,
                    REPEAT_ZERO_3_BITS, REPEAT_ZERO_7_BITS};
use huffman_table::{create_codes, NUM_LITERALS_AND_LENGTHS, NUM_DISTANCE_CODES};
use BitWriter;

// The output ordering of the lenghts for the huffman codes used to encode the lenghts
// used to build the full huffman tree for length/literal codes.
const HUFFMAN_LENGTH_ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14,
                                        1, 15];

// Number of bits used for the values specifying the number of codes
const HLIT_BITS: u8 = 5;
const HDIST_BITS: u8 = 5;
const HCLEN_BITS: u8 = 4;

// The longest a huffman code describing another huffman length can be
const MAX_HUFFMAN_CODE_LENGTH: usize = 7;

pub fn write_huffman_lengths(literal_len_lengths: &[u8],
                             distance_lenghts: &[u8],
                             writer: &mut BitWriter) {
    assert!(literal_len_lengths.len() <= NUM_LITERALS_AND_LENGTHS);
    assert!(distance_lenghts.len() <= NUM_DISTANCE_CODES);
    // Number of length codes - 257
    let hlit = (literal_len_lengths.len() - 257) as u16;
    writer.write_bits(hlit, HLIT_BITS);
    // Number of distance codes - 1
    let hdist = (distance_lenghts.len() - 1) as u16;
    writer.write_bits(hdist, HDIST_BITS);

    // Encode length values
    let (encoded_ll, ll_freqs) = encode_lengths(literal_len_lengths).unwrap();
    let (encoded_d, d_freqs) = encode_lengths(distance_lenghts).unwrap();

    // Add together frequencies of length and distance tables for generating codes for them as they
    // use the same codes
    // TODO: Avoid dynamic memory allocation here
    // TODO: repeats can cross over from lit/len to distances, so we should do this to save a few
    // bytes
    let merged_freqs: Vec<usize> = ll_freqs.iter()
        .zip(d_freqs.iter())
        .map(|(l, d)| (l + d) as usize)
        .collect();

    let huffman_table_lengths = huffman_lengths_from_frequency(merged_freqs.as_slice(),
                                                               MAX_HUFFMAN_CODE_LENGTH);

    // Number of huffman table lengths - 4
    let hclen = huffman_table_lengths.len() - 4;

    writer.write_bits(hclen as u16, HCLEN_BITS);

    println!("hclen: {}", hclen);

    // Write the lengths for the huffman table describing the huffman table
    // Each length is 3 bits
    for n in HUFFMAN_LENGTH_ORDER.iter() {
        writer.write_bits(huffman_table_lengths[usize::from(*n)] as u16, 3);
        println!("writing length {}: {}",
                 *n,
                 huffman_table_lengths[usize::from(*n)]);
    }

    // Generate codes for the main huffman table using the lenghts we just wrote
    let codes = create_codes(&huffman_table_lengths).unwrap();

    // Write the actual huffman lengths
    for v in encoded_ll.into_iter().chain(encoded_d) {
        match v {
            EncodedLength::Length(n) => {
                let code = codes[usize::from(n)];
                writer.write_bits(code.code, code.length);
                println!("Writing code n: {}: code: {:b}, {}",
                         n,
                         code.code,
                         code.length);
            }
            EncodedLength::CopyPrevious(n) => {
                let code = codes[COPY_PREVIOUS];
                writer.write_bits(code.code, code.length);
                assert!(n >= 3);
                writer.write_bits((n - 3).into(), 2);
                println!("Writing copyPrevious({}), code: {:b}, {}",
                         n,
                         code.code,
                         code.length);
            }
            EncodedLength::RepeatZero3Bits(n) => {
                let code = codes[REPEAT_ZERO_3_BITS];
                writer.write_bits(code.code, code.length);
                assert!(n >= 3);
                writer.write_bits((n - 3).into(), 3);
            }
            EncodedLength::RepeatZero7Bits(n) => {
                let code = codes[REPEAT_ZERO_7_BITS];
                writer.write_bits(code.code, code.length);
                assert!(n >= 11);
                writer.write_bits((n - 11).into(), 7);
            }
        }
    }
}