use std::mem;

use bitvec::prelude::*;
use bitvec::{
    bitvec,
    vec::{self, BitVec},
};
use rand::{distributions::Uniform, Rng};

static HASH_COMPLEXITY: usize = 16;

#[derive(Debug)]
pub struct USSL {
    data: Vec<u16>,
    full_bin: BitVec,
    list_size: usize,
    num_hash_funcs: usize,
    num_bins: usize,
    bin_size: usize,
    hash_func_data: Vec<u16>,
    num_insertions_in_bin: Vec<usize>,
}

impl USSL {
    pub fn new(list_size: usize, num_bins: usize, num_hash_funcs: usize, bin_size: usize) -> USSL {
        let data: Vec<u16> = vec![u16::MAX; num_bins * bin_size * list_size];
        let full_bin = bitvec![0;num_bins];

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, u16::MAX - 1);
        let hash_func_data: Vec<u16> = (0..(num_hash_funcs * HASH_COMPLEXITY))
            .map(|_| rng.sample(&range) as u16)
            .collect();
        let num_insertions_in_bin: Vec<usize> = vec![0; num_bins];
        return USSL {
            data,
            full_bin,
            list_size,
            num_hash_funcs,
            num_bins,
            bin_size,
            hash_func_data,
            num_insertions_in_bin,
        };
    }

    fn hash_elem(&self, elem: u16, hash_num: usize) -> usize {
        let index = (elem as usize) % HASH_COMPLEXITY;
        return (elem as usize)
            .wrapping_mul(self.hash_func_data[hash_num * HASH_COMPLEXITY + index] as usize);
    }

    fn hash_list(&self, list: &Vec<u16>, hash_num: usize) -> usize {
        let mut out: usize = 0;
        for i in 0..self.list_size {
            out = out.wrapping_add(self.hash_elem(list[i], hash_num));
        }
        return out;
    }

    fn get(&self, bin_num: usize, list_num: usize, offset: usize) -> u16 {
        return self.data
            [bin_num * self.bin_size * self.list_size + list_num * self.list_size + offset];
    }

    fn put(&mut self, bin_num: usize, list_num: usize, offset: usize, element: u16) {
        self.data[bin_num * self.bin_size * self.list_size + list_num * self.list_size + offset] =
            element;
    }

    fn insert_list_in_bin(&mut self, list: &Vec<u16>, bin_num: usize) {
        let current_insertion_index = self.num_insertions_in_bin[bin_num] % self.bin_size;

        for i in 0..self.list_size {
            self.put(bin_num, current_insertion_index, i, list[i]);
        }
        self.num_insertions_in_bin[bin_num] += 1;
    }

    pub fn insert_list(&mut self, list: &Vec<u16>) {
        //assert!(!self.is_present(list));
        
        let mut best_bin = 0;
        let mut best_num_inserts = usize::MAX;

        for hash_num in 0..self.num_hash_funcs {
            let bin = self.hash_list(list, hash_num) % self.num_bins;
            let num_inserts_in_bin = self.num_insertions_in_bin[bin];
            if num_inserts_in_bin < best_num_inserts {
                best_bin = bin;
                best_num_inserts = num_inserts_in_bin;
            }
        }

        self.insert_list_in_bin(list, best_bin);
    }

    fn list_present_in_bin(&self, list: &Vec<u16>, bin_num: usize) -> bool {
        for list_num in 0..self.bin_size {
            for offset in 0..self.list_size {
                if list[offset] != self.get(bin_num, list_num, offset) {
                    break;
                }

                if offset == self.list_size - 1 {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn is_present(&mut self, list: &Vec<u16>) -> bool {
        for hash_num in 0..self.num_hash_funcs {
            let bin = self.hash_list(list, hash_num) % self.num_bins;

            if self.list_present_in_bin(list, bin) {
                return true;
            }
        }
        return false;
    }

    pub fn clear_all(&mut self) {
        unsafe {
            libc::memset(
                self.data.as_mut_ptr() as _,
                0xffff,
                self.data.len() * mem::size_of::<u16>(),
            );
        }
    }
}
