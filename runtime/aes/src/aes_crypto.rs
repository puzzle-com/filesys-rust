use constants::*;

#[derive(Copy,Clone)]
pub struct Key {
    data: [u8; 16]
}

impl Key {
    pub fn new(data: &[u8; 16]) -> Self {
        Key { data: data.clone() }
    }

    fn expand(&self) -> [Self; 11] {
        let mut keys: [[u8; 4]; 44] = [[0; 4]; 44]; // table of columns

        load_initial_key(&mut keys, &self.data);

        for i in 1..11 {
            ksa_core(&mut keys, i, i*4);
            expand_column(&mut keys, i*4+1);
            expand_column(&mut keys, i*4+2);
            expand_column(&mut keys, i*4+3);
        }

        return columns_to_keys(&keys);

        fn load_initial_key(keys: &mut [[u8;4];44], data: &[u8;16]) {
            let (a, b, c, d) = array_refs![data,4,4,4,4];
            keys[0] = a.clone();
            keys[1] = b.clone();
            keys[2] = c.clone();
            keys[3] = d.clone();
        }

        fn ksa_core(keys: &mut [[u8;4];44], i: usize, column: usize) {
            keys[column][0] = SBOX[keys[column-1][1] as usize];
            keys[column][1] = SBOX[keys[column-1][2] as usize];
            keys[column][2] = SBOX[keys[column-1][3] as usize];
            keys[column][3] = SBOX[keys[column-1][0] as usize];

            keys[column][0] ^= RCON[i];

            keys[column][0] ^= keys[column-4][0];
            keys[column][1] ^= keys[column-4][1];
            keys[column][2] ^= keys[column-4][2];
            keys[column][3] ^= keys[column-4][3];
        }

        fn expand_column(keys: &mut [[u8;4];44], column: usize) {
            keys[column][0] = keys[column-4][0] ^ keys[column-1][0];
            keys[column][1] = keys[column-4][1] ^ keys[column-1][1];
            keys[column][2] = keys[column-4][2] ^ keys[column-1][2];
            keys[column][3] = keys[column-4][3] ^ keys[column-1][3];
        }

        fn columns_to_keys(columns: &[[u8;4];44]) -> [Key; 11] {
            let mut keys = [Key { data: [0;16] }; 11];
            for i in 0..11 {
                keys[i].data[0..4].copy_from_slice(&columns[i*4] as &[u8]);
                keys[i].data[4..8].copy_from_slice(&columns[i*4 + 1] as &[u8]);
                keys[i].data[8..12].copy_from_slice(&columns[i*4 + 2] as &[u8]);
                keys[i].data[12..16].copy_from_slice(&columns[i*4 + 3] as &[u8]);
            }
            return keys;
        }
    }
}


#[derive(Eq,PartialEq,Clone,Copy,Debug)]
pub struct Block {
    data: [u8; 16]
}

impl Block {
    pub fn new(data: &[u8; 16]) -> Block {
        Block { data: data.clone() }
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.data
    }

    fn add_round_key(&mut self, key: &Key) {
        for i in 0..16 {
            self.data[i] ^= key.data[i];
        }
    }

    fn sub_bytes(&mut self) {
        for i in 0..16 {
            self.data[i] = SBOX[self.data[i] as usize];
        }
    }

    fn inv_sub_bytes(&mut self) {
        for i in 0..16 {
            self.data[i] = INV_SBOX[self.data[i] as usize];
        }
    }

    fn shift_rows(&mut self) {
        let mut new_data = self.data.clone();
        for row in 1..4 {
            for col in 0..4 {
                new_data[col*4 + row] = self.data[((col + row)*4 + row) % 16]
            }
        }
        self.data = new_data;
    }

    fn inv_shift_rows(&mut self) {
        let mut new_data = self.data.clone();
        for row in 1..4 {
            for col in 0..4 {
                new_data[col*4 + row] = self.data[((col + 4 - row)*4 + row) % 16]
            }
        }
        self.data = new_data;
    }

    fn mix_columns(&mut self) {
        mix_column(&mut self.data[0..4]);
        mix_column(&mut self.data[4..8]);
        mix_column(&mut self.data[8..12]);
        mix_column(&mut self.data[12..16]);

        fn mix_column(col: &mut[u8]) {
            let mut c: [u8; 4] = [0; 4];
            c.copy_from_slice(col);

            col[0] = MUL2[c[0] as usize] ^ MUL3[c[1] as usize] ^ c[2] ^ c[3];
            col[1] = c[0] ^ MUL2[c[1] as usize] ^ MUL3[c[2] as usize] ^ c[3];
            col[2] = c[0] ^ c[1] ^ MUL2[c[2] as usize] ^ MUL3[c[3] as usize];
            col[3] = MUL3[c[0] as usize] ^ c[1] ^ c[2] ^ MUL2[c[3] as usize];
        }
    }

    fn inv_mix_columns(&mut self) {
        inv_mix_column(&mut self.data[0..4]);
        inv_mix_column(&mut self.data[4..8]);
        inv_mix_column(&mut self.data[8..12]);
        inv_mix_column(&mut self.data[12..16]);

        fn inv_mix_column(col: &mut[u8]) {
            let mut c: [u8; 4] = [0; 4];
            c.copy_from_slice(col);

            col[0] = MUL14[c[0] as usize] ^ MUL11[c[1] as usize]
                    ^ MUL13[c[2] as usize] ^ MUL9[c[3] as usize];
            col[1] = MUL9[c[0] as usize] ^ MUL14[c[1] as usize]
                    ^ MUL11[c[2] as usize] ^ MUL13[c[3] as usize];
            col[2] = MUL13[c[0] as usize] ^ MUL9[c[1] as usize]
                    ^ MUL14[c[2] as usize] ^ MUL11[c[3] as usize];
            col[3] = MUL11[c[0] as usize] ^ MUL13[c[1] as usize]
                    ^ MUL9[c[2] as usize] ^ MUL14[c[3] as usize];
        }
    }
}


pub fn encrypt(key: Key, block: Block) -> Block {
    let mut state = block.clone();
    let keys: [Key; 11] = key.expand();

    state.add_round_key(&keys[0]);

    for i in 1..10 {
        state.sub_bytes();
        state.shift_rows();
        state.mix_columns();
        state.add_round_key(&keys[i]);
    }

    state.sub_bytes();
    state.shift_rows();
    state.add_round_key(&keys[10]);

    return state;
}


pub fn decrypt(key: Key, block: Block) -> Block {
    let mut state = block.clone();
    let keys: [Key; 11] = key.expand();

    state.add_round_key(&keys[10]);

    for i in 1..10 {
        state.inv_shift_rows();
        state.inv_sub_bytes();
        state.add_round_key(&keys[10-i]);
        state.inv_mix_columns();
    }

    state.inv_shift_rows();
    state.inv_sub_bytes();
    state.add_round_key(&keys[0]);

    return state;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_decryption_test() {
        let key = Key { data: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] };
        let message = Block { data: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] };

        let result = decrypt(key, encrypt(key, message));

        assert_eq!(message, result);
    }

    #[test]
    fn encryption_test() {
        let key = Key { data: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] };
        let message = Block { data: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] };
        let expected = Block { data: [0x0a,0x94,0x0b,0xb5,0x41,0x6e,0xf0,0x45
                                     ,0xf1,0xc3,0x94,0x58,0xc6,0x53,0xea,0x5a
                                     ]
                             };
        let encrypted = encrypt(key, message);

        assert_eq!(expected, encrypted);
    }

    #[test]
    fn key_schedule_test() {
        let key = Key { data: [ 0x2b, 0x7e, 0x15, 0x16
                              , 0x28, 0xae, 0xd2, 0xa6
                              , 0xab, 0xf7, 0x15, 0x88
                              , 0x09, 0xcf, 0x4f, 0x3c
                              ]};
        let expected = [ 0xa0, 0xfa, 0xfe, 0x17
                       , 0x88, 0x54, 0x2c, 0xb1
                       , 0x23, 0xa3, 0x39, 0x39
                       , 0x2a, 0x6c, 0x76, 0x05
                       ];

        let expanded = key.expand()[1];

        assert_eq!(expected, expanded.data);
    }

    #[test]
    fn mix_columns_test() {
        let mut block = Block { data:
                                [ 219, 19, 83, 69
                                , 1, 1, 1, 1
                                , 198, 198, 198, 198
                                , 45, 38, 49, 76
                                ] };
        let expected = [ 142, 77, 161, 188
                       , 1, 1, 1, 1
                       , 198, 198, 198, 198
                       , 77, 126, 189, 248
                       ];
        block.mix_columns();
        assert_eq!(expected, block.data);
    }

    #[test]
    fn inverse_mix_columns_test() {
        let mut block = Block { data:
                                [ 219, 19, 83, 69
                                , 1, 1, 1, 1
                                , 198, 198, 198, 198
                                , 45, 38, 49, 76
                                ] };
        let expected = block.clone();

        block.mix_columns();
        block.inv_mix_columns();

        assert_eq!(expected, block);
    }

    #[test]
    fn shift_rows_test() {
        let mut block = Block { data:
                                [ 1, 5, 9, 13
                                , 2, 6, 10, 14
                                , 3, 7, 11, 15
                                , 4, 8, 12, 16
                                ] };
        let expected = [ 1, 6, 11, 16
                       , 2, 7, 12, 13
                       , 3, 8, 9, 14
                       , 4, 5, 10, 15
                       ];
        block.shift_rows();
        assert_eq!(expected, block.data);
    }

    #[test]
    fn inverse_shift_rows_test() {
        let mut block = Block { data:
                                [ 1, 6, 11, 16
                                , 2, 7, 12, 13
                                , 3, 8, 9, 14
                                , 4, 5, 10, 15
                                ] };
        let expected = [ 1, 5, 9, 13
                       , 2, 6, 10, 14
                       , 3, 7, 11, 15
                       , 4, 8, 12, 16
                       ];
        block.inv_shift_rows();
        assert_eq!(expected, block.data);
    }
}
