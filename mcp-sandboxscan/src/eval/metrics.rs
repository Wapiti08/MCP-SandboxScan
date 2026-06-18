use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Confusion {
    pub tp: usize,
    pub fp: usize,
    pub tn: usize,
    pub fn_count: usize,
    pub errors: usize,
}

impl Confusion {
    pub fn precision(&self) -> Option<f64> {
        let denom = self.tp + self.fp;
        if denom == 0 {
            return None;
        }
        Some(self.tp as f64 / denom as f64)
    }

    pub fn recall(&self) -> Option<f64> {
        let denom = self.tp + self.fn_count;
        if denom == 0 {
            return None;
        }
        Some(self.tp as f64 / denom as f64)
    }

    pub fn f1(&self) -> Option<f64> {
        let p = self.precision()?;
        let r = self.recall()?;
        if p + r == 0.0 {
            return None;
        }
        Some(2.0 * p * r / (p + r))
    }

    pub fn specificity(&self) -> Option<f64> {
        let denom = self.tn + self.fp;
        if denom == 0 {
            return None;
        }
        Some(self.tn as f64 / denom as f64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Label {
    Malicious,
    Clean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Detected,
    NotDetected,
    Error,
}

pub fn update_confusion(confusion: &mut Confusion, label: Label, verdict: Verdict) {
    match (label, verdict) {
        (_, Verdict::Error) => confusion.errors += 1,
        (Label::Malicious, Verdict::Detected) => confusion.tp += 1,
        (Label::Malicious, Verdict::NotDetected) => confusion.fn_count += 1,
        (Label::Clean, Verdict::Detected) => confusion.fp += 1,
        (Label::Clean, Verdict::NotDetected) => confusion.tn += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confusion_math() {
        let mut c = Confusion::default();
        update_confusion(&mut c, Label::Malicious, Verdict::Detected);
        update_confusion(&mut c, Label::Malicious, Verdict::NotDetected);
        update_confusion(&mut c, Label::Clean, Verdict::NotDetected);
        update_confusion(&mut c, Label::Clean, Verdict::Detected);

        assert_eq!(c.tp, 1);
        assert_eq!(c.fn_count, 1);
        assert_eq!(c.tn, 1);
        assert_eq!(c.fp, 1);
        assert_eq!(c.precision(), Some(0.5));
        assert_eq!(c.recall(), Some(0.5));
        assert_eq!(c.f1(), Some(0.5));
    }
}
