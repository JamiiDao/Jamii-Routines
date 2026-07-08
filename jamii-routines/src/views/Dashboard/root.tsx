import { useEffect, useState } from "react";
import DashboardHeader from "../../components/DashboardHeader";
import Loader from "../../components/Loader";
import { AppRoutes, AppServerRoutes } from "../../App";
import PasskeyOp from "./PasskeyOp";
import { useNavigate } from "react-router-dom";


type DashboardState = {
    loading: boolean;
    user: string | null;
    error: string;
    createTime: String;
    passkeyExists: boolean;
};

export default function Dashboard() {
    const navigate = useNavigate();


    const [dashboardState, setDashboardState] = useState<DashboardState>({
        loading: true,
        user: null,
        error: "",
        createTime: "",
        passkeyExists: false
    });

    useEffect(() => {
        setDashboardState((current) => ({
            ...current,
            user: localStorage.getItem("user")
        }));

        const loadDashboard = async () => {
            try {
                const response = await fetch(AppServerRoutes.dashboard, {
                    method: "GET",
                    credentials: "include",
                });


                if (response.status === 200) {
                    const parsedJson = await response.json();
                    console.log(parsedJson);

                    setDashboardState((current) => ({
                        ...current,
                        passkeyExists: parsedJson.passkeyExists,
                        createTime: parsedJson.createTime
                    }));
                } else if (response.status === 401) {
                    navigate(AppRoutes.login)
                }
                else {
                    const parsedJson = await response.json();
                    console.log(parsedJson);

                    setDashboardState((current) => ({
                        ...current,
                        error: parsedJson.message
                    }));
                }

            } catch (err) {
                const error = err instanceof Error ? err.message : String(err);

                setDashboardState((current) => ({
                    ...current,
                    error: error
                }));
            } finally {
                setDashboardState((current) => ({
                    ...current,
                    loading: false,
                    error: ""
                }));
            }
        };

        loadDashboard();
    }, []);




    return (
        <div className="flex flex-col min-h-screen w-full px-4">
            <DashboardHeader />

            <div className="flex flex-col flex-1 items-center justify-center w-full px-4">

                {dashboardState.loading ? (
                    <Loader />
                ) : dashboardState.error !== "" ? (
                    <div className="rounded-lg bg-red-700 p-4 text-white">
                        {dashboardState.error}
                    </div>
                ) : (
                    <>
                        {(!dashboardState.passkeyExists) && (
                            <PasskeyOp />
                        )}

                        {(dashboardState.passkeyExists) && (
                            <div>Transactions</div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
}

