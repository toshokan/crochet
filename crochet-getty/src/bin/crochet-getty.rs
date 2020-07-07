use std::fs::File;
use clap::Clap;

#[derive(Debug)]
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Options {
    #[clap(short, long)]
    port: String,
    #[clap(short, long, default_value = "38400")]
    baud: String,
    #[clap(short, long, default_value = "linux")]
    term_type: String
}

fn open_tty(fragment: &str) -> Result<(), std::io::Error> {
    use std::os::unix::io::AsRawFd;
    use std::fs::OpenOptions;
    
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(format!("/dev/{}", fragment))?;
    
    let fd = file.as_raw_fd();
    crochet_getty_sys::fchown(fd, "root", "tty")
	.expect("Failed to set owner/group on tty");
    crochet_getty_sys::fchmod(fd, 0o620)
	.expect("Failed to permission bits on tty");
    crochet_getty_sys::set_stdin(fd)?;
    crochet_getty_sys::set_stdout(fd)?;
    crochet_getty_sys::set_stderr(fd)?;
    Ok(())
}

fn clear_tty() {
    print!("\x1b[2J\x1b[1;1H");
}

fn main() -> std::io::Result<()> {
    let opts = Options::parse();
    eprintln!("args = {:#?}", opts);
    open_tty(&opts.port)?;
    clear_tty();
    println!("{}\n", env!("CARGO_PKG_NAME"));
    crochet_getty_sys::yield_to_login();
    Ok(())
}
