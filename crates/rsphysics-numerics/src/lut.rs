//! Exact top-k lookup-table matching on row-major feature matrices.
use rsphysics_core::DomainError;
use std::{cmp::Ordering, collections::BinaryHeap};

#[derive(Debug)]
pub struct Matches {
    pub indices: Vec<usize>,
    pub scores: Vec<f64>,
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
    if channels == 0
        || !table.len().is_multiple_of(channels)
        || !observations.len().is_multiple_of(channels)
        || mask.len() != observations.len()
        || (!sigma.is_empty() && sigma.len() != observations.len())
        || k == 0
        || k > table.len() / channels
    {
        return Err(DomainError(
            "invalid LUT matching shapes or candidate count",
        ));
    }
    if table.iter().any(|x| !x.is_finite()) {
        return Err(DomainError("simulated LUT observations must be finite"));
    }
    let capacity = (observations.len() / channels)
        .checked_mul(k)
        .ok_or(DomainError("LUT match output is too large"))?;
    let mut result = Matches {
        indices: Vec::with_capacity(capacity),
        scores: Vec::with_capacity(capacity),
    };
    for (row, (observed, valid)) in observations
        .chunks_exact(channels)
        .zip(mask.chunks_exact(channels))
        .enumerate()
    {
        let active = active_channels(observed, valid, sigma, row * channels)?;
        let mut best = BinaryHeap::with_capacity(k);
        for (index, simulated) in table.chunks_exact(channels).enumerate() {
            let score = rmse(simulated, &active);
            if score.is_finite() {
                let candidate = Candidate { score, index };
                if best.len() < k {
                    best.push(candidate);
                } else if let Some(mut largest) = best.peek_mut()
                    && candidate < *largest
                {
                    *largest = candidate;
                }
            }
        }
        if best.len() < k {
            return Err(DomainError(
                "too few LUT candidates have finite residual scores",
            ));
        }
        for candidate in best.into_sorted_vec() {
            result.indices.push(candidate.index);
            result.scores.push(candidate.score);
        }
    }
    Ok(result)
}

fn active_channels(
    observed: &[f64],
    valid: &[bool],
    sigma: &[f64],
    offset: usize,
) -> Result<Vec<(usize, f64, f64)>, DomainError> {
    let mut active = Vec::with_capacity(observed.len());
    for (channel, (&value, &enabled)) in observed.iter().zip(valid).enumerate() {
        if enabled {
            let deviation = sigma.get(offset + channel).copied().unwrap_or(1.);
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
    Ok(active)
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
}
