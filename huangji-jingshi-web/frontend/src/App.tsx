
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Home from './pages/Home';
import Tools from './pages/Tools';
import Story from './pages/Story';
import Login from './pages/Login';
import About from './pages/About';
import Settings from './pages/Settings';
import UserAuth from './components/UserAuth';
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
  )
}

export default App;
