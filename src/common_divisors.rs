use alloc::vec::Vec;
use core::mem::swap;
use core::num::NonZero;

pub fn find_common_divisors(a: NonZero<u32>, b: NonZero<u32>) -> Vec<u32> {
    let has_odd = (a.get() | b.get()) & 1;
    let increment = has_odd as usize + 1;

    let mut divisors = Vec::new();
    let gcd = gcd(a, b);

    for num in (1..=gcd / 2).step_by(increment) {
        if gcd % num == 0 {
            divisors.push(num);
        }
    }

    divisors.push(gcd);

    divisors
}

/// Greatest Common Divisor using binary GCD algorithm
pub fn gcd(u: NonZero<u32>, v: NonZero<u32>) -> u32 {
    let mut u = u.get();
    let mut v = v.get();

    let shift = (u | v).trailing_zeros();
    u >>= shift;
    v >>= shift;
    u >>= u.trailing_zeros();

    loop {
        v >>= v.trailing_zeros();

        if u > v {
            swap(&mut u, &mut v);
        }

        v -= u;

        if v == 0 {
            return u << shift;
        }
    }
}

#[test]
fn common_divisors() {
    let test_cases: [(u32, u32, Vec<u32>); 23] = [
        (1, 1, vec![1]),
        (100, 10, vec![1, 2, 5, 10]),
        (1000, 250, vec![1, 2, 5, 10, 25, 50, 125, 250]),
        (101, 103, vec![1]),
        (12, 18, vec![1, 2, 3, 6]),
        (123456, 789012, vec![1, 2, 3, 4, 6, 12]),
        (140, 90, vec![1, 2, 5, 10]),
        (1402, 1402, vec![1, 2, 701, 1402]),
        (144, 12, vec![1, 2, 3, 4, 6, 12]),
        (15, 25, vec![1, 5]),
        (21, 14, vec![1, 7]),
        (4234, 523, vec![1]),
        (48, 180, vec![1, 2, 3, 4, 6, 12]),
        (56, 98, vec![1, 2, 7, 14]),
        (60, 48, vec![1, 2, 3, 4, 6, 12]),
        (666, 999, vec![1, 3, 9, 37, 111, 333]),
        (6855, 9445, vec![1, 5]),
        (7, 13, vec![1]),
        (81, 27, vec![1, 3, 9, 27]),
        (84720, 74832, vec![1, 2, 3, 4, 6, 8, 12, 16, 24, 48]),
        (849273, 42837, vec![1, 3, 131, 393]),
        (9, 28, vec![1]),
        (94238, 162374, vec![1, 2]),
    ];

    for (a, b, expected) in test_cases.iter() {
        let a = NonZero::new(*a).unwrap();
        let b = NonZero::new(*b).unwrap();
        let result = find_common_divisors(a, b);
        assert_eq!(result, *expected, "a: {}, b: {}", a.get(), b.get());
    }
}
