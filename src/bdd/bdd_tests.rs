#[cfg(test)]
mod tests {
    use crate::{
        encoding::encoder::VarNameGenerator, bdd::bdd,
    };


    

    #[test]
    pub fn test_bdd_creation(){
        let _a = bdd::leq(vec![1, 2, 3, 4, 5], vec![49, 37, 21, 19, 7], 70, &mut VarNameGenerator::new());
    }
}
