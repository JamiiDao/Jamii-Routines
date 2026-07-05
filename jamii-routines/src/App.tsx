import { Route, Routes } from "react-router-dom";
import { Home, About, Login, SignUp } from "./views";
import { useEffect } from "react";


export default function App() {

  useEffect(() => {
    const run = async () => {
      const res = await fetch("http://127.0.0.1:3000/");
      const text = await res.text();
      console.log("RES TEXT:", text);
    };

    run();
  }, []);

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

function useState(arg0: string): [any, any] {
  throw new Error("Function not implemented.");
}
