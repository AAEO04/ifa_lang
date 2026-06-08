use ifa_macros::iwa_pele;

struct MockOtura;
impl MockOtura {
    fn so(&self, _: &str, _: i32) -> MockConn { MockConn }
}
struct MockConn;
impl MockConn {
    fn pa(&self) {}
}

const Otura: MockOtura = MockOtura;

#[iwa_pele]
fn network_task() {
    let conn = Otura.so("example.com", 80);
    // conn.pa() missing
}

fn main() {}
