use std::ops::{Bound, RangeBounds};

#[derive(Debug, thiserror::Error)]
pub enum SliceError {
    #[error("Invalid range {start}..{end} for slice of length {len}")]
    OutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
}

pub fn get_slice<T, R>(slice: &[T], range: R) -> Result<&[T], SliceError>
where
    R: RangeBounds<usize>,
{
    let start = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_add(1),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(&n) => n.saturating_add(1),
        Bound::Excluded(&n) => n,
        Bound::Unbounded => slice.len(),
    };

    // Validate bounds
    if start > slice.len() || end > slice.len() || start > end {
        return Err(SliceError::OutOfBounds {
            start,
            end,
            len: slice.len(),
        });
    }

    Ok(&slice[start..end])
}
