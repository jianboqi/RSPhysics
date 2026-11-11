//! Exact k-d tree search using the same scoring and tie rule as a full scan.
use super::{BinaryHeap, Candidate, push_candidate, rmse};

const LEAF_ROWS: usize = 24;

pub(super) struct KdIndex {
    rows: Vec<usize>,
}

impl KdIndex {
    pub(super) fn new(table: &[f64], channels: usize) -> Self {
        let mut rows: Vec<_> = (0..table.len() / channels).collect();
        partition(&mut rows, table, channels, 0);
        Self { rows }
    }

    pub(super) fn search(
        &self,
        table: &[f64],
        active: &[(usize, f64, f64)],
        k: usize,
        best: &mut BinaryHeap<Candidate>,
    ) {
        let query = Query {
            table,
            active,
            channels: active.len(),
            axis_to_rmse: (1. / active.len() as f64).sqrt(),
            k,
        };
        query.visit(&self.rows, 0, best);
    }
}

fn partition(rows: &mut [usize], table: &[f64], channels: usize, axis: usize) {
    if rows.len() <= LEAF_ROWS {
        return;
    }
    let (left, _, right) = rows.select_nth_unstable_by(rows.len() / 2, |&a, &b| {
        table[a * channels + axis]
            .total_cmp(&table[b * channels + axis])
            .then(a.cmp(&b))
    });
    let next = (axis + 1) % channels;
    partition(left, table, channels, next);
    partition(right, table, channels, next);
}

struct Query<'a> {
    table: &'a [f64],
    active: &'a [(usize, f64, f64)],
    channels: usize,
    axis_to_rmse: f64,
    k: usize,
}

impl Query<'_> {
    fn visit(&self, rows: &[usize], axis: usize, best: &mut BinaryHeap<Candidate>) {
        if rows.len() <= LEAF_ROWS {
            for &index in rows {
                self.consider(index, best);
            }
            return;
        }
        let (left, split) = rows.split_at(rows.len() / 2);
        let (&pivot, right) = split.split_first().expect("nonempty partition");
        let (_, observed, sigma) = self.active[axis];
        let delta = self.table[pivot * self.channels + axis] - observed;
        let (near, far) = if delta >= 0. {
            (left, right)
        } else {
            (right, left)
        };
        let next = (axis + 1) % self.channels;
        self.consider(pivot, best);
        self.visit(near, next, best);

        // Every row in the far partition is at least this far away on the
        // split axis. The scaled RMSE has a maximum component at least as
        // large as that residual, and its normalized sum of squares is >= 1.
        let lower = (delta / sigma).abs() * self.axis_to_rmse;
        if best.len() < self.k
            || !lower.is_finite()
            || lower <= best.peek().expect("full candidate heap").score
        {
            self.visit(far, next, best);
        }
    }

    fn consider(&self, index: usize, best: &mut BinaryHeap<Candidate>) {
        let start = index * self.channels;
        let score = rmse(&self.table[start..start + self.channels], self.active);
        push_candidate(best, self.k, Candidate { score, index });
    }
}
