import { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import StarField from '../components/StarField';

export default function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isSuccess, setIsSuccess] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    const checkUser = async () => {
      try {
        const { supabase } = await import('../lib/supabase');
        // 检查 session 而不是 user，因为 session 更可靠
        const { data: { session } } = await supabase.auth.getSession();
        if (session?.user) {
          console.log('[Login] User already logged in, redirecting...');
          navigate('/tools');
        }
      } catch (err) {
        console.warn('[Login] Auth check failed:', err);
      }
    };
    checkUser();

    // 监听 auth state 变化，当登录成功时自动导航
    let authListener: { unsubscribe: () => void } | null = null;
    const setupAuthListener = async () => {
      try {
        const { supabase } = await import('../lib/supabase');
        const { data: { subscription } } = supabase.auth.onAuthStateChange((event, session) => {
          console.log('[Login] Auth state changed:', event, session ? 'has session' : 'no session');
          if (event === 'SIGNED_IN' && session?.user) {
            console.log('[Login] User signed in via listener, redirecting...');
            navigate('/tools');
          }
        });
        authListener = subscription;
      } catch (err) {
        console.warn('[Login] Failed to setup auth listener:', err);
      }
    };
    setupAuthListener();

    return () => {
      if (authListener) {
        authListener.unsubscribe();
      }
    };
  }, [navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const { supabase } = await import('../lib/supabase');
      const { data, error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error) {
        // 检查是否是邮箱未确认的错误
        if (error.message.includes('Email not confirmed') || error.message.includes('email_not_confirmed')) {
          // 即使 Supabase 返回这个错误，如果用户说邮箱已经确认了，可能是 session 缓存问题
          // 尝试重新获取 session
          const { data: sessionData } = await supabase.auth.getSession();
          if (sessionData?.session) {
            // 如果有 session，说明实际上已经登录了
            console.log('[Login] Session found after error, redirecting...');
            navigate('/tools');
            return;
          }
        }
        throw error;
      }

      // 登录成功，检查 session 和 user
      if (data?.session || data?.user) {
        console.log('[Login] Login successful, session:', data.session ? 'exists' : 'missing', 'user:', data.user ? 'exists' : 'missing');
        
        // 如果已经有 session，直接导航
        if (data.session) {
          console.log('[Login] Session available, redirecting immediately');
          navigate('/tools');
          return;
        }
        
        // 如果没有 session 但有 user，等待一下让 session 更新
        if (data.user) {
          console.log('[Login] User exists but no session, waiting for session update...');
          // 等待最多 1 秒让 session 更新
          for (let i = 0; i < 10; i++) {
            await new Promise(resolve => setTimeout(resolve, 100));
            const { data: { session } } = await supabase.auth.getSession();
            if (session) {
              console.log('[Login] Session found after wait, redirecting');
              navigate('/tools');
              return;
            }
          }
          
          // 如果等待后仍然没有 session，但 user 存在，也尝试导航
          // auth state change listener 应该会处理导航
          console.log('[Login] No session after wait, but user exists. Auth listener should handle navigation.');
          // 不抛出错误，让 auth state change listener 处理
        } else {
          throw new Error('登录失败：未收到有效的用户数据');
        }
      } else {
        throw new Error('登录失败：未收到有效的用户数据');
      }
    } catch (err: unknown) {
      let msg = err instanceof Error ? err.message : '登录失败';
      // 翻译常见登录错误
      if (msg.includes('Invalid login credentials') || msg.includes('invalid_credentials')) {
        msg = '邮箱或密码错误，请检查后重试。';
      } else if (msg.includes('Email not confirmed') || msg.includes('email_not_confirmed')) {
        msg = '请先确认您的邮箱（检查收件箱和垃圾邮件文件夹）。如果已确认，请尝试刷新页面后重新登录。';
      } else if (msg.includes('Too many requests') || msg.includes('too_many_requests')) {
        msg = '请求过于频繁，请稍后再试。';
      } else if (msg.includes('User not found')) {
        msg = '该邮箱未注册，请先注册账户。';
      }
      setError(msg);
      setIsSuccess(false);
      console.error('[Login] Login error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSignUp = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // 验证密码长度
    if (password.length < 6) {
      setError('密码至少需要 6 个字符');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const { supabase } = await import('../lib/supabase');
      const { data, error } = await supabase.auth.signUp({
        email,
        password,
      });

      if (error) {
        // 将常见错误翻译成中文
        let errorMsg = error.message;
        if (error.message.includes('Anonymous sign-ins are disabled')) {
          errorMsg = '匿名登录已禁用。请使用邮箱和密码注册。';
        } else if (error.message.includes('User already registered')) {
          errorMsg = '该邮箱已被注册，请直接登录。';
        } else if (error.message.includes('Invalid email')) {
          errorMsg = '邮箱格式不正确，请检查后重试。';
        } else if (error.message.includes('Password')) {
          errorMsg = '密码不符合要求，请使用至少 6 个字符。';
        } else if (error.message.includes('Email rate limit')) {
          errorMsg = '发送邮件过于频繁，请稍后再试。';
        }
        throw new Error(errorMsg);
      }

      // 注册成功
      if (data?.user) {
        const successMsg = '注册成功！请检查您的邮箱（包括垃圾邮件文件夹）以确认账户。';
        setError(successMsg);
        setIsSuccess(true);
        // 10秒后清除消息
        setTimeout(() => {
          setError(null);
          setIsSuccess(false);
        }, 10000);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '注册失败，请稍后重试';
      setError(msg);
      setIsSuccess(false);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#050508] text-white font-sans relative overflow-hidden flex flex-col items-center justify-center p-4">
      {/* Nebula Background */}
      <div className="nebula-container">
        <div className="nebula-layer nebula-1"></div>
        <div className="nebula-layer nebula-2"></div>
        <div className="nebula-layer nebula-3"></div>
      </div>

      {/* StarField Overlay */}
      <div className="fixed inset-0 z-0 opacity-60 mix-blend-screen pointer-events-none">
        <StarField />
      </div>

      {/* Back Button */}
      <Link 
        to="/" 
        className="absolute top-6 left-6 z-20 text-gray-400 hover:text-gold flex items-center gap-2 transition-colors text-xs uppercase tracking-widest"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
        返回首页
      </Link>

      {/* Login Card */}
      <div className="relative z-10 w-full max-w-md glass-panel border border-white/10 p-10 rounded-3xl shadow-2xl">
        <div className="text-center mb-10">
          {/* Icon with glow */}
          <div className="w-16 h-16 mx-auto mb-6 relative flex items-center justify-center">
            <div className="absolute inset-0 bg-gold/20 blur-xl rounded-full"></div>
            <svg className="w-10 h-10 text-gold relative z-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
            </svg>
          </div>
          
          <h2 className="text-3xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-b from-[#FCEda3] via-[#D4AF37] to-[#8a7020] tracking-widest mb-3">
            用户登录
          </h2>
          <div className="h-px w-24 mx-auto bg-gradient-to-r from-transparent via-gold/40 to-transparent mb-3"></div>
          <p className="text-gray-400 text-xs uppercase tracking-[0.2em]">Cosmic Chronology</p>
        </div>

        {error && (
          <div className={`mb-6 p-4 rounded-lg text-xs text-center backdrop-blur-sm ${
            isSuccess 
              ? 'bg-green-950/30 border border-green-500/30 text-green-200' 
              : 'bg-red-950/30 border border-red-500/30 text-red-200'
          }`}>
            {isSuccess ? '✓' : '⚠️'} {error}
          </div>
        )}

        <form className="space-y-6" onSubmit={handleLogin}>
          <div className="space-y-2">
            <label className="block text-gold/70 text-[10px] uppercase tracking-widest ml-1">邮箱 Email</label>
            <div className="relative group">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-gold transition-colors">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </div>
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className="w-full bg-black/30 border border-white/10 rounded-full py-3 pl-12 pr-4 text-gray-200 placeholder-gray-600 text-sm focus:border-gold/50 focus:ring-1 focus:ring-gold/50 focus:outline-none transition-all"
                placeholder="name@example.com"
              />
            </div>
          </div>
          
          <div className="space-y-2">
            <label className="block text-gold/70 text-[10px] uppercase tracking-widest ml-1">密码 Password</label>
            <div className="relative group">
              <div className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-gold transition-colors">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                className="w-full bg-black/30 border border-white/10 rounded-full py-3 pl-12 pr-4 text-gray-200 placeholder-gray-600 text-sm focus:border-gold/50 focus:ring-1 focus:ring-gold/50 focus:outline-none transition-all"
                placeholder="••••••••"
              />
            </div>
          </div>

          <div className="pt-4 space-y-4">
            <button
              type="submit"
              disabled={loading}
              className="w-full group flex items-center justify-center gap-2 bg-gold/10 hover:bg-gold/20 border border-gold/30 hover:border-gold/60 backdrop-blur-md rounded-full px-5 py-3.5 transition-all duration-300 shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span className="text-sm font-bold uppercase tracking-wider text-gold group-hover:text-white transition-colors">
                {loading ? '登录中...' : '登 录'}
              </span>
            </button>
            
            <div className="relative py-3">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/10"></div>
              </div>
              <div className="relative flex justify-center">
                <span className="px-4 bg-[#050508] text-[10px] text-gray-500 uppercase tracking-widest">或者</span>
              </div>
            </div>

            <button
              type="button"
              onClick={handleSignUp}
              disabled={loading}
              className="w-full group flex items-center justify-center gap-2 bg-black/30 hover:bg-white/5 border border-white/10 hover:border-white/30 backdrop-blur-md rounded-full px-5 py-3 transition-all duration-300 shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span className="text-sm font-medium uppercase tracking-wider text-gray-400 group-hover:text-white transition-colors">
                注册新账户
              </span>
            </button>
          </div>
        </form>
      </div>
      
      <div className="absolute bottom-6 text-[10px] text-gray-600 z-10 uppercase tracking-widest">
        © {new Date().getFullYear()} Huangji Jingshi Platform
      </div>
    </div>
  );
}
