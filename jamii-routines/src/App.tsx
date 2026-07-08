import { Route, Routes } from "react-router-dom";
import { Home, About, Login, SignUp } from "./views";
import Verify from "./views/AuthCode";
import Dashboard from "./views/Dashboard/root";
import Logout from "./views/Logout";


export default function App() {

  return (
    <div className="relative min-h-screen w-full overflow-x-clip font-normal">
      <main className="relative z-10 mx-auto flex flex-col min-h-screen">
        <Routes>
          <Route path={AppRoutes.home} element={<Home />} />
          <Route path={AppRoutes.about} element={<About />} />
          <Route path={AppRoutes.login} element={<Login />} />
          <Route path={AppRoutes.signup} element={<SignUp />} />
          <Route path={AppRoutes.verify} element={<Verify />} />
          <Route path={AppRoutes.dashboard} element={<Dashboard />} />
          <Route path={AppRoutes.logout} element={<Logout />} />
        </Routes>
      </main>
    </div>);
}


export const AppRoutes = {
  home: "/",
  about: "/about",
  login: "/login",
  signup: "/signup",
  verify: "/verify-code",
  dashboard: "/dashboard",
  txs: "/transactions",
  passkey: "/passkey",
  logout: "/logout",
} as const;



export const AppServerRoutes = {
  login: "/api/login",
  logout: "/api/logout",
  signup: "/api/signup",
  verify: "/api/verify-code",
  resend: "/api/resend-code",
  dashboard: "/api/dashboard",
  assets: "/assets",
  passkeyChallenge: "/api/passkey-challenge",
  registerPasskey: "/api/register-passkey",
  verifyPasskey: "/api/verify-passkey",
} as const;