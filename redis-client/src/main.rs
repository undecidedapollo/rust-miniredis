use std::hash::{Hash, Hasher, SipHasher};

fn hashy(str: &str) -> u64 {
    let mut hasher = SipHasher::new_with_keys(0, 0);
    str.hash(&mut hasher);
    let res = hasher.finish();
    println!("Hash result: {str} -> {res}");

    return res;
}

fn main() {
    // let mut hasher = state.build_hasher();
    hashy("a");
    hashy("a");
    hashy("a");
    hashy("a");
    hashy("a");
    hashy("a");
}
