import { useState } from "react";
import {
  BrowserRouter as Router,
  Route,
  Link,
  Routes
} from "react-router-dom";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { SearchTab } from "./Search";

function App() {
  const [greetMsg, setGreetMsg] = useState("");

  async function searchig() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <Router>
      <nav className="tabs">
        <div className="tabs-buttons">
          <Link to="/">Search</Link>
          <Link to="/results">Results</Link>
          <Link to="/add">Add</Link>
        </div>
      </nav>
      <hr />
      <Routes>
        <Route path="/" element={
          <SearchTab />
        } />
        <Route path="/results" element={
          <h1>Results</h1>
        } />
        <Route path="/add" element={
          <h1>Add</h1>
        } />
      </Routes >
    </Router >
  );
}

export default App;
