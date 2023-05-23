use nix::unistd::gethostname;

pub fn get_hostname() -> Option<String> {
    gethostname().ok().and_then(|h| h.into_string().ok())
}