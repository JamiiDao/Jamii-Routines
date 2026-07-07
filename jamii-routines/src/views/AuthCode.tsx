import { useMemo, useRef, useState, useEffect } from "react";
import Header from "../components/header";
import Loader from "../components/Loader";
import getHref from "../components/href";
import { AppRoutes } from "../App";
import { useNavigate } from "react-router-dom";

type LoadingState = {
    state: boolean;
    user: string | null;
    error: string;
};

export default function Verify() {
    const [initialLoading, setInitialLoading] = useState<LoadingState>({
        state: true,
        user: null,
        error: ""
    });

    useEffect(() => {
        const user = localStorage.getItem('user');
        setInitialLoading((previous) => ({
            ...previous,
            user: user,
        }));

        const sendResendCode = async () => {
            try {
                if (user === null) {
                    setInitialLoading((previous) => ({
                        state: false,
                        user: previous.user,
                        error: "User not found",
                    }));
                }
                else {
                    await fetch(getHref(AppRoutes.resend), {
                        method: "POST",
                        credentials: "include",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            email: user,
                        } as { email: string }),
                    });
                }
            } catch (err) {
                setInitialLoading((previous) => ({
                    state: false,
                    user: previous.user,
                    error: err instanceof Error ? err.message : String(err),
                }));
            } finally {
                setInitialLoading({
                    state: false,
                    user: user,
                    error: ""
                });

            }
        };

        sendResendCode();
    }, []);

    return (
        <div className="flex flex-col min-h-screen w-full px-4">
            <Header />

            <div className="flex flex-col flex-1 items-center justify-center w-full px-4">
                <div className="w-full items-center max-w-lg rounded-2xl bg-secondary p-10 shadow-xl">

                    <h1 className="text-center text-3xl font-bold font-heading tracking-widest">
                        Email Authentication
                    </h1>

                    {initialLoading.state && (
                        <>
                            <p className="mt-4 text-center">
                                Resending the 6-digit verification code to {initialLoading.user} .
                            </p>

                            <div className="flex flex-col w-full p-6">

                                <Loader />
                            </div>
                        </>
                    )}

                    {(!initialLoading.state && initialLoading.error !== "") && (
                        <>
                            <p className="mt-4 text-center p-6 rounded-lg bg-red-700 text-white">
                                {initialLoading.error}
                            </p>

                            <div className="flex flex-col w-full p-6">

                                <Loader />
                            </div>
                        </>
                    )}

                    {
                        (!initialLoading.state) && (
                            <CodeInput />
                        )
                    }
                </div>
            </div >
        </div >
    );
}

function CodeInput() {
    const navigator = useNavigate();

    const [verifyState, setVerifyState] = useState<LoadingState>({
        state: false,
        user: localStorage.getItem('user'),
        error: ""
    });

    const [verifyResponseState, setVerifyResponseState] = useState({
        state: false,
        error: ""
    });



    const [code, setCode] = useState(Array(6).fill(""));
    const inputs = useRef<(HTMLInputElement | null)[]>([]);

    const complete = useMemo(
        () => code.every((c) => c !== ""),
        [code]
    );

    function updateValue(index: number, value: string) {
        const character = value
            .replace(/[^a-zA-Z0-9]/g, "")
            .slice(-1)
            .toUpperCase();

        const next = [...code];
        next[index] = character;
        setCode(next);

        if (character && index < 5) {
            inputs.current[index + 1]?.focus();
        }
    }

    function onKeyDown(
        index: number,
        event: React.KeyboardEvent<HTMLInputElement>
    ) {
        setVerifyResponseState({
            state: false,
            error: "",
        });

        if (event.key === "Backspace") {
            if (code[index]) {
                const next = [...code];
                next[index] = "";
                setCode(next);
            } else if (index > 0) {
                inputs.current[index - 1]?.focus();

                const next = [...code];
                next[index - 1] = "";
                setCode(next);
            }

            event.preventDefault();
        }
    }

    function onPaste(event: React.ClipboardEvent<HTMLInputElement>) {
        event.preventDefault();

        setVerifyResponseState({
            state: false,
            error: "",
        });

        const pasted = event.clipboardData
            .getData("text").toUpperCase()
            .replace(/[^a-zA-Z0-9]/g, "")
            .slice(0, 6)
            .split("");
        const next = Array(6).fill("");

        pasted.forEach((c, i) => {
            next[i] = c;
        });

        setCode(next);

        const focus = Math.min(pasted.length, 5);
        inputs.current[focus]?.focus();
    }

    const [timerInfo, setTimer] = useState(60);


    useEffect(() => {
        const interval = setInterval(() => {
            setTimer((current) => {
                if (current === 0) {
                    return 0;
                }

                return current - 1;
            });
        }, 1000);

        return () => clearInterval(interval);
    }, []);

    const handleResend = async () => {
        setVerifyState((current) => ({
            state: true,
            user: current.user,
            error: current.error,
        }));

        setVerifyResponseState({
            state: false,
            error: "",
        });

        try {
            await fetch(getHref(AppRoutes.resend), {
                method: "POST",
                credentials: "include",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    email: verifyState.user,
                }),
            });

        } catch (err) {
            setVerifyState((current) => ({
                state: false,
                user: current.user,
                error: err instanceof Error ? err.message : String(err),
            }));
        } finally {
            setVerifyState((current) => ({
                state: false,
                user: current.user,
                error: ""
            }));

            setTimer(60)
        }
    };

    async function onSubmit(event: React.SubmitEvent) {
        event.preventDefault();

        setVerifyState((current) => ({
            state: false,
            user: current.user,
            error: "",
        }));

        setVerifyResponseState((current) => ({
            ...current,
            state: true,
        }));

        const verificationCode = code.join("");

        const response = await fetch(getHref(AppRoutes.verify), {
            method: "POST",
            credentials: "include",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                email: verifyState.user,
                code: verificationCode,
            }),
        });

        if (response.status === 200) {
            try {
                const parsedJson = await response.json();

                navigator(parsedJson.path)

            } catch (err) {
                setVerifyResponseState({
                    state: false,
                    error: err instanceof Error ? err.message : String(err)
                });
            }
        }

        try {
            const parsedJson = await response.json();

            setVerifyResponseState({
                state: false,
                error: parsedJson.message
            });
        } catch (err) {
            setVerifyResponseState({
                state: false,
                error: err instanceof Error ? err.message : String(err)
            });
        }

        setVerifyState((current) => ({
            state: false,
            user: current.user,
            error: "",
        }));

    }

    return (
        <>
            <p className="mt-4 text-center">
                Enter the 6-digit verification code sent to your email to prove that you control this email address.
            </p>

            <form
                onSubmit={onSubmit}
                className="mt-8"
            >
                <div className="flex justify-between gap-3">
                    {code.map((value, index) => (
                        <input
                            disabled={verifyState.state || verifyResponseState.state}
                            key={index}
                            ref={(element) => {
                                inputs.current[index] = element;
                            }}
                            type="text"
                            inputMode="numeric"
                            maxLength={1}
                            value={value}
                            autoFocus={index === 0}
                            onPaste={onPaste}
                            onChange={(e) =>
                                updateValue(index, e.target.value)
                            }
                            onKeyDown={(e) =>
                                onKeyDown(index, e)
                            }
                            className="
                                    h-16
                                    w-14
                                    rounded-xl
                                    border-3
                                    border-slate-300
                                    text-center
                                    text-3xl
                                    font-bold
                                    outline-none
                                    transition
                                    focus:border-black
                                    font-monospace
                                "
                        />
                    ))}
                </div>

                {!verifyState.state && (
                    <>
                        {
                            (timerInfo !== 0 && !verifyState.state && !verifyResponseState.state) && (
                                <div className="flex text-xl mt-10 text-white">Resend in {timerInfo} seconds...</div>
                            )
                        }

                        {(timerInfo === 0 && !verifyState.state && !verifyResponseState.state) && (
                            <button
                                onClick={handleResend}
                                type="button" className="flex text-xl mt-10 text-white underline p-2 cursor-pointer">Resend code</button>
                        )}
                    </>
                )}

                {(timerInfo === 0 && verifyState.state) && (
                    <>
                        <div className="flex text-xl mt-10 text-white p-2 cursor-pointer">Resending...</div>
                        <Loader />
                    </>
                )}



                {(verifyState.error !== "") && (
                    <div className="flex text-xl mt-10 text-white p-2 bg-red-700">
                        {verifyState.error}.
                    </div>
                )}


                {(verifyResponseState.error !== "") && (
                    <div className="flex text-xl mt-10 text-white p-2 bg-red-700">
                        {verifyResponseState.error}.
                    </div>
                )}

                {(!verifyState.state && verifyState.error === "" && !verifyResponseState.state && verifyResponseState.error === "") &&
                    (<button
                        type="submit"
                        disabled={!complete}
                        className="
                                mt-8
                                w-full
                                rounded-full
                                bg-complement
                                py-2
                                text-2xl
                                text-accent
                                font-bold
                                font-heading
                                tracking-widest
                                transition-transform
                                duration-200
                                enabled:hover:scale-105
                                enabled:active:scale-95
                                disabled:cursor-not-allowed
                                disabled:bg-complement/10
                                disabled:text-white/10
                            ">
                        Verify
                    </button>
                    )
                }

                {(verifyResponseState.state) && (
                    <>
                        <p className="mt-10 text-xl">Verifying code...</p>
                        <Loader />
                    </>
                )}
            </form>
        </>
    );

}