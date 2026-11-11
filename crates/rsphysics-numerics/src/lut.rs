//! Exact top-k lookup-table matching on row-major feature matrices.
use rsphysics_core::DomainError;
use std::{cmp::Ordering, collections::BinaryHeap, sync::OnceLock, thread};

mod index;
use index::KdIndex;

const INDEX_MIN_LUT_ROWS: usize = 4_096;
const INDEX_MIN_QUERIES: usize = 32;
const INDEX_MAX_CHANNELS: usize = 8;
const SAFE_INDEX_MAGNITUDE: f64 = f64::MAX / 2.;

#[derive(Debug)]
pub struct Matches {
    pub indices: Vec<usize>,
    pub scores: Vec<f64>,
}

/// A validated, row-major LUT. Workers borrow the same immutable matrix.
pub struct MatchTable<T> {
    values: T,
    channels: usize,
    index: OnceLock<Option<KdIndex>>,
}

struct Queries<'a> {
    observations: &'a [f64],
    sigma: &'a [f64],
    mask: &'a [bool],
    index: Option<&'a KdIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Candidate {
    score: f64,
    index: usize,
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .total_cmp(&other.score)
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Match each observation against every LUT row. The result is row-major
/// (observation, k), ordered by score and then by LUT row index.
/// An empty sigma slice selects ordinary RMSE. Otherwise scores are RMSE of
/// residuals divided by per-channel standard deviations. A false mask entry
/// excludes its channel from both the residual norm and the channel count.
pub fn match_rows(
    table: &[f64],
    observations: &[f64],
    channels: usize,
    sigma: &[f64],
    mask: &[bool],
    k: usize,
) -> Result<Matches, DomainError> {
    MatchTable::new(table, channels)?.search(observations, sigma, mask, k, 1)
}

impl<T: AsRef<[f64]> + Sync> MatchTable<T> {
    pub fn new(values: T, channels: usize) -> Result<Self, DomainError> {
        if channels == 0
            || values.as_ref().is_empty()
            || !values.as_ref().len().is_multiple_of(channels)
        {
            return Err(DomainError("invalid LUT table shape"));
        }
        if values.as_ref().iter().any(|x| !x.is_finite()) {
            return Err(DomainError("simulated LUT observations must be finite"));
        }
        Ok(Self {
            values,
            channels,
            index: OnceLock::new(),
        })
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Sigma and mask may be empty, one shared channel row, or one row per observation.
    /// Equal scores retain LUT row order regardless of the worker count.
    pub fn search(
        &self,
        observations: &[f64],
        sigma: &[f64],
        mask: &[bool],
        k: usize,
        workers: usize,
    ) -> Result<Matches, DomainError> {
        let channels = self.channels;
        let valid_shape = |len| len == 0 || len == channels || len == observations.len();
        if !observations.len().is_multiple_of(channels)
            || !valid_shape(sigma.len())
            || !valid_shape(mask.len())
            || k == 0
            || k > self.values.as_ref().len() / channels
            || workers == 0
        {
            return Err(DomainError(
                "invalid LUT matching shapes, candidate count or worker count",
            ));
        }
        let rows = observations.len() / channels;
        let capacity = rows
            .checked_mul(k)
            .ok_or(DomainError("LUT match output is too large"))?;
        let mut result = Matches {
            indices: vec![0; capacity],
            scores: vec![0.; capacity],
        };
        let queries = Queries {
            observations,
            sigma,
            mask,
            index: self.index_for(rows, k, mask),
        };
        let available = thread::available_parallelism().map_or(1, |count| count.get());
        let workers = workers.min(rows).min(available);
        if workers <= 1 {
            self.search_block(&queries, 0, k, &mut result.indices, &mut result.scores)?;
        } else {
            let rows_per_worker = rows.div_ceil(workers);
            let block_size = rows_per_worker * k;
            let pairs = result
                .indices
                .chunks_mut(block_size)
                .zip(result.scores.chunks_mut(block_size));
            thread::scope(|scope| {
                let mut handles = Vec::with_capacity(workers);
                let mut status = Ok(());
                let queries = &queries;
                for (chunk, (indices, scores)) in pairs.enumerate() {
                    let start = chunk * rows_per_worker;
                    match thread::Builder::new().spawn_scoped(scope, move || {
                        self.search_block(queries, start, k, indices, scores)
                    }) {
                        Ok(handle) => handles.push(handle),
                        Err(_) => {
                            status = Err(DomainError("could not start LUT search worker"));
                            break;
                        }
                    }
                }
                for handle in handles {
                    let result = handle
                        .join()
                        .unwrap_or(Err(DomainError("LUT search worker failed")));
                    if status.is_ok() {
                        status = result;
                    }
                }
                status
            })?;
        }
        Ok(result)
    }

    fn index_for(&self, observations: usize, k: usize, mask: &[bool]) -> Option<&KdIndex> {
        let table = self.values.as_ref();
        let rows = table.len() / self.channels;
        if rows < INDEX_MIN_LUT_ROWS || self.channels > INDEX_MAX_CHANNELS || k > rows / 16 {
            return None;
        }
        if let Some(index) = self.index.get() {
            return index.as_ref();
        }
        if observations < INDEX_MIN_QUERIES
            || (!mask.is_empty()
                && !mask
                    .chunks_exact(self.channels)
                    .any(|row| row.iter().all(|&v| v)))
        {
            return None;
        }
        self.index
            .get_or_init(|| {
                table
                    .iter()
                    .all(|x| x.abs() <= SAFE_INDEX_MAGNITUDE)
                    .then(|| KdIndex::new(table, self.channels))
            })
            .as_ref()
    }

    fn search_block(
        &self,
        queries: &Queries<'_>,
        start: usize,
        k: usize,
        indices: &mut [usize],
        scores: &mut [f64],
    ) -> Result<(), DomainError> {
        let channels = self.channels;
        let table = self.values.as_ref();
        let mut active = Vec::with_capacity(channels);
        let mut best = BinaryHeap::with_capacity(k);
        for (offset, (row_indices, row_scores)) in indices
            .chunks_exact_mut(k)
            .zip(scores.chunks_exact_mut(k))
            .enumerate()
        {
            let row = start + offset;
            let observed = channel_row(queries.observations, channels, row);
            let sigma = channel_row(queries.sigma, channels, row);
            let mask = channel_row(queries.mask, channels, row);
            active_channels(observed, mask, sigma, &mut active)?;
            if let Some(index) = queries.index
                && active.len() == channels
                && active
                    .iter()
                    .all(|&(_, value, _)| value.abs() <= SAFE_INDEX_MAGNITUDE)
            {
                index.search(table, &active, k, &mut best);
            } else {
                for (index, simulated) in table.chunks_exact(channels).enumerate() {
                    push_candidate(
                        &mut best,
                        k,
                        Candidate {
                            score: rmse(simulated, &active),
                            index,
                        },
                    );
                }
            }
            if best.len() < k {
                return Err(DomainError(
                    "too few LUT candidates have finite residual scores",
                ));
            }
            for (index, score) in row_indices.iter_mut().zip(row_scores).rev() {
                let candidate = best.pop().expect("validated candidate count");
                *index = candidate.index;
                *score = candidate.score;
            }
        }
        Ok(())
    }
}

fn channel_row<T>(values: &[T], channels: usize, row: usize) -> &[T] {
    if values.len() <= channels {
        values
    } else {
        &values[row * channels..(row + 1) * channels]
    }
}

fn push_candidate(best: &mut BinaryHeap<Candidate>, k: usize, candidate: Candidate) {
    if candidate.score.is_finite() {
        if best.len() < k {
            best.push(candidate);
        } else if let Some(mut largest) = best.peek_mut()
            && candidate < *largest
        {
            *largest = candidate;
        }
    }
}

fn active_channels(
    observed: &[f64],
    valid: &[bool],
    sigma: &[f64],
    active: &mut Vec<(usize, f64, f64)>,
) -> Result<(), DomainError> {
    active.clear();
    for (channel, &value) in observed.iter().enumerate() {
        if valid.get(channel).copied().unwrap_or(true) {
            let deviation = sigma.get(channel).copied().unwrap_or(1.);
            if !value.is_finite() || !deviation.is_finite() || deviation <= 0. {
                return Err(DomainError(
                    "active observations and standard deviations must be finite, with sigma > 0",
                ));
            }
            active.push((channel, value, deviation));
        }
    }
    if active.is_empty() {
        return Err(DomainError(
            "each observation needs at least one active channel",
        ));
    }
    Ok(())
}

fn rmse(simulated: &[f64], active: &[(usize, f64, f64)]) -> f64 {
    // Scale the sum of squares to preserve both small and large residuals.
    let mut scale = 0.;
    let mut squares = 0.;
    for &(channel, observed, deviation) in active {
        let value = simulated[channel];
        let difference = value - observed;
        let distance = if difference.is_finite() {
            (difference / deviation).abs()
        } else {
            (value / deviation - observed / deviation).abs()
        };
        if !distance.is_finite() {
            return f64::INFINITY;
        }
        if distance > scale {
            let ratio = scale / distance;
            squares = 1. + squares * ratio * ratio;
            scale = distance;
        } else if distance > 0. {
            let ratio = distance / scale;
            squares += ratio * ratio;
        }
    }
    scale * (squares / active.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_masks_and_sigma_preserve_exact_ranking() {
        let matched = match_rows(
            &[0., 10., 1., 0., 1., 9.],
            &[1., f64::NAN, 0., 0.],
            2,
            &[1., f64::NAN, 0.1, 10.],
            &[true, false, true, true],
            2,
        )
        .unwrap();
        assert_eq!(matched.indices, [1, 2, 0, 1]);
        assert_eq!(&matched.scores[..2], &[0., 0.]);
        assert!((matched.scores[2] - 1. / 2_f64.sqrt()).abs() < 1e-15);
    }

    #[test]
    fn extreme_finite_residuals_keep_their_order() {
        let matches = match_rows(&[2e-200, 1e-200, 1e200], &[0.], 1, &[], &[true], 3).unwrap();
        assert_eq!(matches.indices, [1, 0, 2]);
        assert_eq!(matches.scores, [1e-200, 2e-200, 1e200]);
    }

    #[test]
    fn indexed_search_equals_full_scan_with_ties_and_mixed_masks() {
        let channels = 6;
        let rows = 8192;
        let queries = 73;
        let mut state = 617_u64;
        let mut uniform = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };
        let mut table: Vec<_> = (0..rows * channels).map(|_| uniform()).collect();
        table.copy_within(0..channels, (rows - 1) * channels);
        let mut obs: Vec<_> = (0..queries * channels).map(|_| uniform()).collect();
        obs[..channels].copy_from_slice(&table[..channels]);
        let sigma: Vec<_> = (0..obs.len())
            .map(|_| 10_f64.powf(5. * uniform() - 3.))
            .collect();
        let mut mask = vec![true; obs.len()];
        for row in (1..queries).step_by(3) {
            mask[row * channels + 2] = false;
            obs[row * channels + 2] = f64::NAN;
        }

        let scan = MatchTable::new(table.as_slice(), channels).unwrap();
        assert!(scan.index.set(None).is_ok());
        let expected = scan.search(&obs, &sigma, &mask, 7, 1).unwrap();
        let indexed = MatchTable::new(table.as_slice(), channels).unwrap();
        let actual = indexed.search(&obs, &sigma, &mask, 7, 4).unwrap();
        assert!(indexed.index.get().is_some_and(Option::is_some));
        assert_eq!(actual.indices, expected.indices);
        assert_eq!(actual.scores, expected.scores);
        assert_eq!(&actual.indices[..2], &[0, rows - 1]);
    }
}
