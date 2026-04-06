use super::matcher::{MatchCandidate, MatchQuery};

pub fn score_candidates(query: &MatchQuery, candidates: &[MatchCandidate]) -> Vec<MatchCandidate> {
    let mut scored = candidates.to_vec();
    if query.display_name.is_some() {
        scored.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    scored
}
