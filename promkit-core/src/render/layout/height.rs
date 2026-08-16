//! Vertical allocation for already-laid-out, non-empty panes.
//!
//! Every pane first reserves one row. Ordered-content panes then consume rows in
//! pane order. Fair panes split what remains into equal shares. Finally,
//! fair-content panes shrink to their content without returning unused rows,
//! while rows released by a capped fair-fill pane may move to another fair-fill
//! pane.

use crate::HeightPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HeightRequest {
    policy: HeightPolicy,
    content_height: usize,
    max_height: Option<usize>,
}

impl HeightRequest {
    pub(super) fn new(
        policy: HeightPolicy,
        content_height: usize,
        max_height: Option<usize>,
    ) -> Self {
        Self {
            policy,
            content_height,
            max_height,
        }
    }

    fn is_fair(self) -> bool {
        matches!(
            self.policy,
            HeightPolicy::FairContent | HeightPolicy::FairFill
        )
    }

    fn content_limit(self) -> usize {
        self.max_height
            .unwrap_or(self.content_height)
            .min(self.content_height)
            .max(1)
    }

    fn fill_limit(self, available: usize) -> usize {
        self.max_height.unwrap_or(available).max(1)
    }
}

pub(super) fn allocate(requests: &[HeightRequest], available: usize) -> Vec<usize> {
    debug_assert!(requests.len() <= available);

    let mut heights = vec![1; requests.len()];
    let remaining = available.saturating_sub(requests.len());
    let remaining = allocate_ordered_content(requests, &mut heights, remaining);

    allocate_equal_fair_shares(requests, &mut heights, remaining);
    apply_fair_limits(requests, &mut heights, available);

    heights
}

fn allocate_ordered_content(
    requests: &[HeightRequest],
    heights: &mut [usize],
    mut remaining: usize,
) -> usize {
    for (index, request) in requests.iter().copied().enumerate() {
        if request.policy != HeightPolicy::OrderedContent {
            continue;
        }

        let extra = request.content_limit().saturating_sub(1).min(remaining);
        heights[index] += extra;
        remaining -= extra;
    }

    remaining
}

fn allocate_equal_fair_shares(requests: &[HeightRequest], heights: &mut [usize], remaining: usize) {
    let fair_count = requests.iter().filter(|request| request.is_fair()).count();
    if fair_count == 0 {
        return;
    }

    let rows_per_pane = remaining / fair_count;
    let extra_panes = remaining % fair_count;

    let mut fair_index = 0;
    for (index, request) in requests.iter().copied().enumerate() {
        if !request.is_fair() {
            continue;
        }

        heights[index] += rows_per_pane + usize::from(fair_index < extra_panes);
        fair_index += 1;
    }
}

fn apply_fair_limits(requests: &[HeightRequest], heights: &mut [usize], available: usize) {
    let mut redistributable_fill_rows = 0;

    for (index, request) in requests.iter().copied().enumerate() {
        match request.policy {
            HeightPolicy::OrderedContent => {}
            HeightPolicy::FairContent => {
                // Its equal share is an upper bound; unused rows stay unused.
                heights[index] = heights[index].min(request.content_limit());
            }
            HeightPolicy::FairFill => {
                let limit = request.fill_limit(available);
                // A max-height cap releases fill rows for other fill panes.
                redistributable_fill_rows += heights[index].saturating_sub(limit);
                heights[index] = heights[index].min(limit);
            }
        }
    }

    redistribute_fair_fill_rows(requests, heights, available, redistributable_fill_rows);
}

fn redistribute_fair_fill_rows(
    requests: &[HeightRequest],
    heights: &mut [usize],
    available: usize,
    mut remaining: usize,
) {
    while remaining > 0 {
        let mut distributed = false;

        for (index, request) in requests.iter().copied().enumerate() {
            if request.policy != HeightPolicy::FairFill
                || heights[index] >= request.fill_limit(available)
            {
                continue;
            }

            heights[index] += 1;
            remaining -= 1;
            distributed = true;
            if remaining == 0 {
                break;
            }
        }

        if !distributed {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(policy: HeightPolicy, content_height: usize) -> HeightRequest {
        HeightRequest::new(policy, content_height, None)
    }

    mod allocate {
        use super::super::allocate as allocate_heights;
        use super::*;

        #[test]
        fn preserves_content_allocation_in_key_order() {
            let requests = [
                request(HeightPolicy::OrderedContent, 10),
                request(HeightPolicy::OrderedContent, 10),
                request(HeightPolicy::OrderedContent, 10),
            ];

            assert_eq!(allocate_heights(&requests, 8), [6, 1, 1]);
        }

        #[test]
        fn shares_height_equally_between_fair_fill_entries() {
            let requests = [
                request(HeightPolicy::OrderedContent, 2),
                request(HeightPolicy::FairFill, 1),
                request(HeightPolicy::FairFill, 1),
            ];

            assert_eq!(allocate_heights(&requests, 10), [2, 4, 4]);
        }

        #[test]
        fn reallocates_height_after_a_fair_fill_entry_reaches_its_limit() {
            let requests = [
                request(HeightPolicy::OrderedContent, 2),
                request(HeightPolicy::FairFill, 1),
                HeightRequest::new(HeightPolicy::FairFill, 1, Some(3)),
            ];

            assert_eq!(allocate_heights(&requests, 12), [2, 7, 3]);
        }

        #[test]
        fn keeps_fair_content_within_its_equal_share() {
            let requests = [
                request(HeightPolicy::FairContent, 1),
                request(HeightPolicy::FairContent, 10),
                request(HeightPolicy::FairContent, 10),
            ];

            assert_eq!(allocate_heights(&requests, 8), [1, 3, 2]);
        }
    }
}
