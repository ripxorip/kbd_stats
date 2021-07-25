use notify_rust::Notification;

#[derive(Debug)]
pub struct Keydata {
    pub symbol: String,
}

pub struct Processor {}

impl Processor {

    pub fn new() -> Processor {
        Processor{}
    }

    pub fn process_key(&self, kd: Keydata) {
        println!("{:?}", kd);

    }

    pub fn process_second(&self) {
        println!("Second callback");
        /*
        Notification::new()
            .summary("Firefox News")
            .body("This will almost look like a real firefox notification.")
            .icon("firefox")
            .show().unwrap();
            */
    }
}
