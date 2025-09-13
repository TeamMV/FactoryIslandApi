pub trait AssertOnFalse {
    fn assert(&self, msg: &str);
}

impl AssertOnFalse for bool {
    fn assert(&self, msg: &str) {
        if !*self { 
            panic!("{msg}");
        }
    }
}