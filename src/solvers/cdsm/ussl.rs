use std::mem;

use rand::{distributions::Uniform, Rng};

use crate::solvers::cdsm::state_db::StateDB;

static HASH_COMPLEXITY: usize = 16;

#[derive(Debug)]
pub struct USSL {
    data: Vec<u16>,
    list_size: usize,
    num_hash_funcs: usize,
    num_bins: usize,
    bin_size: usize,
    hash_func_data: Vec<u16>,
    num_els_in_bin: Vec<usize>,
}

impl USSL {
    pub fn new(list_size: usize, num_bins: usize, num_hash_funcs: usize, bin_size: usize) -> USSL {
        let data: Vec<u16> = vec![u16::MAX; num_bins * bin_size * list_size];

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, u16::MAX - 1);
        let hash_func_data: Vec<u16> = (0..(num_hash_funcs * HASH_COMPLEXITY))
            .map(|_| rng.sample(&range) as u16)
            .collect();
        let num_els_in_bin: Vec<usize> = vec![0; num_bins];
        return USSL {
            data,
            list_size,
            num_hash_funcs,
            num_bins,
            bin_size,
            hash_func_data,
            num_els_in_bin: num_els_in_bin,
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
        return out % self.num_bins;
    }

    fn get(&self, bin_num: usize, list_num: usize, offset: usize) -> u16 {
        return self.data
            [bin_num * self.bin_size * self.list_size + list_num * self.list_size + offset];
    }

    fn put(&mut self, bin_num: usize, list_num: usize, offset: usize, element: u16) {
        self.data[bin_num * self.bin_size * self.list_size + list_num * self.list_size + offset] =
            element;
    }

    fn insert_list_in_bin(&mut self, list: &Vec<u16>, bin_num: usize, insertion_index: usize) {
        for i in 0..self.list_size {
            self.put(bin_num, insertion_index, i, list[i]);
        }

        self.num_els_in_bin[bin_num] = self.num_els_in_bin[bin_num].min(self.bin_size);
    }

    fn static_insert_list(&mut self, list: &Vec<u16>) -> bool {
        //assert!(!self.is_present(list));

        let mut list_to_insert = list.clone();
        let mut next_list_to_insert = vec![0; self.list_size];

        let mut hash_index_to_force = 0;
        let mut index_to_force = 0;
        let mut num_tries = 0;
        while num_tries < 100 {
            for hash_num in 0..self.num_hash_funcs {
                let bin = self.hash_list(&list_to_insert, hash_num);

                if self.num_els_in_bin[bin] < self.bin_size - 1 {
                    self.insert_list_in_bin(&list_to_insert, bin, self.num_els_in_bin[bin]);
                    self.num_els_in_bin[bin] += 1;
                    return true;
                }
            }

            let bin_to_force = self.hash_list(&list_to_insert, hash_index_to_force);
            for i in 0..self.list_size {
                next_list_to_insert[i] = self.get(bin_to_force, index_to_force, i);
            }

            self.insert_list_in_bin(&list_to_insert, bin_to_force, index_to_force);
            index_to_force += 1;
            hash_index_to_force += 1;
            index_to_force %= self.bin_size;
            hash_index_to_force %= self.num_hash_funcs;

            for i in 0..self.list_size {
                list_to_insert[i] = next_list_to_insert[i];
            }

            num_tries += 1;
        }
        return false;
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

    fn resize(&mut self) {
        //println!("resize");
        let old_data = self.data.clone();
        self.num_bins *= 2;
        self.data = vec![u16::MAX; self.num_bins * self.bin_size * self.list_size];

        let mut list_to_insert = vec![0; self.list_size];
        let mut list_pointer = 0;
        self.num_els_in_bin = vec![0; self.num_bins];
        while list_pointer < old_data.len() {
            if old_data[list_pointer] != u16::MAX {
                for i in 0..self.list_size {
                    list_to_insert[i] = old_data[list_pointer + i];
                }
                self.static_insert_list(&list_to_insert);
            }
            list_pointer += self.list_size;
        }
    }
}

impl StateDB<u16> for USSL {
    fn is_present(&mut self, list: &Vec<u16>) -> bool {
        for hash_num in 0..self.num_hash_funcs {
            let bin = self.hash_list(list, hash_num);

            if self.list_present_in_bin(list, bin) {
                return true;
            }
        }
        return false;
    }

    fn clear_all(&mut self) {
        unsafe {
            libc::memset(
                self.data.as_mut_ptr() as _,
                0xffff,
                self.data.len() * mem::size_of::<u16>(),
            );
        }
    }

    fn insert_list(&mut self, list: &Vec<u16>) {
        let success = self.static_insert_list(list);

        if !success {
            self.resize();
        }
    }
}
