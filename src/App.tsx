import { useState } from "react";
import {
  BrowserRouter as Router,
  Route,
  Link,
  Routes,
  useNavigate
} from "react-router-dom";
import "./App.css";
import { SearchForm, SearchTab } from "./Search";
import { ResultsTab } from "./Results";
import React from "react";

function App() {
  const [searchQuery, setSearchQuery] = useState([]);
  const [searchResultHashes, setSearchResultHashes] = useState([]);

  return (
    <Router>
      <Routes>
        <Route path="/" element={
          <>
            <nav className="page tabs">
              <div className="tabs-buttons">
                <Link to="/results">Results</Link>
                <Link to="/add">Add</Link>
              </div>
            </nav>
            <hr />
            <div className="page mainPage">
              <SearchTab searchState={{ searchQuery, setSearchQuery }} updateSearch={setSearchResultHashes} />
            </div>
          </>
        } />
        <Route path="/results" element={
          <>
            <nav className="page tabs">
              <SearchForm searchState={{ searchQuery, setSearchQuery }} updateSearch={setSearchResultHashes} setErrorString={undefined} />
            </nav>
            <hr />
            <div className="page mainPage">
              <ResultsTab query={searchQuery} searchResultHashes={searchResultHashes} />
            </div>
          </>
        } />
        <Route path="/add" element={
          <h1>Add</h1>
        } />
      </Routes >
    </Router >
  );
}

export default App;
