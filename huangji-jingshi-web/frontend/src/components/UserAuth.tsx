import { useState, useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';
import { ChevronDown } from 'lucide-react';

// 用户接口
interface UserData {
  email: string;
  id: string;
  avatar_url?: string;
}

export default function UserAuth() {
  const [user, setUser] = useState<UserData | null>(null);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 安全地检查 Supabase 状态
    const checkAuth = async () => {
      try {
        const { supabase } = await import('../lib/supabase');
        const { data, error } = await supabase.auth.getSession();
        
        if (error) {
          console.warn('[UserAuth] Supabase error:', error.message);
          setError(true);
          setLoading(false);
          return;
        }

        if (data?.session?.user) {
          const avatarUrl = data.session.user.user_metadata?.avatar_url || null;
          setUser({
            email: data.session.user.email || 'user',
            id: data.session.user.id,
            avatar_url: avatarUrl,
          });
        }
        
        // 监听认证状态变化
        const { data: { subscription } } = supabase.auth.onAuthStateChange((_event, session) => {
          if (session?.user) {
            const avatarUrl = session.user.user_metadata?.avatar_url || null;
            setUser({
              email: session.user.email || 'user',
              id: session.user.id,
              avatar_url: avatarUrl,
            });
          } else {
            setUser(null);
          }
        });

        setLoading(false);
        
        return () => {
          try {
            subscription.unsubscribe();
          } catch (e) {
            console.warn('[UserAuth] Cleanup error:', e);
          }
        };
      } catch (err) {
        console.warn('[UserAuth] Failed to initialize:', err);
        setError(true);
        setLoading(false);
      }
    };

    checkAuth();
  }, []);

  // 关闭下拉菜单
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleLogout = async () => {
    try {
      const { supabase } = await import('../lib/supabase');
      await supabase.auth.signOut();
      setUser(null);
      setIsOpen(false);
    } catch (err) {
      console.error('[UserAuth] Logout error:', err);
    }
  };

  // 如果初始化失败，不显示组件
  if (error) {
    return null;
  }

  // 加载中
  if (loading) {
    return (
      <div className="fixed top-6 right-6 z-50">
        <div className="w-8 h-8 rounded-full bg-white/5 animate-pulse"></div>
      </div>
    );
  }

  return (
    <div className="fixed top-6 right-6 z-50" ref={menuRef}>
      {user ? (
        <div className="relative">
          <button
            onClick={() => setIsOpen(!isOpen)}
            className="flex items-center gap-3 bg-black/20 hover:bg-white/10 border border-white/10 hover:border-gold/30 backdrop-blur-md rounded-full pl-2 pr-4 py-1.5 transition-all duration-300 group"
          >
            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-gold/20 to-transparent border border-gold/30 flex items-center justify-center text-gold group-hover:text-white transition-colors overflow-hidden">
              {user.avatar_url ? (
                <img 
                  src={user.avatar_url} 
                  alt="Avatar" 
                  className="w-full h-full object-cover"
                />
              ) : (
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
              )}
            </div>
            <span className="text-xs font-medium text-gray-300 group-hover:text-gold transition-colors max-w-[100px] truncate">
              {user.email.split('@')[0]}
            </span>
            <ChevronDown 
              size={14} 
              className={`text-gray-500 transition-transform duration-300 ${isOpen ? 'rotate-180' : ''}`} 
            />
          </button>

          {/* 下拉菜单 */}
          {isOpen && (
            <div className="absolute right-0 mt-2 w-48 bg-[#0a0a0c]/95 border border-white/10 rounded-xl shadow-2xl backdrop-blur-xl overflow-hidden">
              <div className="p-3 border-b border-white/5">
                <p className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">已登录</p>
                <p className="text-xs text-white truncate font-mono">{user.email}</p>
              </div>
              <div className="p-1">
                <Link 
                  to="/tools" 
                  className="flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
                  onClick={() => setIsOpen(false)}
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                  </svg>
                  推演工具
                </Link>
                <Link 
                  to="/settings" 
                  className="flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
                  onClick={() => setIsOpen(false)}
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  设置
                </Link>
                <div className="h-px bg-white/5 my-1"></div>
                <button
                  onClick={handleLogout}
                  className="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-colors text-left"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                  </svg>
                  登出
                </button>
              </div>
            </div>
          )}
        </div>
      ) : (
        <Link
          to="/login"
          className="group flex items-center gap-2 bg-black/30 hover:bg-gold/10 border border-white/10 hover:border-gold/50 backdrop-blur-md rounded-full px-5 py-2.5 transition-all duration-300 shadow-lg"
        >
          <div className="w-2 h-2 rounded-full bg-gold/50 group-hover:bg-gold transition-all group-hover:shadow-[0_0_8px_#D4AF37]"></div>
          <span className="text-xs font-bold text-gray-300 group-hover:text-gold uppercase tracking-wider transition-colors">
            登录 / 注册
          </span>
        </Link>
      )}
    </div>
  );
}
