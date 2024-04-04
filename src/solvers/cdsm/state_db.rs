
pub trait StateDB<T> {
    fn insert_list(&mut self, list: &Vec<T>);
    fn is_present(&mut self, list: &Vec<T>) -> bool;
    fn clear_all(&mut self);
}