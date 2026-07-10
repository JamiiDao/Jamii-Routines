mod login;
pub use login::*;

mod logout;
pub use logout::*;

mod signup;
pub use signup::*;

mod auth_code_mailer;
pub use auth_code_mailer::*;

mod verify_auth_code;
pub use verify_auth_code::*;

mod dashboard;
pub use dashboard::*;

pub struct RouteHandler;
