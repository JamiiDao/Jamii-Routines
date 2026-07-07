import { useEffect, useState } from "react";
import Header from "../components/header";
import { AppRoutes } from "../App";
import { useNavigate } from "react-router-dom";
import Loader from "../components/Loader";
import getHref from "../components/href";

export default function Login() {
    const navigate = useNavigate();

    useEffect(() => {
        localStorage.removeItem("user");
    }, []);


    const [email, setEmail] = useState("");

    const isValidEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);

    const [processing, setProcessing] = useState(false);

    const [response, setResponse] = useState({
        status: 0,
        error: "",
    });
    const handleEmailChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setEmail(e.target.value.trim());

        setResponse({
            status: 0,
            error: "",
        });
    };

    const runProcessing = async () => {
        setProcessing(true);

        try {
            const body = {
                email: email || null,
            };

            const response = await fetch(getHref(AppRoutes.signup), {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify(body),
            });


            if (response.status === 200) {
                const parsedResponse = await response.json();

                if (parsedResponse.email === undefined) {
                    setResponse({
                        status: 0,
                        error: "Email is missing"
                    });

                    return;
                }

                localStorage.setItem("user", parsedResponse.email);

                navigate("/verify-code");
            }

            else {
                const parseResponse = await response.json();

                setResponse({
                    status: response.status,
                    error: parseResponse.message,
                });
            }

        } catch (err) {
            setResponse({
                status: 0,
                error: err instanceof Error ? err.message : String(err),
            });
        }

        setProcessing(false);

    };

    return (
        <>
            <Header />
            <div className="flex flex-col w-full items-center justify-center min-h-full flex-1">
                <div className="flex flex-col justify-center items-center ">
                    <IdCardPhone
                        width="clamp(150px, 20vw, 200px)"
                        height="clamp(150px, 20vw, 200px)"
                    />

                    <div className="font-heading mb-20 text-4xl text-accent">
                        Create an Account
                    </div>
                </div>
                <div className="w-full max-w-lg mx-auto">
                    <div className="flex w-full shadow-xs rounded-base">
                        <span className="inline-flex items-center px-3 text-sm text-body bg-neutral-tertiary border rounded-s-full border-default-medium border-e-0 rounded-s-base">
                            <svg
                                className="w-4 h-4 text-body"
                                aria-hidden="true"
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                fill="none"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke="currentColor"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth="2"
                                    d="M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm0 0a8.949 8.949 0 0 0 4.951-1.488A3.987 3.987 0 0 0 13 16h-2a3.987 3.987 0 0 0-3.951 3.512A8.948 8.948 0 0 0 12 21Zm3-11a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                />
                            </svg>
                        </span>

                        <input
                            type="text"
                            id="email-data"
                            value={email}
                            disabled={processing}
                            onChange={handleEmailChange}
                            className="rounded-none rounded-e-full block w-full px-3 py-2.5 bg-neutral-secondary-medium border border-default-medium text-heading text-sm focus:ring-brand focus:border-brand placeholder:text-body"
                            placeholder="Enter your email address"
                        />
                    </div>

                    {email !== "" && !isValidEmail && (
                        <p className="mt-2 text-sm text-red-500">
                            Invalid email address.
                        </p>
                    )}

                    {response.error && (
                        <div className="mt-2 flex tex-wrap text-lg font-bold bg-red-500 rounded-xl p-2">
                            <p className="p-1">{response.error}</p>
                        </div>
                    )}

                    <div className="flex flex-col w-full items-center justify-center text-xl font-heading">
                        {(!processing && response.status !== 404) && (
                            <button
                                onClick={runProcessing}
                                disabled={!isValidEmail || processing}
                                className="min-w-sm tracking-widest bg-secondary text-white px-8 mt-10 py-1.5 rounded-full enabled:transition-all enabled:duration-200 disabled:bg-secondary/40 disabled:opacity-50 disabled:cursor-not-allowed enabled:hover:scale-105">
                                Sign Up
                            </button>
                        )}

                        {(processing) && <Loader />}

                    </div>
                </div>
            </div>
        </>
    );
}


type IdCardPhoneProps = {
    width: string;
    height: string;
};

function IdCardPhone({ width, height }: IdCardPhoneProps) {
    return (
        <svg
            width={width}
            height={height} viewBox="0 0 485.2 485.2" xmlns="http://www.w3.org/2000/svg"><g fill="var(--accent)"><path d="m137 188.95c-45.2 0-81.9 36.7-81.9 81.9 0 19.4 6.8 37.3 18.1 51.3.8-6.6 3.5-14 11.9-16.2 16.1-4.1 29.1-13.3 29.1-13.3l12.1 38.2 1.7 5.2 5.4-15.3c-13.6-18.9 3.6-18.2 3.6-18.2s17.2-.7 3.6 18.2l5.4 15.2 1.7-5.1v.1l1.9-6.1 10.2-32.2s13 9.2 29.1 13.3c8.4 2.1 11.2 9.6 11.9 16.2 11.3-14 18.1-31.9 18.1-51.3-.1-45.2-36.7-81.9-81.9-81.9zm31.4 72.1c-2.1 7.4-4 9.4-5.5 9.6-1.4 8.8-8.3 19.9-19.3 23.8-4.5 1.6-9.6 1.6-14.1 0-11.1-3.9-17.9-15.1-19.2-23.9-1.5-.1-3.4-2.2-5.5-9.6-2.9-10.1.2-11.6 2.7-11.3-.5-1.4-.9-2.8-1.1-4.2-.9-4.7-1.2-9.1-.1-13.3 1.3-5.6 4.4-10.1 7.8-13.6 2.2-2.3 4.6-4.3 7.2-5.9 2.1-1.5 4.4-2.7 7-3.6 2-.7 4.1-1.1 6.3-1.2 6.9-.6 12.2 1.1 15.9 3.4 5.6 3.1 7.8 7.2 7.8 7.2s12.9.9 8.5 27.1c-.3 1.4-.6 2.8-1.1 4.2 2.6-.3 5.6 1.2 2.7 11.3z" /><path d="m187.8 130.75h109.6c11 0 19.9-8.9 19.9-19.9v-13.5c0-11-8.9-19.9-19.9-19.9h-6.2v-8.5c0-8.4-6.8-15.3-15.3-15.3h-66.6c-8.4 0-15.3 6.8-15.3 15.3v8.5h-6.2c-11 0-19.9 8.9-19.9 19.9v13.5c0 11 8.9 19.9 19.9 19.9z" /><path d="m429.4 105.85h-93.6v5c0 21.2-17.2 38.4-38.4 38.4h-109.6c-21.2 0-38.4-17.2-38.4-38.4v-5h-93.6c-30.8 0-55.8 25-55.8 55.8v214.1c0 30.8 25 55.8 55.8 55.8h373.6c30.8 0 55.8-25 55.8-55.8v-214.2c0-30.8-25-55.7-55.8-55.7zm-292.8 266.4c-56 0-101.4-45.4-101.4-101.4s45.4-101.4 101.4-101.4 101.4 45.4 101.4 101.4-45.4 101.4-101.4 101.4zm294.1-49.4h-162.8v-20.8h162.8zm0-43.8h-162.8v-20.8h162.8zm0-43.8h-162.8v-20.8h162.8z" /></g></svg>)
}