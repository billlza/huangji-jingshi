
import { useState, useEffect } from 'react';
import { supabase } from '../lib/supabase';
import { useNavigate } from 'react-router-dom';
import type { User } from '@supabase/supabase-js';

export default function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    const checkUser = async () => {
      const { data: { user } } = await supabase.auth.getUser();
      if (user) {
        setUser(user);
        navigate('/tools');
      }
    };
    checkUser();
  }, [navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const { error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error) throw error;
      navigate('/tools');
    } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : 'Login failed';
        setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleSignUp = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const { error } = await supabase.auth.signUp({
        email,
        password,
      });

      if (error) throw error;
      alert('Check your email for the login link!');
    } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : 'Signup failed';
        setError(msg);
    } finally {
      setLoading(false);
    }
  };

  if (user) return null;

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-950 p-4">
      <div className="w-full max-w-md bg-white/5 border border-white/10 p-8 rounded-xl shadow-2xl backdrop-blur-sm">
        <div className="text-center mb-8">
          <div className="w-12 h-12 bg-gold mx-auto rotate-45 mb-6 shadow-[0_0_15px_#D4AF37]"></div>
          <h2 className="text-2xl font-serif text-gold tracking-widest">用户登录</h2>
          <p className="text-gray-400 text-sm mt-2">皇极经世 · 登录以保存您的推演记录</p>
        </div>

        {error && (
          <div className="mb-6 p-3 bg-red-500/20 border border-red-500/50 text-red-200 text-sm rounded text-center">
            {error}
          </div>
        )}

        <form className="space-y-6">
          <div>
            <label className="block text-gray-400 text-xs uppercase tracking-widest mb-2">Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full bg-black/20 border border-white/10 rounded px-4 py-3 text-gray-200 focus:border-gold focus:outline-none transition-colors"
              placeholder="your@email.com"
            />
          </div>
          <div>
            <label className="block text-gray-400 text-xs uppercase tracking-widest mb-2">Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full bg-black/20 border border-white/10 rounded px-4 py-3 text-gray-200 focus:border-gold focus:outline-none transition-colors"
              placeholder="••••••••"
            />
          </div>

          <div className="flex gap-4 pt-4">
            <button
              onClick={handleLogin}
              disabled={loading}
              className="flex-1 bg-gold/90 hover:bg-gold text-black font-bold py-3 rounded transition-all transform hover:scale-[1.02] disabled:opacity-50"
            >
              {loading ? '处理中...' : '登 录'}
            </button>
            <button
              onClick={handleSignUp}
              disabled={loading}
              className="flex-1 bg-white/5 hover:bg-white/10 text-gold border border-gold/30 py-3 rounded transition-all disabled:opacity-50"
            >
              注 册
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
