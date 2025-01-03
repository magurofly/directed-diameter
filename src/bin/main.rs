use directed_diameter::*;
use proconio::{input, marker::Usize1};

fn main() {
    input! {
        n: usize, m: usize,
        edges: [(Usize1, Usize1); m],
    }

    let diameter = graph_diameter(n, &edges);
    println!("{diameter}");
}
