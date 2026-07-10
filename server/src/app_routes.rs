#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppRoutes {
    Root,
    Login,
    Logout,
    SignUp,
    About,
    Dashboard,
    VerifyCode,
    ResendCode,
    RegisterPasskeyChallenge,
    RegisterPasskey,
    ConnectPasskey,
    VerifyPasskey,
}

impl AppRoutes {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "/",
            Self::Login => "/login",
            Self::Logout => "/logout",
            Self::SignUp => "/signup",
            Self::About => "/about",
            Self::Dashboard => "/dashboard",
            Self::VerifyCode => "/verify-code",
            Self::ResendCode => "/resend-code",
            Self::RegisterPasskeyChallenge => "/passkey-challenge",
            Self::RegisterPasskey => "/register-passkey",
            Self::ConnectPasskey => "/connect-passkey",
            Self::VerifyPasskey => "/verify-passkey",
        }
    }
}
