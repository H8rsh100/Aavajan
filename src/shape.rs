const MASK_WIDTH: usize = 60;
const MASK_HEIGHT: usize = 84;
const MAX_TARGETS: usize = 900;

const MASK: &str = include_str!("shape_mask.txt");

#[derive(Clone, Copy)]
pub struct Target {
    pub x: f32,
    pub y: f32,
}

pub fn targets() -> Vec<Target> {
    let mut cells = Vec::new();
    for (row, line) in MASK.lines().enumerate() {
        for (col, byte) in line.bytes().enumerate() {
            if byte == b'#' {
                cells.push(Target {
                    x: (col as f32 + 0.5) / MASK_WIDTH as f32,
                    y: (row as f32 + 0.5) / MASK_HEIGHT as f32,
                });
            }
        }
    }

    let step = cells.len().div_ceil(MAX_TARGETS).max(1);
    cells
        .into_iter()
        .step_by(step)
        .map(|mut target| {
            target.x = 0.14 + target.x * 0.72;
            target.y = 0.06 + target.y * 0.88;
            target
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_points_are_bounded_and_limited() {
        let points = targets();
        assert!(!points.is_empty());
        assert!(points.len() <= MAX_TARGETS);
        assert!(
            points
                .iter()
                .all(|point| { (0.0..=1.0).contains(&point.x) && (0.0..=1.0).contains(&point.y) })
        );
    }

    #[test]
    fn target_points_span_the_artwork() {
        let points = targets();
        assert!(points.iter().any(|point| point.y < 0.4));
        assert!(points.iter().any(|point| point.y > 0.6));
        assert!(points.iter().any(|point| point.x < 0.4));
        assert!(points.iter().any(|point| point.x > 0.6));
    }
}
