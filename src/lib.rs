use std::collections::VecDeque;

use ac_library::SccGraph;
use rand::prelude::*;

const NUM_DOUBLE_SWEEP: usize = 10;

/// 重みなし有向グラフの直径を求める。
pub fn graph_diameter(n: usize, edges: &[(usize, usize)]) -> usize {
    // 隣接リストを構築
    let mut graph_f = vec![vec![]; n];
    let mut graph_r = vec![vec![]; n];
    for &(from, to) in edges {
        graph_f[from].push(to);
        graph_r[to].push(from);
    }

    // 強連結成分分解・トポロジカルソート
    let (sccs, scc_id) = {
        let mut graph_scc = SccGraph::new(n);
        for &(from, to) in edges {
            graph_scc.add_edge(from, to);
        }
        let mut sccs = graph_scc.scc();
        // 強連結成分をトポロジカル逆順に並べる
        sccs.reverse();
        let mut scc_id = vec![0; n];
        for (id, scc) in sccs.iter_mut().enumerate() {
            // 強連結成分内は、入次数と出次数の積の降順に並べる
            scc.sort_by_key(|&v| std::cmp::Reverse(graph_f[v].len() * graph_r[v].len()) );
            for &mut v in scc {
                scc_id[v] = id;
            }
        }
        (sccs, scc_id)
    };

    // 直径の下界を Double-Sweep で求める
    let mut diameter = 0;
    for _ in 0 .. NUM_DOUBLE_SWEEP {
        diameter = diameter.max(dbg!(double_sweep(n, &graph_f, &graph_r)));
    }

    // ecc[v]: 離心数の上界
    let mut ecc = vec![n; n];
    for scc in &sccs {
        for &u in scc {
            eprintln!("D = {diameter}, scc = {scc:?}, u = {u}, ecc = {ecc:?}");

            // 離心数の上界が直径の下界以下ならもう用はない
            if ecc[u] <= diameter {
                continue;
            }

            // 離心数の上界を改善する
            let mut neighbors = graph_f[u].iter().map(|&v| (scc_id[v], ecc[v] + 1) ).collect::<Vec<_>>();
            neighbors.sort();
            neighbors.reverse();
            neighbors.dedup_by_key(|p| p.0 );
            let mut ub = 0;
            for (_, lb) in neighbors {
                ub = ub.max(lb);
                if ub > diameter {
                    break;
                }
            }

            // 離心数の上界が直径の下界以下ならもう用はない
            if ub <= diameter {
                ecc[u] = ub;
                continue;
            }

            // 離心数を正確に求める
            let dist_f = bfs_dist(n, &graph_f, u);
            ecc[u] = dist_f.iter().copied().max().unwrap();

            // 直径の下界を改善する
            diameter = diameter.max(ecc[u]);

            // 同じ強連結成分内で離心数の上界を改善する
            let mut queue = VecDeque::new();
            let mut dist_b = vec![n; n];
            queue.push_back(u);
            dist_b[u] = 0;
            while let Some(v) = queue.pop_front() {
                ecc[v] = ecc[v].min(dist_b[v] + ecc[u]);
                for &j in &graph_r[v] {
                    if scc_id[j] == scc_id[u] && dist_b[j] > dist_b[v] + 1 {
                        dist_b[j] = dist_b[v] + 1;
                        queue.push_back(j);
                    }
                }
            }
        }
    }

    eprintln!("ecc = {ecc:?}");

    diameter
}

/// ランダムに選んだ始点からの Double-Sweep アルゴリズムにより、グラフ直径の下限を求める。（2-近似）
/// 複数回呼び出し、最大値を取るとより精度が良くなる。
fn double_sweep(n: usize, graph_f: &[Vec<usize>], graph_r: &[Vec<usize>]) -> usize {
    let start1 = thread_rng().gen_range(0 .. n);
    let dist1 = bfs_dist(n, graph_f, start1);
    let start2 = (0 .. n).max_by_key(|&v| dist1[v] ).unwrap();
    let dist2 = bfs_dist(n, graph_r, start2);
    dist2.into_iter().max().unwrap()
}

fn bfs_dist(n: usize, graph: &[Vec<usize>], start: usize) -> Vec<usize> {
    let mut dist = vec![n; n];
    dist[start] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        let d = dist[u];
        for &v in &graph[u] {
            if dist[v] > d + 1 {
                dist[v] = d + 1;
                queue.push_back(v);
            }
        }
    }
    dist
}