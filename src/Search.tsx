import { invoke } from "@tauri-apps/api/tauri";
import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";
import "./Search.css";

export function SearchForm({ searchState, updateSearch, setErrorString }) {
    const [searchText, setSearchText] = useState(searchState.searchQuery);
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
    }, [searchText, setErrorString, navigate, updateSearch])
    return (<form onSubmit={(e) => {
        e.preventDefault()
        submitSearch();
    }}>
        <input type="text" className="searchField" autoFocus={true} value={searchText} onChange={(e) => setSearchText(e.target.value)} />
        <input type="submit" className="searchButton" value="Search" />
    </form>);
}

export function SearchTab({ searchState, updateSearch }) {
    const [errorString, setErrorString] = useState("");

    return (
        <div className="searchPage">
            <h1>Search</h1>
            <SearchForm searchState={searchState} updateSearch={updateSearch} setErrorString={setErrorString} />
            <br />
            {errorString.length > 0 ? <h3 className="errorString">{errorString}</h3> : null}
        </div>
    )
}