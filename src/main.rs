use std::fs;

fn main() {
    // Start by reading the file as bytes
    let file_data_result = fs::read("./test_docs/test.txt");
    let original_file_data = match file_data_result {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };
    // Multiply length by 8 to go from bytes length to bits length
    let original_bit_length: u64 = (original_file_data.len() as u64) * 8;

    let mut file_data = original_file_data.clone();

    // Add a 1 bit followed by 7 zero bits
    file_data.push(0x80);

    // Add 0 bits until we are 64 bits shy of a 512 multiple
    let file_data_length = file_data.len();
    const REQUIRED_REMAINDER: usize = 56; // 56 bytes == 448 bit
    let mut remainder = file_data_length % 64; // 64 bytes == 512 bits

    while remainder != REQUIRED_REMAINDER {
        file_data.push(0x00);
        remainder = (remainder + 1) % 64;
    }

    // Append length of the ORIGINAL, unpadded message as a 64 bit int
    file_data.extend_from_slice(&original_bit_length.to_le_bytes());

    // Init registers
    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xEFCDAB89;
    let mut c0: u32 = 0x98BADCFE;
    let mut d0: u32 = 0x10325476;

    // Init shifts
    const SHIFTS: [u32; 64] = [7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  // 0..15
        5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  // 16..31
        4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  // 32..47
        6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21]; // 48..63

    // Init constants
    const K: [u32; 64] = [ 0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,  // 0..3
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,  // 4..7
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,  // 8..11
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,  // 12..15
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,  // 16..19
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,  // 20..23
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,  // 24..27
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,  // 28..31
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,  // 32..35
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,  // 36..39
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,  // 40..43
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,  // 44..47
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,  // 48..51
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,  // 52..55
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,  // 56..59
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391];  // 60..63


    // Loop over 512 bit blocks one at a time
    const CHUNK_SIZE: usize = 64; // 64 bytes == 512 bits
    const M_SIZE: usize = 4;  // 4 bytes == 32 bits
    for chunk in file_data.chunks(CHUNK_SIZE) {
        // Break up into 16 u32 little-endian words
        let m: Vec<u32> = chunk.chunks(M_SIZE)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();

        let mut a = a0.clone();
        let mut b = b0.clone();
        let mut c = c0.clone();
        let mut d = d0.clone();

        // Perform 64 operations
        for i in 0..64 {
            let mut f: u32;
            let g: u32;

            if (0..16).contains(&i) {
                // F := (B and C) or ((not B) and D)
                // g := i
                f = (b & c) | ((!b) & d);
                g = i.clone();
            } else if (16..32).contains(&i) {
                // F := (D and B) or ((not D) and C)
                // g := (5×i + 1) mod 16
                f = (d & b) | ((!d) & c);
                g = (5 * i + 1) % 16;
            } else if (32..48).contains(&i) {
                // F := B xor C xor D
                // g := (3×i + 5) mod 16
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                // F := C xor (B or (not D))
                // g := (7×i) mod 16
                f = c ^ (b | (!d));
                g = (7 * i) % 16;
            }

            // F := F + A + K[i] + M[g]
            f = f.wrapping_add(a).wrapping_add(K[i as usize]).wrapping_add(m[g as usize]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(SHIFTS[i as usize]));
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    // Calculate the final digest
    let digest = [a0, b0, c0, d0].iter()
        .flat_map(|x| x.to_le_bytes())
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    println!("MD5 CHECKSUM: {}", digest);
}
