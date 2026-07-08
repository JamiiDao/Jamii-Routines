import { useState } from "react";
import { useNavigate } from "react-router-dom";

import { AppRoutes, AppServerRoutes } from "../App";
import DashboardHeader from "../components/DashboardHeader";
import ActionButton from "../components/ActionButton";

export default function Logout() {
    const navigate = useNavigate();

    const [logoutStateData, setLogoutStateData] = useState("");

    const logoutHandler = async () => {
        try {
            const response = await fetch(AppServerRoutes.logout, {
                method: "GET",
                credentials: "include",
            });


            if (response.status === 200) {
                navigate(AppRoutes.login)
            }
            else {
                const parsedJson = await response.json();
                console.log(parsedJson);
                setLogoutStateData(parsedJson.message);
            }

        } catch (err) {
            const error = err instanceof Error ? err.message : String(err);

            setLogoutStateData(error);
        }
    };

    return (
        <div className="flex-col flex-1 flex w-full">
            <DashboardHeader />

            <div className="flex flex-col flex-1 w-full items-center justify-center">
                <h3>Are you sure you want to logout?</h3>

                {(logoutStateData !== "") && (<div className="text-white py-2 px-4 bg-red-600 text-wrap">{logoutStateData}</div>)}

                <ActionButton onClick={logoutHandler} bgBackground="bg-complement" minWidth="w-[200px]">
                    Yes
                </ActionButton>

            </div>
        </div>
    )

}

