interface ActionButtonProps {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    marginTop?: string;
    bgBackground?: string;
    minWidth?: string;
}

export default function ActionButton({
    children,
    onClick,
    disabled = false,
    marginTop = "mt-20",
    bgBackground = "bg-secondary",
    minWidth = "",
}: ActionButtonProps) {
    return (
        <button
            type="button"
            disabled={disabled}
            onClick={onClick}
            className={`
                ${marginTop}
                ${bgBackground}
                ${minWidth}
                rounded-full
                px-4
                py-1.5
                text-xl
                md:text-2xl
                text-white
                font-heading
                tracking-widest
                shadow-4xl
                transition-transform
                duration-200
                enabled:hover:scale-105
                enabled:active:scale-95
                disabled:opacity-50
                disabled:cursor-not-allowed
            `}
        >
            {children}
        </button>
    );
}