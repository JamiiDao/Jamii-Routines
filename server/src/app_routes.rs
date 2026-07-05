#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppRoutes {
    Login,
    SignUp,
    About,
    Dashboard,
}

impl AppRoutes {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "/login",
            Self::SignUp => "/signup",
            Self::About => "/about",
            Self::Dashboard => "/dashboard",
        }
    }
}
