use std::env;
use std::path::PathBuf;

pub struct Cli {
    pub url: String,
    pub user_agent: Option<String>,
    pub cookie: Option<String>,
    pub connect_timeout: Option<f64>,
    pub header: Vec<String>,
    pub include: bool,
    pub head: bool,
    pub insecure: bool,
    pub max_time: Option<f64>,
    pub output: Option<PathBuf>,
    pub remote_name: bool,
    pub request: Option<String>,
    pub data: Option<String>,
    pub fail: bool,
    pub location: bool,
    pub silent: bool,
    pub show_error: bool,
    pub verbose: bool,
    pub cacert: Option<PathBuf>,
    pub user: Option<String>,
    pub max_redirs: Option<usize>,
}

impl Cli {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut url = String::new();
        let mut user_agent = None;
        let mut cookie = None;
        let mut connect_timeout = None;
        let mut header = Vec::new();
        let mut include = false;
        let mut head = false;
        let mut insecure = false;
        let mut max_time = None;
        let mut output = None;
        let mut remote_name = false;
        let mut request = None;
        let mut data = None;
        let mut fail = false;
        let mut location = false;
        let mut silent = false;
        let mut show_error = false;
        let mut verbose = false;
        let mut cacert = None;
        let mut user = None;
        let mut max_redirs = None;

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "-A" | "--user-agent" => {
                    if i + 1 < args.len() {
                        user_agent = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-b" | "--cookie" => {
                    if i + 1 < args.len() {
                        cookie = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--connect-timeout" => {
                    if i + 1 < args.len() {
                        connect_timeout = args[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "-H" | "--header" => {
                    if i + 1 < args.len() {
                        header.push(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-i" | "--include" => include = true,
                "-I" | "--head" => head = true,
                "-k" | "--insecure" => insecure = true,
                "--max-time" => {
                    if i + 1 < args.len() {
                        max_time = args[i + 1].parse().ok();
                        i += 1;
                    }
                }
                "-o" | "--output" => {
                    if i + 1 < args.len() {
                        output = Some(PathBuf::from(&args[i + 1]));
                        i += 1;
                    }
                }
                "-O" | "--remote-name" => remote_name = true,
                "-X" | "--request" => {
                    if i + 1 < args.len() {
                        request = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-d" | "--data" => {
                    if i + 1 < args.len() {
                        data = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-f" | "--fail" => fail = true,
                "-L" | "--location" => location = true,
                "-s" | "--silent" => silent = true,
                "-S" | "--show-error" => show_error = true,
                "-v" | "--verbose" => verbose = true,
                "--cacert" => {
                    if i + 1 < args.len() {
                        cacert = Some(PathBuf::from(&args[i + 1]));
                        i += 1;
                    }
                }
                "-u" | "--user" => {
                    if i + 1 < args.len() {
                        user = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--max-redirs" => {
                    if i + 1 < args.len() {
                        max_redirs = args[i + 1].parse().ok();
                        i += 1;
                    }
                }
                _ => {
                    if !arg.starts_with('-') && url.is_empty() {
                        url = arg.clone();
                    }
                }
            }
            i += 1;
        }

        Self {
            url,
            user_agent,
            cookie,
            connect_timeout,
            header,
            include,
            head,
            insecure,
            max_time,
            output,
            remote_name,
            request,
            data,
            fail,
            location,
            silent,
            show_error,
            verbose,
            cacert,
            user,
            max_redirs,
        }
    }

    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if let Some(ref ua) = self.user_agent {
            headers.push(("User-Agent".to_string(), ua.clone()));
        }

        if let Some(ref c) = self.cookie {
            headers.push(("Cookie".to_string(), c.clone()));
        }

        for h in &self.header {
            if let Some(colon_pos) = h.find(':') {
                let name = h[..colon_pos].trim().to_string();
                let value = h[colon_pos + 1..].trim().to_string();
                headers.push((name, value));
            }
        }

        headers
    }

    pub fn get_method(&self) -> Option<String> {
        self.request.clone().or_else(|| {
            if self.head {
                Some("HEAD".to_string())
            } else {
                None
            }
        })
    }

    pub fn timeout(&self) -> Option<std::time::Duration> {
        self.max_time.map(std::time::Duration::from_secs_f64)
    }

    pub fn connect_timeout(&self) -> Option<std::time::Duration> {
        self.connect_timeout.map(std::time::Duration::from_secs_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_to_headers() {
        let mut cli = Cli {
            url: String::new(),
            user_agent: Some("test-agent".to_string()),
            cookie: None,
            connect_timeout: None,
            header: vec!["Content-Type: application/json".to_string()],
            include: false,
            head: false,
            insecure: false,
            max_time: None,
            output: None,
            remote_name: false,
            request: None,
            data: None,
            fail: false,
            location: false,
            silent: false,
            show_error: false,
            verbose: false,
            cacert: None,
            user: None,
            max_redirs: None,
        };

        let headers = cli.to_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| k == "User-Agent" && v == "test-agent"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Content-Type" && v == "application/json"));
    }

    #[test]
    fn test_cli_method() {
        let mut cli = Cli {
            url: "http://example.com".to_string(),
            user_agent: None,
            cookie: None,
            connect_timeout: None,
            header: vec![],
            include: false,
            head: false,
            insecure: false,
            max_time: None,
            output: None,
            remote_name: false,
            request: Some("POST".to_string()),
            data: None,
            fail: false,
            location: false,
            silent: false,
            show_error: false,
            verbose: false,
            cacert: None,
            user: None,
            max_redirs: None,
        };

        assert_eq!(cli.get_method(), Some("POST".to_string()));

        cli.request = None;
        cli.head = true;
        assert_eq!(cli.get_method(), Some("HEAD".to_string()));
    }
}
