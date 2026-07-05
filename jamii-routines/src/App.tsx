import { Route, Routes } from "react-router-dom";
import { Home, About, Login, SignUp } from "./views";


export default function App() {
  return (
    <div className="relative min-h-screen w-full overflow-x-clip font-normal">
      <main className="relative z-10 mx-auto flex flex-col min-h-screen">
        <Routes>
          <Route path={AppRoutes.home} element={<Home />} />
          <Route path={AppRoutes.about} element={<About />} />
          <Route path={AppRoutes.login} element={<Login />} />
          <Route path={AppRoutes.signup} element={<SignUp />} />
        </Routes>
      </main>
    </div>);
}


export const AppRoutes = {
  home: "/",
  about: "/about",
  login: "/login",
  signup: "/signup",
} as const;