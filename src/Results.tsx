import React from "react";
import { useNavigate } from "react-router-dom";

export function ResultsTab({ searchResultHashes }) {
    const navigate = useNavigate();

    // Go back when pressing the Esc key
    React.useEffect(() => {
        const handleKeydown = (event: KeyboardEvent) => {
            if (event.key === "Escape") {
                navigate(-1);
            }
        };

        document.addEventListener('keydown', handleKeydown);

        return () => {
            document.removeEventListener('keydown', handleKeydown);
        };
    });

    return (
        <>
            <span>Hiii welcome to Results tab</span>
            <h1>Results from the ctx: {searchResultHashes}</h1>
        </>
    )
}