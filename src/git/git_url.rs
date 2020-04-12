#[derive(Clone)]
pub struct GitUrl {
    host: String,
    path: String,
}

impl GitUrl {
    const HTTP_PREFIX: &'static str = "http://";
    const HTTPS_PREFIX: &'static str = "https://";

    #[allow(dead_code)]
    pub fn parse(value: &str) -> Option<Self> {
        if value.starts_with(Self::HTTP_PREFIX) {
            value[Self::HTTP_PREFIX.len()..].find("/").map(|p| Self {
                host: value[..Self::HTTP_PREFIX.len() + p].to_string(),
                path: value[Self::HTTP_PREFIX.len() + p + 1..].to_string(),
            })
        } else if value.starts_with(Self::HTTPS_PREFIX) {
            value[Self::HTTPS_PREFIX.len()..].find("/").map(|p| Self {
                host: value[..Self::HTTPS_PREFIX.len() + p].to_string(),
                path: value[Self::HTTPS_PREFIX.len() + p + 1..].to_string(),
            })
        } else {
            value.find(":").map(|p| Self {
                host: value[..p].to_string(),
                path: value[p + 1..].to_string(),
            })
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self.path.len() {
            0 => self.host.to_string(),
            _ => self.host.to_string() + ":" + &self.path,
        }
    }

    #[allow(dead_code)]
    pub fn pop(&self) -> Option<Self> {
        let mut temp = self.clone();
        match temp.pop_mut() {
            true => Some(temp),
            false => None,
        }
    }

    #[allow(dead_code)]
    pub fn pop_mut(&mut self) -> bool {
        Self::pop_helper(&mut self.path)
    }

    #[allow(dead_code)]
    pub fn join(&self, child_path: &str) -> Option<Self> {
        let mut temp = self.clone();
        match temp.join_mut(child_path) {
            true => Some(temp),
            false => None,
        }
    }

    #[allow(dead_code)]
    pub fn join_mut(&mut self, child_path: &str) -> bool {
        let mut path = self.path.clone();
        for part in child_path.split("/") {
            if part.len() == 0 {
                return false;
            } else if part == ".." {
                if !Self::pop_helper(&mut path) {
                    return false;
                }
            } else if part != "." {
                if path.len() > 0 {
                    path += "/"
                }
                path += part
            }
        }
        self.path = path;
        true
    }

    fn pop_helper(path: &mut String) -> bool {
        if path.len() == 0 {
            false
        } else {
            match path.rfind('/') {
                Some(pos) => path.truncate(pos),
                None => path.clear(),
            }
            true
        }
    }
}

impl std::fmt::Display for GitUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_https() {
        let x0 = GitUrl::parse("https://github.com/user/foo/bar/quux.git").expect("parse failed");
        assert_eq!(x0.host, "https://github.com");
        assert_eq!(x0.path, "user/foo/bar/quux.git");

        let x1 = GitUrl::parse("http://github.com/user/foo/bar/quux.git").expect("parse failed");
        assert_eq!(x1.host, "http://github.com");
        assert_eq!(x1.path, "user/foo/bar/quux.git");

        let x2 = GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
        assert_eq!(x2.host, "git@github.com");
        assert_eq!(x2.path, "user/foo/bar/quux.git");
    }

    #[test]
    fn test_pop() {
        let x0 = GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");

        assert_eq!(x0.host, "git@github.com");
        assert_eq!(x0.path, "user/foo/bar/quux.git");
        assert_eq!(x0.to_string(), "git@github.com:user/foo/bar/quux.git");

        let x1 = x0.pop().expect("pop failed");
        assert_eq!(x1.host, "git@github.com");
        assert_eq!(x1.path, "user/foo/bar");
        assert_eq!(x1.to_string(), "git@github.com:user/foo/bar");

        let x2 = x1.pop().expect("pop failed");
        assert_eq!(x2.host, "git@github.com");
        assert_eq!(x2.path, "user/foo");
        assert_eq!(x2.to_string(), "git@github.com:user/foo");

        let x3 = x2.pop().expect("pop failed");
        assert_eq!(x3.host, "git@github.com");
        assert_eq!(x3.path, "user");
        assert_eq!(x3.to_string(), "git@github.com:user");

        let x4 = x3.pop().expect("pop failed");
        assert_eq!(x4.host, "git@github.com");
        assert_eq!(x4.path, "");
        assert_eq!(x4.to_string(), "git@github.com");

        assert!(x4.pop().is_none())
    }

    #[test]
    fn test_pop_mut() {
        let mut git_url =
            GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");

        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "user/foo/bar/quux.git");
        assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar/quux.git");

        assert!(git_url.pop_mut());
        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "user/foo/bar");
        assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar");

        assert!(git_url.pop_mut());
        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "user/foo");
        assert_eq!(git_url.to_string(), "git@github.com:user/foo");

        assert!(git_url.pop_mut());
        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "user");
        assert_eq!(git_url.to_string(), "git@github.com:user");

        assert!(git_url.pop_mut());
        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "");
        assert_eq!(git_url.to_string(), "git@github.com");

        assert!(!git_url.pop_mut());
        assert_eq!(git_url.host, "git@github.com");
        assert_eq!(git_url.path, "");
        assert_eq!(git_url.to_string(), "git@github.com");
    }

    #[test]
    fn test_join() {
        let git_url = GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");

        assert_eq!(
            git_url.join("aaa").expect("join failed").to_string(),
            "git@github.com:user/foo/bar/quux.git/aaa"
        );

        assert_eq!(
            git_url.join("aaa/bbb").expect("join failed").to_string(),
            "git@github.com:user/foo/bar/quux.git/aaa/bbb"
        );

        assert_eq!(
            git_url.join(".").expect("join failed").to_string(),
            "git@github.com:user/foo/bar/quux.git"
        );

        assert_eq!(
            git_url.join("..").expect("join failed").to_string(),
            "git@github.com:user/foo/bar"
        );

        assert_eq!(
            git_url.join("../aaa").expect("join failed").to_string(),
            "git@github.com:user/foo/bar/aaa"
        );

        assert_eq!(
            git_url.join("../aaa/bbb").expect("join failed").to_string(),
            "git@github.com:user/foo/bar/aaa/bbb"
        );

        assert_eq!(
            git_url
                .join("../../../aaa/bbb")
                .expect("join failed")
                .to_string(),
            "git@github.com:user/aaa/bbb"
        );

        assert_eq!(
            git_url
                .join("../../../../aaa/bbb")
                .expect("join failed")
                .to_string(),
            "git@github.com:aaa/bbb"
        );

        assert!(git_url.join("/aaa").is_none());
    }

    #[test]
    fn test_join_mut() {
        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("aaa"));
            assert_eq!(
                git_url.to_string(),
                "git@github.com:user/foo/bar/quux.git/aaa"
            )
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("aaa/bbb"));
            assert_eq!(
                git_url.to_string(),
                "git@github.com:user/foo/bar/quux.git/aaa/bbb"
            )
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("."));
            assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar/quux.git")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut(".."));
            assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("../aaa"));
            assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar/aaa")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("../aaa/bbb"));
            assert_eq!(git_url.to_string(), "git@github.com:user/foo/bar/aaa/bbb")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("../../../aaa/bbb"));
            assert_eq!(git_url.to_string(), "git@github.com:user/aaa/bbb")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(git_url.join_mut("../../../../aaa/bbb"));
            assert_eq!(git_url.to_string(), "git@github.com:aaa/bbb")
        }

        {
            let mut git_url =
                GitUrl::parse("git@github.com:user/foo/bar/quux.git").expect("parse failed");
            assert!(!git_url.join_mut("/aaa"))
        }
    }
}
