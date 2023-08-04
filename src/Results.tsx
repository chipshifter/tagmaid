import { invoke } from "@tauri-apps/api/tauri";
import React, { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import "./Results.css";

function Result(props: { resultHash: string }) {
    const [resultHtml, setResultHtml] = useState(<></>);
    const result = useMemo(
        () => {
            invoke("get_result", { fileHash: props.resultHash })
                .then((res) => {
                    // Search OK
                    console.log(res)
                    setResultHtml(<button className="resultBlock">
                        <img src={res.image_path} width={"140"} height={"140"} />
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

export function ResultsTab(props: { searchResultHashes: string[] }) {
    const navigate = useNavigate();
    let results: string[] = props.searchResultHashes;
    console.log("wawa " + results);
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
        <div className="resultPage">
            {results.map((resultHash: string) => (<Result resultHash={resultHash} />))}
        </div>
    )
}