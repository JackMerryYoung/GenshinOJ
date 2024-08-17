import { CSSProperties, MouseEventHandler, ReactNode, useEffect, useState } from "react";

export default function BadgeButton({ children, style, onClick }: {
    children: ReactNode;
    style?: CSSProperties | undefined;
    onClick?: MouseEventHandler<HTMLDivElement> | undefined;
}) {
    const [stateHover, setStateHover] = useState(false);
    const [stateClick, setStateClick] = useState(false);

    useEffect(() => {
        if (stateClick === true)
            setTimeout(() => setStateClick(false), 500);
    }, [stateClick]);

    const handleMouseEnter = () => {
        setStateHover(true);
    };

    const handleMouseLeave = () => {
        setStateHover(false);
    };

    const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
        if (onClick) onClick(event); setStateClick(true);
    };

    return <div style={{ ...style, display: "flex", padding: "0 4px", alignItems: "center", border: "1px solid", borderWidth: "1px", borderRadius: "3px", backgroundColor: stateClick ? "rgb(65, 131, 196)" : (stateHover ? "rgba(65, 131, 196, 0.1)" : "rgba(65, 131, 196, 0)"), color: "#4183C4" }}
        onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave}
        onMouseDown={handleClick}>
        <div style={{ margin: "0 2px 0 2px", color: stateClick ? "#ffffff" : "#4183C4", lineHeight: "1.5", display: "flex", alignItems: "center" }}>
            {children}
        </div>
    </div>;
}