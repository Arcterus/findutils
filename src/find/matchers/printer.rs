use super::PathInfo;
use super::SideEffectRefs;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// This matcher just prints the name of the file to stdout.
pub struct Printer {
    output: Rc<RefCell<Write>>,
}

impl Printer {
    pub fn new(output: Rc<RefCell<Write>>) -> Printer {
        Printer { output: output.clone() }
    }

    pub fn new_box(output: Rc<RefCell<Write>>) -> Box<super::Matcher> {
        Box::new(Printer { output: output.clone() })
    }
}

impl super::Matcher for Printer {
    fn matches(&self, file_info: &PathInfo, _: &mut SideEffectRefs) -> bool {
        writeln!(self.output.borrow_mut(),
                 "{}",
                 file_info.path().to_string_lossy())
            .unwrap();
        true
    }

    fn has_side_effects(&self) -> bool {
        true
    }
}

#[cfg(test)]

mod tests {
    use super::super::tests::*;
    use super::Printer;
    use super::super::Matcher;
    use super::super::SideEffectRefs;
    use std::cell::RefCell;
    use std::io::Cursor;
    use std::rc::Rc;
    use std::io::Read;

    #[test]
    fn prints() {
        let abbbc = get_dir_entry_for("./test_data/simple", "abbbc");

        let output = Rc::new(RefCell::new(Cursor::new(vec![])));
        let matcher = Printer::new(output.clone());
        assert!(matcher.matches(&abbbc, &mut SideEffectRefs::new()));
        let mut cursor = output.borrow_mut();
        cursor.set_position(0);
        let mut contents = String::new();
        cursor.read_to_string(&mut contents).unwrap();
        assert_eq!("./test_data/simple/abbbc\n", contents);
    }
}
