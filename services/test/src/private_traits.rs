use crate::generated::common::SharedCustomProviderMethods;

pub trait PrivateTestTrait : SharedCustomProviderMethods {
    fn hello_world(&self) {
        println!("Hello World!");
    }
}
