import { useState } from "react";
import {
  BrowserRouter as Router,
  Route,
  Link,
  Routes
} from "react-router-dom";
import "./App.css";
import { SearchTab } from "./Search";
import { ResultsTab } from "./Results";
function App() {
  const [searchQuery, setSearchQuery] = useState([]);
  const [searchResultHashes, setSearchResultHashes] = useState([]);

  return (
    <Router>
      <nav className="page tabs">
        <div className="tabs-buttons">
          <Link to="/">Search</Link>
          <Link to="/results">Results</Link>
          <Link to="/add">Add</Link>
        </div>
      </nav>
      <hr />
      <div className="page mainPage">
        <Routes>
          <Route path="/" element={
            <SearchTab searchState={{ searchQuery, setSearchQuery }} updateSearch={setSearchResultHashes} />
          } />
          <Route path="/results" element={
            <ResultsTab query={searchQuery} searchResultHashes={searchResultHashes} />
          } />
          <Route path="/add" element={
            <h1>Add</h1>
          } />
        </Routes >
      </div>
    </Router >
  );
}

export default App;
