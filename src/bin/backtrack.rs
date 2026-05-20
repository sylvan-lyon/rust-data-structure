struct Solution;

fn factor(x: i32) -> usize {
    let mut mul: usize = 0;
    for val in 1..x {
        mul = mul * val as usize
    }
    mul
}

impl Solution {
    pub fn combine(n: i32, k: i32) -> Vec<Vec<i32>> {
        let (mut path, mut answer) = (vec![], vec![]);
        // 一点小小的 trick
        path.reserve(k as usize);
        let reserve = match factor(k) * factor(n - k) {
            a if a != 0 => factor(n) / a,
            _ => 4,
        };
        answer.reserve(reserve);

        fn backtrack(
            path: &mut Vec<i32>,
            answer: &mut Vec<Vec<i32>>,
            min: i32,
            max: i32,
            length: i32,
        ) {
            if path.len() == length as usize {
                answer.push(path.clone());
                println!("满足长度要求, 保存");
                return;
            }

            for value in min..=max {
                path.push(value);

                {
                    println!("压入 {:4} path -> {:?}", value, path);
                }

                backtrack(path, answer, value + 1, max, length);
                path.pop();

                {
                    println!("弹出 {:4} path -> {:?}", value, path);
                    println!("---------------------")
                }
            }
        }

        backtrack(&mut path, &mut answer, 1, n, k);
        answer
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let (n, k) = (
        args.get(1)
            .map(|s| s.parse().expect("first arg should be a i32"))
            .unwrap_or(4),
        args.get(2)
            .map(|s| s.parse().expect("second arg should be a i32"))
            .unwrap_or(2),
    );

    let res = Solution::combine(n, k);
    println!("\n[Result]:");
    println!("[");
    for row in res.iter() {
        println!("    {:?},", row)
    }
    println!("]");
    println!("total: {}", res.len());
}
