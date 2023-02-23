pub fn circuit(source_list: Vec<String>, block_list: Vec<String>) -> bool {
    for source in source_list {
        if block_list.contains(&source) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit() {
        let source_list = vec![String::from("0x0000000000000000000000000000000000000000")];
        let block_list = vec![String::from("0x0000000000000000000000000000000000000001")];

        assert!(circuit(source_list.clone(), block_list.clone()));

        let mut combined_list = source_list.clone();
        combined_list.extend(block_list.clone());

        assert!(circuit(combined_list, block_list) == false);
    }
}
