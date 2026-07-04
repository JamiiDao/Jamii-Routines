import { useState } from "react";
import Logo from "./logo";


export default function Header() {
    const [isHidden, setIsHidden] = useState(true);

    const handleClick = () => {
        setIsHidden(current => !current);
    };


    return (
        <header className="">
            <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
                <a href="#" className="text-2xl font-bold text-gray-900">
                    <div className="flex w-[clamp(24px,29px,29px)]"><Logo /></div>
                </a>

                <nav className="hidden items-center space-x-8 md:flex opacity-100">
                    <a href="#" className="text-white hover:text-(--accent-color)">Home</a>

                    <div className="group relative">
                        <button
                            className="flex items-center gap-1 text-white hover:text-(--accent-color)"
                        >
                            Services
                            <svg
                                className="h-4 w-4 transition-transform group-hover:rotate-180"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M19 9l-7 7-7-7"
                                />
                            </svg>
                        </button>

                        <div
                            className="invisible absolute left-0 mt-2 w-48 rounded-lg border  opacity-100 shadow-lg transition-all duration-200 group-hover:visible group-hover:opacity-100"
                        >
                            <a href="#" className="block px-4 py-2 text-white hover:text-(--accent-color)">
                                Web Development
                            </a>
                            <a href="#" className="block px-4 py-2 text-white hover:text-(--accent-color)">
                                Mobile Apps
                            </a>
                            <a href="#" className="block px-4 py-2 text-white hover:text-(--accent-color)">
                                Consulting
                            </a>
                        </div>
                    </div>

                    <a href="#" className="text-white hover:text-(--accent-color)">About</a>
                    <a href="#" className="text-white hover:text-(--accent-color)">Contact</a>
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
                    <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">Home</a>

                    <details className="group">
                        <summary
                            className="flex cursor-pointer list-none items-center justify-between rounded px-3 py-2 hover:bg-gray-100"
                        >
                            <span>Services</span>

                            <svg
                                className="h-4 w-4 transition-transform group-open:rotate-180"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M19 9l-7 7-7-7"
                                />
                            </svg>
                        </summary>

                        <div className="ml-4 mt-1 space-y-1">
                            <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">
                                Web Development
                            </a>
                            <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">
                                Mobile Apps
                            </a>
                            <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">
                                Consulting
                            </a>
                        </div>
                    </details>

                    <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">About</a>
                    <a href="#" className="block rounded px-3 py-2 hover:bg-gray-100">Contact</a>
                </nav>
            </div>
        </header >
    )

}