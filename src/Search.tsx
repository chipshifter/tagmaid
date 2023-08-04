import { invoke } from "@tauri-apps/api/tauri";
import React, { useCallback, useState } from "react";

function SearchForm() {
    const [searchText, setSearchText] = useState("");
    const [loadingString, setLoadingString] = useState("");
    const submitSearch = useCallback(async () => {
        console.log("Hiii");
        setLoadingString("Searching...");
        await invoke("do_search", { query: searchText })
            .then((res) => { 
                // Search OK
                console.log(res)
                setLoadingString("Search success: " + res);
            })
            .catch((error) => {
                setLoadingString("Error: " + error);
                console.error(error)
            });
    }, [searchText, setLoadingString])

    return (
        <form onSubmit={(e) => {
            e.preventDefault()
            submitSearch();
        }}>
            <input type="text" value={searchText} onChange={(e) => setSearchText(e.target.value)} />
            <input type="submit" value="Submit" />
            <h1>{loadingString}</h1>
        </form>
    );
}

export function SearchTab() {
    return (
        <>
            <span>Hiii welcome to Search tab</span>
            <SearchForm />
        </>
    )
}