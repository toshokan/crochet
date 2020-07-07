use chrono::Local;
use clap::Clap;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::process;

#[derive(Debug, Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Options {
    #[clap(short, long)]
    port: String,
    #[clap(short, long, default_value = "38400")]
    baud: String,
    #[clap(short, long, default_value = "linux")]
    term_type: String,
}

fn open_tty(fragment: &str) -> Result<(File, RawFd), std::io::Error> {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(format!("/dev/{}", fragment))?;

    let fd = file.as_raw_fd();
    crochet_getty_sys::fchown(fd, "root", "tty").expect("Failed to set owner/group on tty");
    crochet_getty_sys::fchmod(fd, 0o620).expect("Failed to permission bits on tty");
    Ok((file, fd))
}

fn clear_tty(file: &mut File) -> std::io::Result<()> {
    write!(file, "\x1b[2J\x1b[1;1H")?;
    Ok(())
}

fn open_streams(fd: RawFd) -> (process::Stdio, process::Stdio, process::Stdio) {
    let open_stream = || unsafe { process::Stdio::from_raw_fd(fd) };
    (open_stream(), open_stream(), open_stream())
}

struct Issue(String);

impl Issue {
    fn render(self) -> String {
        let mut rendered = String::with_capacity(self.0.len());
        let mut chars = self.0.chars().into_iter();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(opcode) = chars.next() {
		    use std::fmt::Write as _;
                    match opcode {
                        't' => {

                            let date = Local::now();
                            write!(rendered, "{}", date.format("%H:%M:%S"))
                                .expect("Failed to write time");
                        },
                        _ => {
			    write!(rendered, "\\{}", opcode)
    				.expect("Failed to write opcode");
			},
                    }
                }
            } else {
                rendered.push(c)
            }
        }
        rendered
    }
}

fn get_issue() -> std::io::Result<Issue> {
    let contents = std::fs::read_to_string("/etc/issue")?;
    Ok(Issue(contents))
}

fn main() -> std::io::Result<()> {
    let opts = Options::parse();
    eprintln!("args = {:#?}", opts);

    loop {
        let (mut file, fd) = open_tty(&opts.port)?;
        let (stdin, stdout, stderr) = open_streams(fd);

        clear_tty(&mut file)?;
        let issue = get_issue()?;
	writeln!(file)?;
        write!(file, "{}", issue.render())?;
        process::Command::new("/bin/login")
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("Failed.")
            .wait()?;
    }
}
