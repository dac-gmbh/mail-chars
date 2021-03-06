//! Provides char classification for mail related grammar parts/charset, i.e.
//! if a given char belongs characters valid in atext, ctext, dtext, token etc.
//!
//! The `Charset` enum is used to determine which set of character is used.  To
//! check if a `char` is in that set either use `Charset::contains(&self,
//! char)` or use `ch.is(charset)` which is provided through the `CharMatchExt`
//! extension trait.
//!
//! # Why `Ws` is merged into `CText`, `DText` and `QText`
//!
//! Any grammar part in which `qtext`/`ctext`/`dtext` is used is in a form
//! which 1. repeats 2. prepends `FWS` in the repeating part.
//!
//! Which means any parser would have to parse for chars which are
//! `qtext`/`ctext`/`dtext` OR `ws` (and special handling if it hits another
//! character like `"\r"` indicating the start of a soft line break etc.).
//!
//! For example wrt. `dtext` the grammar is `... *([FWS] dtext) [FWS] ...`
//! which you can validate by parsing chars which are either `dtext` or `ws`
//! and if you hit a `"\r"` (which btw. is _not_ in `ws`) you make sure it's
//! followed by `"\n "` or `"\n\t"` and then you continue with parsing.
//!
//! # Alternative interface
//!
//! All enum variants are re-exported under a module with the name of the rfc where
//! they are specified. E.g. `Charset::CText` is also available as `rfc5322::CText`.
//!
//! # Example
//!
//! ```rust
//! extern crate mail_chars;
//! use mail_chars::{Charset, rfc5322, rfc2045, CharMatchExt};
//!
//! fn main() {
//!     assert!(Charset::AText.contains('d'));
//!     assert!('d'.is(Charset::AText));
//!     assert!('d'.is(rfc5322::AText));
//!
//!     // `rfc*::*` are just reexports grouped by RFC.
//!     assert_eq!(Charset::Token, rfc2045::Token);
//!
//!     // If we want to test for more than on char set we can use lookup.
//!     let res = Charset::lookup('.');
//!     // Has the benefit that there is a is_ascii method
//!     assert!(res.is_ascii());
//!     assert!(res.is(rfc2045::Token));
//!     assert!(res.is(rfc5322::CTextWs));
//!     assert!(!res.is(rfc5322::AText));
//! }
//! ```

mod lookup;

/// An enum for the charsets represented through an internal lookup table.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Charset {
    /// `qtext` + `ws`, basically anything which can appear in a quoted string
    /// which is not a quoted-pair.
    ///
    /// Note: this is equivalent to rfc7230 `qdtext`, excluding the obsolete
    ///       part of the grammar.
    /// Note: the obsolete part of the grammar is excluded
    ///
    /// **rfc: 5322**
    QTextWs = lookup::QC,

    /// `ctext` + `ws`
    ///
    /// Note: the obsolete part of the grammar is excluded.
    ///
    /// **rfc: 5322**
    CTextWs = lookup::CT,

    /// `dtext` + `ws`
    ///
    /// **rfc: 5322**
    DTextWs = lookup::DT,

    /// `atext`
    ///
    /// **rfc: 5322**
    AText = lookup::AT,

    /// Restricted-name-char subset of rfc2045 token with which IETF-tokens and
    /// IANA-tokens have to comply.
    ///
    /// **rfc: 6838** (related rfc2045)
    RestrictedToken = lookup::RT,

    /// `token`
    ///
    /// Note> there are multiple mail related definitions of token, this one is the rfc2045 based
    /// one.
    ///
    /// **rfc: 2045**
    ///
    Token = lookup::TO,

    /// obs-NO-WS-CTL
    ///
    /// **rfc: 5322**
    ///
    /// combine with CText or QText to support the obsolete part of the grammar
    ///
    /// # Example
    ///
    /// ```
    /// use mail_chars::{Charset, CharMatchExt, rfc5322};
    ///
    /// fn is_ctext_with_obs(ch: char) -> bool {
    ///     let res = Charset::lookup(ch);
    ///     res.is(rfc5322::CTextWs) || res.is(rfc5322::ObsNoWsCtl)
    /// }
    ///
    /// assert!("\x01 comment \x02".chars().all(is_ctext_with_obs));
    /// ```
    ObsNoWsCtl = lookup::NC,

    /// `token`
    ///
    /// **rfc: 7230**
    ///
    /// Token as defined in rfc7230 (HTTP/1.1) not directly a mail grammar, but
    /// relevant for shared utilities like e.g. anything Media Type (i.e.
    /// MIME-Type/Content-Type) related.
    Rfc7230Token = lookup::HT
}

impl Charset {
    /// Returns true if the `char` is part of this set of chars.
    #[inline]
    pub fn contains(&self, ch: char) -> bool {
        self.contains_lookup(ch, false)
    }

    /// Returns true if the `char` is part of the set of chars or not an ascii
    /// char.
    ///
    /// This is mainly meant to be used in combination with rfc6532 which
    /// extends all `*text` grammar parts/character sets to contain any
    /// non-us-ascii character.
    #[inline]
    pub fn contains_or_non_ascii(&self, ch: char) -> bool {
        self.contains_lookup(ch, true)
    }

    fn contains_lookup(&self, ch: char, out_of_table_value: bool) -> bool {
        let index = ch as u32;
        if index < 0x80 {
            lookup::US_ASCII_LOOKUP[index as usize] & (*self as u8) != 0
        } else {
            out_of_table_value
        }
    }

    /// Uses the internal lookup table to classify a `char`.
    pub fn lookup(ch: char) -> LookupResult {
        let index = ch as u32;
        if index < 0x80 {
            LookupResult(Some(lookup::US_ASCII_LOOKUP[index as usize]))
        } else {
            LookupResult(None)
        }
    }
}

mod sealed{ pub trait Seal {} }
pub use self::sealed::Seal;
pub trait CharMatchExt: Seal+Copy {
    /// Returns true if the char is a char belonging to the given charset.
    fn is(self, charset: Charset) -> bool;

    /// Returns true if the char is a char belonging to the given charset or a
    /// non-us-ascii char.
    fn is_inkl_non_ascii(self, charset: Charset) -> bool;
}

impl Seal for char {}
impl CharMatchExt for char {
    #[inline]
    fn is(self, charset: Charset) -> bool {
        charset.contains(self)
    }
    #[inline]
    fn is_inkl_non_ascii(self, charset: Charset) -> bool {
        charset.contains_or_non_ascii(self)
    }
}

/// Represents the result of a lookup of a char.
///
/// `CharMatchExt` is implemented on it so that you can treat it the same
/// as a char (wrt. this trait).
///
/// # Example
///
/// ```
/// use mail_chars::{Charset, CharMatchExt};
///
/// let res = Charset::lookup('↓');
/// assert!(!res.is_ascii());
/// assert!(!res.is(Charset::QTextWs));
/// assert!(res.is_inkl_non_ascii(Charset::QTextWs));
///
/// let res = Charset::lookup('<');
/// assert!(res.is_ascii());
/// assert!(res.is(Charset::QTextWs));
/// assert!(res.is(Charset::CTextWs));
/// assert!(!res.is(Charset::AText));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LookupResult(Option<u8>);

impl LookupResult {

    pub fn is_ascii(&self) -> bool {
        self.0.is_some()
    }

    fn lookup_contains(&self, charset: Charset, default: bool) -> bool {
        self.0.map(|res| {
            res & (charset as u8) != 0
        }).unwrap_or(default)
    }
}

impl Seal for LookupResult {}
impl CharMatchExt for LookupResult {
    #[inline]
    fn is(self, charset: Charset) -> bool {
        self.lookup_contains(charset, false)
    }
    #[inline]
    fn is_inkl_non_ascii(self, charset: Charset) -> bool {
        self.lookup_contains(charset, true)
    }
}

/// Re-export of all charsets (Charset::... variants) from rfc5322.
pub mod rfc5322 {
    pub use super::Charset::{QTextWs, CTextWs, AText, DTextWs, ObsNoWsCtl};
}

/// Re-export of all charsets (Charset::... variants) from rfc2045.
pub mod rfc2045 {
    pub use super::Charset::Token;
}

/// Re-export of all charsets (Charset::... variants) from rfc6838.
pub mod rfc6838 {
    pub use super::Charset::RestrictedToken;
}

/// Re-export of all charsets (Charset::... variants) from rfc7320.
///
/// Note that `QTextWs` (rfc5322) is exported as `Qdtext` (rfc7320) as they are
/// the equivalent (if the obsolete part of both grammars is excluded).
pub mod rfc7230 {
    pub use super::Charset::{
        QTextWs as QDText,
        Rfc7230Token as Token
    };

}


#[inline]
pub fn is_ws(ch: char) -> bool {
    ch == ' ' || ch  == '\t'
}

#[inline]
pub fn is_vchar(ch: char) -> bool {
    ' ' < ch && ch <= '~'
}

#[cfg(test)]
mod test {
    use super::{Charset, CharMatchExt, is_vchar};

    #[test]
    fn lookup_result_ascii() {
        let res = Charset::lookup('<');
        assert!(res.is_ascii());
        assert!(res.is(Charset::QTextWs));
        assert!(res.is_inkl_non_ascii(Charset::QTextWs));
        assert!(res.is(Charset::CTextWs));
        assert!(res.is_inkl_non_ascii(Charset::CTextWs));
        assert!(!res.is(Charset::AText));
        assert!(!res.is_inkl_non_ascii(Charset::AText));
    }

    #[test]
    fn lookup_result_utf8() {
        let res = Charset::lookup('↓');
        assert!(!res.is_ascii());
        assert!(!res.is(Charset::QTextWs));
        assert!(res.is_inkl_non_ascii(Charset::QTextWs));
    }

    #[test]
    fn is_part_of_charset() {
        // Just a "general" check if it works, any specific checks about which
        // chars belong to which set of chars is handled in the lookup modules
        // tests.
        assert!('<'.is(Charset::QTextWs));
        assert!('<'.is_inkl_non_ascii(Charset::QTextWs));
        assert!(!'<'.is(Charset::AText));
        assert!(!'<'.is_inkl_non_ascii(Charset::AText));

        let first_char_not_in_table = '\u{80}';
        assert!(!first_char_not_in_table.is(Charset::CTextWs));
        assert!(first_char_not_in_table.is_inkl_non_ascii(Charset::CTextWs));
    }

    #[test]
    fn is_vchar_boundaries() {
        let min = '!';
        let min_m1 = ' ';
        assert_eq!(min as u32 - 1, min_m1 as u32);
        let max = '~';
        let max_p1 = '\u{7f}';
        assert_eq!(max as u32 + 1, max_p1 as u32);

        assert!(is_vchar(min));
        assert!(!is_vchar(min_m1));
        assert!(is_vchar(max));
        assert!(!is_vchar(max_p1));
    }
}
