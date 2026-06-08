use ifa_macros::Ebo;

#[derive(Ebo)]
#[ebo(cleanup = "close")]
struct MyFile {
    handle: i32,
}

impl MyFile {
    fn close(&mut self) {}
}

fn main() {
    let _f = MyFile { handle: 1 };
}
