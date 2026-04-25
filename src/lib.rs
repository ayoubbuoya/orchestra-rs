use std::collections::HashMap;

use crate::core::provider::Provider;

mod core;

pub struct Orchestra {
    providers: HashMap<String, Box<dyn Provider>>,
}
