import { invoke } from "@tauri-apps/api/tauri";
import React, { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import "./Results.css";

function Result(props: { resultHash: string }) {
    const [resultHtml, setResultHtml] = useState(<></>);
    useMemo(
        () => {
            invoke("get_result", { fileHash: props.resultHash })
                .then((res) => {
                    // Search OK
                    console.log(res)
                    setResultHtml(<button className="resultBlock">
                        <img src={res.image_path} />
                        <span>{res.file_name}</span>
                    </button>);
                })
                .catch((error) => {
                    console.error(error)
                });
        },
        [props]
    );

    return resultHtml;
}

export function ResultsTab(props: { query: string, searchResultHashes: string[] }) {
    let results: string[] = props.searchResultHashes;

    const navigate = useNavigate();
    // Go back when pressing the Esc key
    React.useEffect(() => {
        const handleKeydown = (event: KeyboardEvent) => {
            if (event.key === "Escape") {
                event.preventDefault();
                navigate(-1);
            }
        };

        document.addEventListener('keydown', handleKeydown);

        return () => {
            document.removeEventListener('keydown', handleKeydown);
        };
    });

    return (
        <div className="resultPage">
            {
                results.length > 0 ?
                    results.map((resultHash: string) => (<Result resultHash={resultHash} />)) :
                    <span className="noResults">No results found for "{props.query}"</span>
            }
        </div>
    )
}