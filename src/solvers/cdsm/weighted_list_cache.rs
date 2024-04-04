use std::mem;

use rand::{distributions::Uniform, Rng};

static HASH_COMPLEXITY: usize = 16;

#[derive(Debug)]
pub struct WLC {
    data: Vec<u16>,
    list_size: usize,
    num_hash_funcs: usize,
    num_bins: usize,
    bin_size: usize,
    hash_func_data: Vec<u16>,
    score_multiplier : Vec<usize>,
    score: Vec<usize>
}

impl WLC {
    pub fn new(list_size: usize, num_bins: usize, num_hash_funcs: usize, bin_size: usize) -> WLC {
        let data: Vec<u16> = vec![u16::MAX; num_bins * bin_size * list_size];

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, u16::MAX - 1);
        let hash_func_data: Vec<u16> = (0..(num_hash_funcs * HASH_COMPLEXITY))
            .map(|_| rng.sample(&range) as u16)
            .collect();
        return WLC {
            data,
            list_size,
            num_hash_funcs,
            num_bins,
            bin_size,
            hash_func_data,
            score_multiplier: vec![0;num_bins * bin_size],
            score: vec![0;num_bins * bin_size]
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

    fn insert_list_in_bin(&mut self, list: &Vec<u16>, bin_num: usize, bin_offset: usize, score_multiplier: usize) {
        
        for i in 0..self.list_size {
            self.put(bin_num, bin_offset, i, list[i]);
        }
        self.score[bin_num* self.bin_size + bin_offset] = score_multiplier;
        self.score_multiplier[bin_num* self.bin_size + bin_offset] = score_multiplier;
    }

    pub fn insert_list(&mut self, list: &Vec<u16>, list_score: usize) {
        //assert!(!self.is_present(list));

        let mut best_bin = 0;
        let mut lowest_score = list_score;
        let mut bin_offset = 0;

        for hash_num in 0..self.num_hash_funcs {
            let bin = self.hash_list(list, hash_num) % self.num_bins;
            for li in 0..self.bin_size {
                let score = self.score[bin* self.bin_size + li];
                if score < lowest_score {
                    best_bin = bin;
                    lowest_score = score;
                    bin_offset = li;
                }
            }
        }

        if lowest_score == list_score{
            for hash_num in 0..self.num_hash_funcs {
                let bin = self.hash_list(list, hash_num) % self.num_bins;
                self.decrease_scores(bin);
            }
            return;
        }

        self.insert_list_in_bin(list, best_bin, bin_offset, list_score);
    }

    fn list_present_in_bin(&mut self, list: &Vec<u16>, bin_num: usize) -> bool {
        for list_num in 0..self.bin_size {
            for offset in 0..self.list_size {
                if list[offset] != self.get(bin_num, list_num, offset) {
                    break;
                }

                if offset == self.list_size - 1 {
                    self.score[bin_num*self.bin_size + list_num] += self.score_multiplier[bin_num*self.bin_size + list_num];
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
    
    fn decrease_scores(&mut self, bin: usize) {
        for i in 0..self.bin_size{
            self.score[bin * self.bin_size + i] -=1;
        }
    }
}
