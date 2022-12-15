mod files;
mod local;
mod global;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() > 1 && args[1] == "-l" {
        local::args(&args[2..])
    } else {
        global::args(&args[1..])
    }
}
