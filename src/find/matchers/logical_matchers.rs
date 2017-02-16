//! This modules contains the matchers used for combining other matchers and
//! performing boolean logic on them (and a couple of trivial always-true and
//! always-false matchers). The design is strongly tied to the precedence rules
//! when parsing command-line options (e.g. "-foo -o -bar -baz" is equivalent
//! to "-foo -o ( -bar -baz )", not "( -foo -o -bar ) -baz").

use super::PathInfo;
use std::error::Error;

/// This matcher contains a collection of other matchers. A file only matches
/// if it matches ALL the contained sub-matchers. For sub-matchers that have
/// side effects, the side effects occur in the same order as the sub-matchers
/// were pushed into the collection.
pub struct AndMatcher {
    submatchers: Vec<Box<super::Matcher>>,
}

impl AndMatcher {
    pub fn new() -> AndMatcher {
        AndMatcher { submatchers: Vec::new() }
    }

    pub fn new_and_condition(&mut self, matcher: Box<super::Matcher>) {
        self.submatchers.push(matcher);
    }
}


impl super::Matcher for AndMatcher {
    /// Returns true if all sub-matchers return true. Short-circuiting does take
    /// place. If the nth sub-matcher returns false, then we immediately return
    /// and don't make any further calls.
    fn matches(&self, dir_entry: &PathInfo) -> bool {
        self.submatchers.iter().all(|ref x| x.matches(dir_entry))
    }

    fn has_side_effects(&self) -> bool {
        self.submatchers.iter().any(|ref x| x.has_side_effects())
    }
}

/// This matcher contains a collection of other matchers. A file matches
/// if it matches any of the contained sub-matchers. For sub-matchers that have
/// side effects, the side effects occur in the same order as the sub-matchers
/// were pushed into the collection.
pub struct OrMatcher {
    submatchers: Vec<AndMatcher>,
}

impl OrMatcher {
    pub fn new_and_condition(&mut self, matcher: Box<super::Matcher>) {
        // safe to unwrap. submatchers always has at least one member
        self.submatchers.last_mut().unwrap().new_and_condition(matcher);
    }

    pub fn new_or_condition(&mut self, arg: &str) -> Result<(), Box<Error>> {
        if self.submatchers.last().unwrap().submatchers.is_empty() {
            return Err(From::from(format!("invalid expression; you have used a binary operator \
                                           '{}' with nothing before it.",
                                          arg)));
        }
        self.submatchers.push(AndMatcher::new());
        Ok(())
    }

    pub fn new() -> OrMatcher {
        let mut o = OrMatcher { submatchers: Vec::new() };
        o.submatchers.push(AndMatcher::new());
        o
    }
}


impl super::Matcher for OrMatcher {
    /// Returns true if any sub-matcher returns true. Short-circuiting does take
    /// place. If the nth sub-matcher returns true, then we immediately return
    /// and don't make any further calls.
    fn matches(&self, dir_entry: &PathInfo) -> bool {
        self.submatchers.iter().any(|ref x| x.matches(dir_entry))
    }

    fn has_side_effects(&self) -> bool {
        self.submatchers.iter().any(|ref x| x.has_side_effects())
    }
}

/// This matcher contains a collection of other matchers. In contrast to
/// OrMatcher and AndMatcher, all the submatcher objects are called regardless
/// of the results of previous submatchers. This is primarily used for
/// submatchers with side-effects. For such sub-matchers the side effects occur
/// in the same order as the sub-matchers were pushed into the collection.
pub struct ListMatcher {
    submatchers: Vec<OrMatcher>,
}

impl ListMatcher {
    pub fn new_and_condition(&mut self, matcher: Box<super::Matcher>) {
        // safe to unwrap. submatchers always has at least one member
        self.submatchers.last_mut().unwrap().new_and_condition(matcher);
    }

    pub fn new_or_condition(&mut self, arg: &str) -> Result<(), Box<Error>> {
        self.submatchers.last_mut().unwrap().new_or_condition(arg)
    }

    pub fn new_list_condition(&mut self) -> Result<(), Box<Error>> {
        {
            let child_or_matcher = &self.submatchers.last().unwrap();
            let grandchild_and_matcher = &child_or_matcher.submatchers.last().unwrap();

            if grandchild_and_matcher.submatchers.is_empty() {
                return Err(From::from("invalid expression; you have used a binary operator ',' \
                                       with nothing before it."));
            }
        }
        self.submatchers.push(OrMatcher::new());
        Ok(())
    }

    pub fn new() -> ListMatcher {
        let mut o = ListMatcher { submatchers: Vec::new() };
        o.submatchers.push(OrMatcher::new());
        o
    }
}


impl super::Matcher for ListMatcher {
    /// Calls matches on all submatcher objects, with no short-circuiting.
    /// Returns the result of the call to the final submatcher
    fn matches(&self, dir_entry: &PathInfo) -> bool {
        let mut rc = false;
        for ref matcher in &self.submatchers {
            rc = matcher.matches(dir_entry);
        }
        rc
    }

    fn has_side_effects(&self) -> bool {
        self.submatchers.iter().any(|ref x| x.has_side_effects())
    }
}

/// A simple matcher that always matches.
pub struct TrueMatcher {
}

impl super::Matcher for TrueMatcher {
    fn matches(&self, _dir_entry: &PathInfo) -> bool {
        true
    }

    fn has_side_effects(&self) -> bool {
        false
    }
}

/// A simple matcher that never matches.
pub struct FalseMatcher {
}

impl super::Matcher for FalseMatcher {
    fn matches(&self, _dir_entry: &PathInfo) -> bool {
        false
    }

    fn has_side_effects(&self) -> bool {
        false
    }
}

/// Matcher that wraps another matcher and inverts matching criteria.
pub struct NotMatcher {
    submatcher: Box<super::Matcher>,
}

impl NotMatcher {
    pub fn new(submatcher: Box<super::Matcher>) -> NotMatcher {
        NotMatcher { submatcher: submatcher }
    }
}

impl super::Matcher for NotMatcher {
    fn matches(&self, dir_entry: &PathInfo) -> bool {
        !self.submatcher.matches(dir_entry)
    }

    fn has_side_effects(&self) -> bool {
        self.submatcher.has_side_effects()
    }
}

#[cfg(test)]

mod tests {
    use super::super::tests::*;
    use super::*;
    use super::super::Matcher;
    use super::super::PathInfo;

    /// Simple Matcher impl that has side effects
    pub struct HasSideEffects {}

    impl Matcher for HasSideEffects {
        fn matches(&self, _: &PathInfo) -> bool {
            false
        }

        fn has_side_effects(&self) -> bool {
            true
        }
    }



    #[test]
    fn and_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let mut matcher = AndMatcher::new();
        let everything = Box::new(TrueMatcher {});
        let nothing = Box::new(FalseMatcher {});

        // start with one matcher returning true
        matcher.new_and_condition(everything);
        assert!(matcher.matches(&abbbc));
        matcher.new_and_condition(nothing);
        assert!(!matcher.matches(&abbbc));
    }

    #[test]
    fn or_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let mut matcher = OrMatcher::new();
        let matches_everything = Box::new(TrueMatcher {});
        let matches_nothing = Box::new(FalseMatcher {});

        // start with one matcher returning false
        matcher.new_and_condition(matches_nothing);
        assert!(!matcher.matches(&abbbc));
        matcher.new_or_condition("-o").unwrap();
        matcher.new_and_condition(matches_everything);
        assert!(matcher.matches(&abbbc));
    }

    #[test]
    fn list_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let mut matcher = ListMatcher::new();
        let matches_everything = Box::new(TrueMatcher {});
        let matches_nothing = Box::new(FalseMatcher {});
        let matches_nothing2 = Box::new(FalseMatcher {});

        // result should always match that of the last pushed submatcher
        matcher.new_and_condition(matches_nothing);
        assert!(!matcher.matches(&abbbc));
        matcher.new_list_condition().unwrap();
        matcher.new_and_condition(matches_everything);
        assert!(matcher.matches(&abbbc));
        matcher.new_list_condition().unwrap();
        matcher.new_and_condition(matches_nothing2);
        assert!(!matcher.matches(&abbbc));
    }

    #[test]
    fn true_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let matcher = TrueMatcher {};

        assert!(matcher.matches(&abbbc));
    }

    #[test]
    fn false_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let matcher = FalseMatcher {};

        assert!(!matcher.matches(&abbbc));
    }

    #[test]
    fn and_has_side_effects_works() {
        let mut matcher = AndMatcher::new();
        let no_side_effects = Box::new(TrueMatcher {});
        let side_effects = Box::new(HasSideEffects {});

        // start with one matcher returning false
        matcher.new_and_condition(no_side_effects);
        assert!(!matcher.has_side_effects());
        matcher.new_and_condition(side_effects);
        assert!(matcher.has_side_effects());
    }

    #[test]
    fn or_has_side_effects_works() {
        let mut matcher = OrMatcher::new();
        let no_side_effects = Box::new(TrueMatcher {});
        let side_effects = Box::new(HasSideEffects {});

        // start with one matcher returning false
        matcher.new_and_condition(no_side_effects);
        assert!(!matcher.has_side_effects());
        matcher.new_or_condition("-o").unwrap();
        matcher.new_and_condition(side_effects);
        assert!(matcher.has_side_effects());
    }

    #[test]
    fn list_has_side_effects_works() {
        let mut matcher = ListMatcher::new();
        let no_side_effects = Box::new(TrueMatcher {});
        let side_effects = Box::new(HasSideEffects {});

        // start with one matcher returning false
        matcher.new_and_condition(no_side_effects);
        assert!(!matcher.has_side_effects());
        matcher.new_list_condition().unwrap();
        matcher.new_and_condition(side_effects);
        assert!(matcher.has_side_effects());
    }

    #[test]
    fn true_has_side_effects_works() {
        let matcher = TrueMatcher {};
        assert!(!matcher.has_side_effects());
    }

    #[test]
    fn false_has_side_effects_works() {
        let matcher = FalseMatcher {};
        assert!(!matcher.has_side_effects());
    }

    #[test]
    fn not_matches_works() {
        let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
        let not_true = NotMatcher::new(Box::new(TrueMatcher {}));
        let not_false = NotMatcher::new(Box::new(FalseMatcher {}));
        assert!(!not_true.matches(&abbbc));
        assert!(not_false.matches(&abbbc));
    }

    #[test]
    fn not_has_side_effects_works() {
        let has_fx = NotMatcher::new(Box::new(HasSideEffects {}));
        let hasnt_fx = NotMatcher::new(Box::new(FalseMatcher {}));
        assert!(has_fx.has_side_effects());
        assert!(!hasnt_fx.has_side_effects());
    }

}
