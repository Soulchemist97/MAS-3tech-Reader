use std::fmt;

/// Geldbetrag in Cent. Ganzzahl statt f64, damit Summen exakt bleiben.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money(pub i64);

impl Money {
    /// Parst Beträge wie `1133,00`, `0,5` oder `1215.4` (1–2 Nachkommastellen).
    ///
    /// Vorzeichen gibt es hier nicht: wie im Python-Original wird nur der
    /// Betrag gelesen, das Vorzeichen ergibt sich aus dem Stichwort (AUSWURF -).
    pub fn parse(s: &str) -> Option<Money> {
        let (whole, frac) = s.trim().split_once([',', '.'])?;
        let whole: i64 = whole.parse().ok()?;
        let frac_value: i64 = frac.parse().ok()?;
        let cents = match frac.len() {
            1 => frac_value * 10, // "1215,4" = 1215,40
            2 => frac_value,
            _ => return None,
        };
        Some(Money(whole * 100 + cents))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{:02}", self.0 / 100, self.0 % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_varianten() {
        assert_eq!(Money::parse("1133,00"), Some(Money(113300)));
        assert_eq!(Money::parse("0,00"), Some(Money(0)));
        assert_eq!(Money::parse("1215,4"), Some(Money(121540)));
        assert_eq!(Money::parse("12.5"), Some(Money(1250)));
        assert_eq!(Money::parse("abc"), None);
        assert_eq!(Money::parse("12"), None);
    }

    #[test]
    fn display() {
        assert_eq!(Money(113300).to_string(), "1133,00");
        assert_eq!(Money(5).to_string(), "0,05");
    }
}
