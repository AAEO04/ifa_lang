use ifa_parser::parse;

fn main() {
    let src = r#"
    iwa Logger {
        #[iwa_pele_pair(open, close)]
        open();
        close();
    }
    "#;
    
    match parse(src) {
        Ok(program) => println!("{:#?}", program),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
