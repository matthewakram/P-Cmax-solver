use core::num;
use std::{fmt, mem};
extern crate libc;

pub struct MultiListSet {
    data: Vec<u16>,
    num_tables: usize,
    group_size: usize,
    list_size: usize,
    table_size: usize,
    current_clear: usize,
    misses_til_next_clear: usize
}

impl MultiListSet {
    pub fn new(num_tables: usize, group_size: usize, list_size: usize) -> MultiListSet {
        let table_size = group_size * (list_size - 1);
        let data: Vec<u16> = vec![u16::MAX; num_tables * table_size];

        return MultiListSet {
            data,
            num_tables,
            group_size,
            list_size,
            table_size,
            current_clear : 0,
            misses_til_next_clear: 100,
        };
    }

    fn calc_index(&self, table_num: usize, index_in_list: usize, element: u16) -> usize {
        if element  >= self.group_size as u16 {
            println!("oh no an error element {} group size {}", element, self.group_size);
            assert!(element < self.group_size as u16);
        }
        assert_ne!(index_in_list, self.list_size - 1);
        return table_num * self.table_size + self.group_size * index_in_list + element as usize;
    }

    fn insert_list_in_table(&mut self, table_num: usize, list: &Vec<u16>) -> bool {
        for i in 0..list.len() - 1 {
            let index = self.calc_index(table_num, i, list[i]);
            if self.data[index] != u16::MAX {
                return false;
            }
        }
        for i in 0..list.len() - 1 {
            let index = self.calc_index(table_num, i, list[i]);
            self.data[index] = list[i + 1];
        }
        return true;
    }

    pub fn insert_list(&mut self, list: &Vec<u16>) -> bool {
        for i in 0..self.num_tables {
            if self.insert_list_in_table(i, list) {
                return true;
            }
        }
        if self.misses_til_next_clear == 0 {
            self.clear_table(self.current_clear);
            self.current_clear +=1;
            self.current_clear %= self.num_tables;
        }
        self.misses_til_next_clear = 100;
        return false;
    }

    pub fn is_present_in_table(&self, table_num: usize, list: &Vec<u16>) -> bool {
        for i in 0..list.len() - 1 {
            let index = self.calc_index(table_num, i, list[i]);
            if self.data[index] != list[i + 1] {
                return false;
            }
        }
        return true;
    }
    pub fn is_present(&self, list: &Vec<u16>) -> bool {
        for i in 0..self.num_tables {
            if self.is_present_in_table(i, list) {
                return true;
            }
        }
        return false;
    }

    pub fn clear_table(&mut self, table_num: usize) {
        let start_index = self.calc_index(table_num, 0, 0);

        unsafe {
            libc::memset(
                (self.data.as_mut_ptr().offset(start_index as isize)) as _,
                0xffff,
                self.table_size * mem::size_of::<u16>(),
            );
        }
    }

    pub fn clear_all(&mut self) {

        unsafe {
            libc::memset(
                self.data.as_mut_ptr() as _,
                0xffff,
                self.num_tables * self.table_size * mem::size_of::<u16>(),
            );
        }
    }
}

impl fmt::Display for MultiListSet {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out: String = String::new();
        for table_num in (0..self.num_tables).rev() {
            for element in (0..self.group_size).rev() {
                for index_in_list in 0..self.list_size-1 {
                    let index = self.calc_index(table_num, index_in_list, element as u16);
                    out += &format!(", {}", self.data[index]);
                }
                out += "\n";
            }
            out += "\n"
        }
        return  f.write_str(&out);
    }
}
