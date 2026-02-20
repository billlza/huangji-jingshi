import { BrowserRouter, Route, Routes } from 'react-router-dom';
import UserAuth from './components/UserAuth';
import About from './pages/About';
import Home from './pages/Home';
import Login from './pages/Login';
import Settings from './pages/Settings';
import Story from './pages/Story';
import Tools from './pages/Tools';
import './index.css';

function App() {
  return (
    <BrowserRouter>
      <UserAuth />
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/tools" element={<Tools />} />
        <Route path="/story" element={<Story />} />
        <Route path="/login" element={<Login />} />
        <Route path="/about" element={<About />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
