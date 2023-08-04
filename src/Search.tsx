import { invoke } from "@tauri-apps/api/tauri";
import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";
import "./Search.css";

export function SearchTab({ searchState, updateSearch }) {
    const [searchText, setSearchText] = useState(searchState.searchQuery);
    const [errorString, setErrorString] = useState("");
    // When search is done, redirect to results page
    const navigate = useNavigate();

    const submitSearch = useCallback(async () => {
        searchState.setSearchQuery(searchText);
        await invoke("do_search", { query: searchText })
            .then((res) => {
                // Search OK
                console.log(res)
                updateSearch(res)
                navigate("/results");
            })
            .catch((error) => {
                setErrorString("Error: " + error)
                console.error(error)
            });
    }, [searchText, errorString, setErrorString, navigate, updateSearch])

    return (
        <div className="searchPage">
            <h1>Search</h1>
            <form onSubmit={(e) => {
                e.preventDefault()
                submitSearch();
            }}>
                <input type="text" className="searchField" autoFocus={true} value={searchText} onChange={(e) => setSearchText(e.target.value)} />
                <input type="submit" className="searchButton" value="Submit" />
            </form>
            <br />
            { errorString.length > 0 ? <h3 className="errorString">{errorString}</h3> : null}
        </div>
    )
}