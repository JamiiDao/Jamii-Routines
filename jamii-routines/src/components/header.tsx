import { useState } from "react";
import { Link } from "react-router-dom";

import Logo from "./logo";
import { AppRoutes } from "../App";


export default function Header() {
    const [isHidden, setIsHidden] = useState(true);

    const handleClick = () => {
        setIsHidden(current => !current);
    };


    return (
        <header className="">
            <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">

                <Link to={AppRoutes.home} className="flex items-center gap-2">
                    <span className="w-[clamp(24px,29px,29px)] font-bold flex items-center">
                        <Logo /></span>
                    <span className="text-2xl text-accent font-heading leading-none">
                        Routines
                    </span>
                </Link>

                <nav className="hidden items-center space-x-8 md:flex opacity-100">
                    <Link to={AppRoutes.home} className="hover:text-secondary">Home</Link>
                    <Link to={AppRoutes.about} className="hover:text-secondary">About</Link>
                    <Link to={AppRoutes.login} className="hover:text-secondary">Login</Link>
                    <Link to={AppRoutes.signup} className="hover:text-secondary">Sign Up</Link>
                </nav>

                <button
                    id="menu-btn"
                    className="rounded p-2 md:hidden"
                    aria-label="Open Menu"
                    onClick={handleClick}
                >
                    <svg
                        className="h-6 w-6"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6h16M4 12h16M4 18h16"
                        />
                    </svg>
                </button>
            </div>

            <div id="mobile-menu" className="">
                <nav className={`  ${isHidden ? "hidden" : ""} space-y-1 px-4 py-4`}>
                    <Link to={AppRoutes.home} className="block hover:text-secondary rounded px-3 py-2">Home</Link>
                    <Link to={AppRoutes.about} className="block hover:text-secondary rounded px-3 py-2">About</Link>
                    <Link to={AppRoutes.login} className="block hover:text-secondary rounded px-3 py-2">Login</Link>
                    <Link to={AppRoutes.signup} className="block hover:text-secondary rounded px-3 py-2">Sign Up</Link>
                </nav>
            </div>
        </header >
    )

}