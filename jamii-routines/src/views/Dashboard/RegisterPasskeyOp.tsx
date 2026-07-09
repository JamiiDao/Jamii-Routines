import { useState } from "react";
import {
    bufferToBase64URLString,
    startRegistration,
    WebAuthnError,
} from "@simplewebauthn/browser";

import ActionButton from "../../components/ActionButton";
import { AppServerRoutes } from "../../App";
import Loader from "../../components/Loader";
import { PasskeyPhone } from "./root";

export default function PasskeyOp() {
    const [passkeyStatus, setPasskeyStatus] = useState(
        {
            state: PasskeyOpStatus.Pending,
            error: ""
        }
    );

    const handleConnectPasskey = async () => {
        setPasskeyStatus((current) => ({
            ...current,
            state: PasskeyOpStatus.Connect
        }));

        let registrationResponse;

        try {
            const response = await fetch((AppServerRoutes.passkeyChallenge), {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include"
            });


            console.log("STATUS: ", response.status);

            if (response.status === 200) {
                const parsed = await response.json();
                parsed.publicKey.challenge = bufferToBase64URLString(
                    parsed.publicKey.challenge
                );

                parsed.publicKey.user.id = bufferToBase64URLString(
                    parsed.publicKey.user.id
                );
                console.log("Text: ", parsed);


                try {
                    registrationResponse = await startRegistration({
                        optionsJSON: parsed.publicKey,
                    });

                } catch (error) {
                    if (error instanceof WebAuthnError) {
                        console.error(error.code);
                        console.error(error.message);
                    }

                    setPasskeyStatus((current) => ({
                        ...current,
                        error: "Passkey request rejected!",
                    }));
                }

                console.log("registrationResponse: ", registrationResponse);

            } else {
                const parsed = await response.json();

                setPasskeyStatus((current) => ({
                    ...current,
                    error: parsed.message,
                }));
            }


        } catch (err) {
            setPasskeyStatus((current) => ({
                ...current,
                state: PasskeyOpStatus.Pending,
                error: err instanceof Error ? err.message : String(err),
            }));
        }

        if (registrationResponse !== null || registrationResponse !== undefined) {
            setPasskeyStatus((current) => ({
                ...current,
                state: PasskeyOpStatus.Verifying,
                error: ""
            }));

            try {
                const response = await fetch((AppServerRoutes.registerPasskey), {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    credentials: "include",
                    body: JSON.stringify(registrationResponse),
                });


                console.log("STATUS: ", response.status);

                if (response.status === 200) {
                    const parsedJson = await response.json();

                    console.log("SUCCESS: ", parsedJson);
                    sessionStorage.setItem("passkey", JSON.stringify(parsedJson));

                    setPasskeyStatus((current) => ({
                        ...current,
                        state: PasskeyOpStatus.Registered,
                        error: "",
                    }));
                } else {
                    const parsedJson = await response.json();
                    setPasskeyStatus((current) => ({
                        ...current,
                        state: PasskeyOpStatus.Pending,
                        error: parsedJson.message,
                    }));
                }

            } catch (err) {
                setPasskeyStatus((current) => ({
                    ...current,
                    state: PasskeyOpStatus.Pending,
                    error: err instanceof Error ? err.message : String(err),
                }));
            }
        }

    };

    const handleTryAgainConnectPasskey = async () => {
        setPasskeyStatus((current) => ({
            ...current,
            state: PasskeyOpStatus.Pending,
            error: ""
        }));
    }


    return (
        <>
            <div className="flex flex-col flex-1 items-center justify-center w-full px-4">
                {(passkeyStatus.state !== PasskeyOpStatus.Registered) && (
                    <>
                        <div className="flex flex-col text-white w-full justify-center items-center max-w-lg rounded-2xl bg-secondary p-10 shadow-xl">
                            <PasskeyPhone />
                            <div className="text-4xl font-heading">Connect A Passkey</div>

                            <div className="text-lg text-center mt-10 text-normal">
                                A Passkey is required to create a spending account. You can use your phone biometrics, TPM or any FIDO USB key.
                            </div>

                            {(passkeyStatus.state === PasskeyOpStatus.Connect && passkeyStatus.error === "") && (<div className="text-lg text-center mt-10 text-normal">
                                Passkey connecting...
                            </div>)}

                            {(passkeyStatus.state === PasskeyOpStatus.Verifying && passkeyStatus.error === "") && (
                                <>
                                    <div className="text-lg text-center mt-10 text-normal">
                                        Passkey is being verified...
                                    </div>
                                    <Loader />
                                </>
                            )}

                            {(passkeyStatus.error !== "") &&
                                (<>
                                    <div className="bg-red-600 px-4 py-2 w-full mt-5 rounded-lg font-bold text-lg text-wrap">{passkeyStatus.error}</div>

                                    <ActionButton onClick={handleTryAgainConnectPasskey} bgBackground="bg-complement" marginTop="mt-6">
                                        Try Again
                                    </ActionButton>
                                </>
                                )
                            }
                        </div>


                        {(passkeyStatus.state === PasskeyOpStatus.Pending && passkeyStatus.error === "") && (
                            <ActionButton onClick={handleConnectPasskey}>
                                Connect Passkey
                            </ActionButton>
                        )}
                    </>
                )}
            </div>


        </>
    );
}

enum PasskeyOpStatus {
    Pending,
    Connect,
    Verifying,
    Registered
}

