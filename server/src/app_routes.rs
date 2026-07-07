#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppRoutes {
    Root,
    Login,
    SignUp,
    About,
    Dashboard,
    VerifyCode,
    ResendCode,
}

impl AppRoutes {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "/",
            Self::Login => "/login",
            Self::SignUp => "/signup",
            Self::About => "/about",
            Self::Dashboard => "/dashboard",
            Self::VerifyCode => "/verify-code",
            Self::ResendCode => "/resend-code",
        }
    }
}
