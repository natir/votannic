//! Utils

/* std use */

/* crate use */

/* module declaration */

/* project use */

const WOUND_MAX: usize = 25;
static WOUND_THRESHOLD: [u64; WOUND_MAX * WOUND_MAX] = {
    let mut data = [0; WOUND_MAX * WOUND_MAX];

    let mut index = 0;
    while index != WOUND_MAX * WOUND_MAX {
        let (s, t) = (index / WOUND_MAX, index % WOUND_MAX);

        data[index] = const_wound_threshold(s as u64, t as u64);

        index += 1;
    }

    data
};

const fn const_wound_threshold(s: u64, t: u64) -> u64 {
    if s >= 2 * t {
        2
    } else if s > t {
        3
    } else if s == t {
        4
    } else if 2 * s >= t {
        5
    } else {
        6
    }
}

/// Get wound threshold
pub fn wound_threshold(s: u64, t: u64) -> u64 {
    WOUND_THRESHOLD[s as usize * WOUND_MAX + t as usize]
}

#[cfg(test)]
mod tests {
    /* std use */

    /* crate use */

    /* project use */
    use super::*;

    #[test]
    fn wound() {
        for s in 0usize..WOUND_MAX {
            for t in 0usize..WOUND_MAX {
                assert_eq!(
                    wound_threshold(s as u64, t as u64),
                    const_wound_threshold(s as u64, t as u64)
                );
            }
        }

        assert_eq!(wound_threshold(1, 5), 6);
        assert_eq!(wound_threshold(2, 5), 6);
        assert_eq!(wound_threshold(3, 5), 5);
        assert_eq!(wound_threshold(4, 5), 5);
        assert_eq!(wound_threshold(5, 5), 4);
        assert_eq!(wound_threshold(6, 5), 3);
        assert_eq!(wound_threshold(7, 5), 3);
        assert_eq!(wound_threshold(8, 5), 3);
        assert_eq!(wound_threshold(9, 5), 3);
        assert_eq!(wound_threshold(10, 5), 2);
    }
}
