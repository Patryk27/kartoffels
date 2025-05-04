use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionRole {
    Admin,
    #[default]
    User,
}

impl SessionRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn is_user(&self) -> bool {
        matches!(self, Self::User)
    }
}

impl fmt::Display for SessionRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Admin => "admin",
                Self::User => "user",
            }
        )
    }
}
