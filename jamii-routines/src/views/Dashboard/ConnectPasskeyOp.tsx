import { useState } from "react";
import {
    bufferToBase64URLString,
    startAuthentication,
    startRegistration,
    WebAuthnError,
} from "@simplewebauthn/browser";

import ActionButton from "../../components/ActionButton";
import { AppRoutes, AppServerRoutes } from "../../App";
import Loader from "../../components/Loader";
import { PasskeyPhone } from "./root";
import { error } from "console";
import { useNavigate } from "react-router-dom";

export default function ConnectPasskey() {
    const navigate = useNavigate()

    enum PasskeyVerifiedState {
        Unverified,
        Verified,
    }

    enum PasskeyConnectState {
        Initial,
        Connecting,
        Verifying,
    }

    const [passkeyStatus, setPasskeyStatus] = useState(
        {
            verifyState: PasskeyVerifiedState.Unverified,
            state: PasskeyConnectState.Initial,
            error: ""
        }
    );

    const handleConnectPasskey = async () => {
        setPasskeyStatus((current) => ({
            ...current,
            state: PasskeyConnectState.Connecting,
            error: ""
        }));



        let authenticationResponse;

        try {
            const response = await fetch((AppServerRoutes.connectPasskey), {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include"
            });

            console.log("STATUS: ", response.status);

            if (response.status === 200) {
                const parsedResponse = await response.json();

                parsedResponse.publicKey.challenge = bufferToBase64URLString(
                    parsedResponse.publicKey.challenge,
                );

                for (const cred of parsedResponse.publicKey.allowCredentials) {
                    cred.id = bufferToBase64URLString(cred.id);
                }

                console.log("SUCCESS", parsedResponse);

                try {
                    authenticationResponse = await startAuthentication({
                        optionsJSON: parsedResponse.publicKey,
                    });

                    setPasskeyStatus((current) => ({
                        ...current,
                        state: PasskeyConnectState.Verifying,
                        error: ""
                    }));

                    try {
                        const verifyPasskeyResponse = await fetch((AppServerRoutes.verifyPasskey), {
                            method: "POST",
                            headers: {
                                "Content-Type": "application/json",
                            },
                            credentials: "include",
                            body: JSON.stringify(authenticationResponse)
                        });

                        if (response.status === 200) {
                            console.log("Status code verify: ", verifyPasskeyResponse.status);
                            setPasskeyStatus((current) => ({
                                ...current,
                                state: PasskeyConnectState.Initial,
                                verifyState: PasskeyVerifiedState.Verified,
                                error: "",
                            }));
                        } else {
                            console.log("Status code verify: ", verifyPasskeyResponse.status);
                            const parsedJsonError = await verifyPasskeyResponse.json();

                            setPasskeyStatus((current) => ({
                                ...current,
                                state: PasskeyConnectState.Initial,
                                error: parsedJsonError.message,
                            }));
                        }
                    } catch (err) {
                        const error = err instanceof Error ? err.message : String(err);

                        setPasskeyStatus((current) => ({
                            ...current,
                            state: PasskeyConnectState.Initial,
                            error: error,
                        }));
                    }

                    console.log("Authentication Response: ", authenticationResponse);
                }
                catch (err) {
                    if (err instanceof WebAuthnError) {
                        console.error(err.code);
                        console.error(err.message);
                    }

                    setPasskeyStatus((current) => ({
                        ...current,
                        state: PasskeyConnectState.Initial,
                        error: "Passkey request rejected!",
                    }));
                }

            } else if (response.status === 401) {
                navigate(AppRoutes.login)
            } else {
                const parsedResponse = await response.json();

                console.error("ERROR: ", parsedResponse);

                setPasskeyStatus((current) => ({
                    ...current,
                    state: PasskeyConnectState.Initial,
                    error: parsedResponse.message
                }));
            }
        } catch (err) {
            const error = err instanceof Error ? err.message : String(err);

            setPasskeyStatus((current) => ({
                ...current,
                state: PasskeyConnectState.Initial,
                error: error,
            }));
        }
    }

    return (
        <>
            {(passkeyStatus.verifyState === PasskeyVerifiedState.Unverified) && (
                <>
                    <div className="flex flex-col text-white w-full justify-center items-center max-w-lg rounded-2xl bg-secondary p-10 shadow-xl">
                        <PasskeyPhone />
                        <div className="text-4xl font-heading">Link Passkey</div>

                        {(passkeyStatus.state === PasskeyConnectState.Initial) && (
                            <div className="text-lg text-center mt-10 text-normal">
                                Connect your passkey to link your onchain account
                            </div>
                        )}

                        {(passkeyStatus.state === PasskeyConnectState.Connecting) && (
                            <>
                                <div className="text-lg text-center mt-10 text-normal">
                                    Linking passkey...
                                </div>
                            </>
                        )}

                        {(passkeyStatus.state === PasskeyConnectState.Verifying) && (
                            <>
                                <div className="text-lg text-center mt-10 text-normal">
                                    Verifying passkey operation...
                                </div>
                            </>
                        )}

                        {(passkeyStatus.error !== "") && (
                            <>
                                <div className="text-lg bg-red-600 px-4 py-2 rounded-lg text-center mt-10 text-normal">
                                    {passkeyStatus.error}
                                </div>
                            </>
                        )}
                    </div>

                    <ActionButton onClick={handleConnectPasskey}>
                        Connect Passkey
                    </ActionButton>
                </>
            )}

            {(passkeyStatus.verifyState === PasskeyVerifiedState.Verified) && (
                <div>Passkey verified</div>
            )}
        </>
    )
}