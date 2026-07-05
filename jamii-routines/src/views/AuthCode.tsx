import { useMemo, useRef, useState, useEffect } from "react";
import Header from "../components/header";
import Loader from "../components/Loader";

export default function About() {
    const [timerInfo, setTimer] = useState({
        timer: 60,
        resending: false,
    });

    useEffect(() => {
        const interval = setInterval(() => {
            setTimer((current) => {
                if (current.timer <= 0) {
                    clearInterval(interval);
                    return {
                        ...current,
                        timer: 0
                    }
                }

                return {
                    ...current,
                    timer: current.timer - 1,
                };
            });
        }, 1000);

        return () => clearInterval(interval);
    }, []);

    const [processing, setProcessing] = useState(false);

    const [code, setCode] = useState(Array(6).fill(""));
    const inputs = useRef<(HTMLInputElement | null)[]>([]);

    const complete = useMemo(
        () => code.every((c) => c !== ""),
        [code]
    );

    function updateValue(index: number, value: string) {
        const digit = value.replace(/\D/g, "").slice(-1);

        const next = [...code];
        next[index] = digit;
        setCode(next);

        if (digit && index < 5) {
            inputs.current[index + 1]?.focus();
        }
    }

    function onKeyDown(
        index: number,
        event: React.KeyboardEvent<HTMLInputElement>
    ) {
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

        const pasted = event.clipboardData
            .getData("text")
            .replace(/\D/g, "")
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

    async function onSubmit(event: React.SubmitEvent) {
        event.preventDefault();

        setProcessing(true)

        const verificationCode = code.join("");

        console.log(verificationCode);

        // await fetch("/verify", {
        //     method: "POST",
        //     headers: {
        //         "Content-Type": "application/json",
        //     },
        //     body: JSON.stringify({
        //         code: verificationCode,
        //     }),
        // });
    }

    const handleResend = async () => {
        setProcessing(true)
        setTimer((current) => {
            return {
                resending: true,
                timer: current.timer,
            };
        });
    };

    return (
        <div className="flex flex-col min-h-screen w-full  px-4">
            <Header />

            <div className="flex flex-col flex-1 items-center justify-center w-full  px-4">
                <div className="w-full items-center max-w-lg rounded-2xl bg-secondary p-10 shadow-xl">

                    <h1 className="text-center text-3xl font-bold font-heading tracking-widest">
                        Email Authentication
                    </h1>

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
                                    disabled={processing}
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

                        {!processing && (
                            <>
                                {(timerInfo.timer !== 0 && !timerInfo.resending) && (
                                    <div className="flex text-xl mt-10 text-white">Resend in {timerInfo.timer} seconds...</div>
                                )}

                                {(timerInfo.timer === 0 && !timerInfo.resending) && (
                                    <button onClick={handleResend} type="button" className="flex text-xl mt-10 text-white underline p-2 cursor-pointer">Resend to todo from store</button>
                                )}
                            </>
                        )}



                        {(timerInfo.timer === 0 && timerInfo.resending) && (
                            <div className="flex text-xl mt-10 text-white p-2 cursor-pointer">Resending to todo from store</div>
                        )}

                        {(!processing) &&
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

                        {(processing && !timerInfo.resending) &&
                            (<div className="mt-10 text-xl">Verifying...</div>)}

                        {(processing) && (
                            <Loader />
                        )}

                    </form>
                </div>
            </div>
        </div>
    );
}
