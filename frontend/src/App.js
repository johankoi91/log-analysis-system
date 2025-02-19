import React from 'react';
import LogSearch from './components/LogSearch';
import FilterForUpload from './components/FilterForUpload';
import ContextDisplay from "./components/ContextDisplay";
import './App.css';

function App() {
    return (
        <div>
            <h1>Log Search System</h1>
            <FilterForUpload />
            <LogSearch />
        </div>
    );
}

export default App;
