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

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [selectedTab, setSelectedTab] = useState("Search");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <Router>
      <nav className="tabs">
        <div className="tabs-buttons">
          <Link to="/">Search</Link>
          <Link className="link" to="/results">Results</Link>
          <Link className="link" to="/add">Add</Link>
        </div>
      </nav>
      <hr />
      <Routes>
        <Route path="/" element={
          <h1>Search</h1>
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
