import { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import StarField from '../components/StarField';

interface UserData {
  email: string;
  id: string;
  created_at?: string;
  avatar_url?: string;
}

export default function Settings() {
  const [user, setUser] = useState<UserData | null>(null);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [avatarPreview, setAvatarPreview] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const { supabase } = await import('../lib/supabase');
        const { data: { session }, error } = await supabase.auth.getSession();
        
        if (error) throw error;
        
        if (!session?.user) {
          navigate('/login');
          return;
        }

        // 获取用户元数据中的头像 URL
        const avatarUrl = session.user.user_metadata?.avatar_url || null;
        
        setUser({
          email: session.user.email || '',
          id: session.user.id,
          created_at: session.user.created_at,
          avatar_url: avatarUrl,
        });
        
        if (avatarUrl) {
          setAvatarPreview(avatarUrl);
        }
      } catch (err) {
        console.error('[Settings] Auth check failed:', err);
        setError('无法加载用户信息');
      } finally {
        setLoading(false);
      }
    };

    checkAuth();
  }, [navigate]);

  const handlePasswordReset = async () => {
    if (!user?.email) return;
    
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const { supabase } = await import('../lib/supabase');
      const { error } = await supabase.auth.resetPasswordForEmail(user.email, {
        redirectTo: `${window.location.origin}/login`,
      });

      if (error) throw error;
      setSuccess('密码重置邮件已发送，请检查您的邮箱。');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '发送密码重置邮件失败';
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file || !user) return;

    // 验证文件类型
    if (!file.type.startsWith('image/')) {
      setError('请选择图片文件');
      return;
    }

    // 验证文件大小 (最大 5MB)
    if (file.size > 5 * 1024 * 1024) {
      setError('图片大小不能超过 5MB');
      return;
    }

    setUploading(true);
    setError(null);
    setSuccess(null);

    try {
      const { supabase } = await import('../lib/supabase');
      
      // 创建预览
      const reader = new FileReader();
      reader.onload = (e) => {
        setAvatarPreview(e.target?.result as string);
      };
      reader.readAsDataURL(file);

      // 上传到 Supabase Storage
      const fileExt = file.name.split('.').pop();
      const fileName = `${user.id}-${Date.now()}.${fileExt}`;
      const filePath = `avatars/${fileName}`;

      // 先尝试删除旧头像
      if (user.avatar_url) {
        try {
          // 从完整 URL 中提取路径
          const urlParts = user.avatar_url.split('/');
          const fileNameIndex = urlParts.findIndex(part => part === 'avatars');
          if (fileNameIndex !== -1 && fileNameIndex < urlParts.length - 1) {
            const fileName = urlParts.slice(fileNameIndex + 1).join('/');
            await supabase.storage.from('avatars').remove([fileName]);
          }
        } catch (err) {
          console.warn('[Settings] Failed to remove old avatar:', err);
          // 继续上传新头像，即使删除旧头像失败
        }
      }

      const { error: uploadError } = await supabase.storage
        .from('avatars')
        .upload(filePath, file, {
          cacheControl: '3600',
          upsert: false,
        });

      if (uploadError) {
        // 友好的错误提示
        if (uploadError.message.includes('Bucket not found') || uploadError.message.includes('not found')) {
          throw new Error('存储空间未配置');
        }
        if (uploadError.message.includes('new row violates row-level security') || uploadError.message.includes('permission denied')) {
          throw new Error('权限不足。请检查 Supabase Storage 的权限策略设置。');
        }
        throw uploadError;
      }

      // 获取公共 URL
      const { data: urlData } = supabase.storage
        .from('avatars')
        .getPublicUrl(filePath);

      const avatarUrl = urlData.publicUrl;

      // 更新用户元数据
      const { error: updateError } = await supabase.auth.updateUser({
        data: { avatar_url: avatarUrl },
      });

      if (updateError) throw updateError;

      // 更新本地状态
      setUser({
        ...user,
        avatar_url: avatarUrl,
      });

      setSuccess('头像上传成功！');
      
      // 3秒后清除成功消息
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      console.error('[Settings] Avatar upload failed:', err);
      let msg = err instanceof Error ? err.message : '头像上传失败';
      
      // 翻译常见错误
      if (msg.includes('Bucket not found') || msg.includes('not found')) {
        msg = '存储空间未配置。请在 Supabase Dashboard 的 Storage 中创建名为 "avatars" 的 bucket，并设置为公开可读。';
      } else if (msg.includes('new row violates row-level security')) {
        msg = '权限不足。请检查 Supabase Storage 的权限设置。';
      } else if (msg.includes('JWT')) {
        msg = '认证失败。请重新登录后重试。';
      }
      
      setError(msg);
      setAvatarPreview(user.avatar_url || null);
    } finally {
      setUploading(false);
    }
  };

  const handleRemoveAvatar = async () => {
    if (!user?.avatar_url) return;

    setUploading(true);
    setError(null);
    setSuccess(null);

    try {
      const { supabase } = await import('../lib/supabase');
      
      // 从 Storage 删除头像
      try {
        const urlParts = user.avatar_url.split('/');
        const fileNameIndex = urlParts.findIndex(part => part === 'avatars');
        if (fileNameIndex !== -1 && fileNameIndex < urlParts.length - 1) {
          const fileName = urlParts.slice(fileNameIndex + 1).join('/');
          await supabase.storage.from('avatars').remove([fileName]);
        }
      } catch (err) {
        console.warn('[Settings] Failed to remove avatar from storage:', err);
        // 如果 bucket 不存在，只更新元数据即可
        const errorMsg = err instanceof Error ? err.message : '';
        if (!errorMsg.includes('Bucket not found') && !errorMsg.includes('not found')) {
          // 只有非 bucket 错误才记录警告
        }
        // 继续更新用户元数据，即使删除文件失败
      }

      // 更新用户元数据
      const { error: updateError } = await supabase.auth.updateUser({
        data: { avatar_url: null },
      });

      if (updateError) throw updateError;

      // 更新本地状态
      setUser({
        ...user,
        avatar_url: undefined,
      });

      setAvatarPreview(null);
      setSuccess('头像已删除');
      
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      console.error('[Settings] Remove avatar failed:', err);
      const msg = err instanceof Error ? err.message : '删除头像失败';
      
      // 如果 bucket 不存在，只更新元数据即可，不算错误
      if (msg.includes('Bucket not found') || msg.includes('not found')) {
        // 这种情况下，元数据已经更新，可以忽略存储错误
        setSuccess('头像已删除');
        setTimeout(() => setSuccess(null), 3000);
        return;
      }
      
      setError(msg);
    } finally {
      setUploading(false);
    }
  };

  const handleLogout = async () => {
    try {
      const { supabase } = await import('../lib/supabase');
      await supabase.auth.signOut();
      navigate('/');
    } catch (err) {
      console.error('[Settings] Logout failed:', err);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[#050508] text-white font-sans relative overflow-hidden flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-gold/30 border-t-gold rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-gray-400 text-sm">加载中...</p>
        </div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  return (
    <div className="min-h-screen bg-[#050508] text-white font-sans relative overflow-hidden">
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

      {/* Main Content */}
      <main className="relative z-10 container mx-auto px-4 py-20 max-w-4xl">
        <div className="mb-12 text-center">
          <div className="w-20 h-20 mx-auto mb-6 relative flex items-center justify-center">
            <div className="absolute inset-0 bg-gold/20 blur-xl rounded-full"></div>
            <svg className="w-12 h-12 text-gold relative z-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <h1 className="text-4xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-b from-[#FCEda3] via-[#D4AF37] to-[#8a7020] tracking-widest mb-3">
            账户设置
          </h1>
          <div className="h-px w-24 mx-auto bg-gradient-to-r from-transparent via-gold/40 to-transparent mb-3"></div>
          <p className="text-gray-400 text-xs uppercase tracking-[0.2em]">Account Settings</p>
        </div>

        {/* Messages */}
        {error && (
          <div className="mb-6 p-4 bg-red-950/30 border border-red-500/30 text-red-200 text-sm rounded-lg text-center backdrop-blur-sm">
            ⚠️ {error}
          </div>
        )}

        {success && (
          <div className="mb-6 p-4 bg-green-950/30 border border-green-500/30 text-green-200 text-sm rounded-lg text-center backdrop-blur-sm">
            ✓ {success}
          </div>
        )}

        {/* Avatar Section */}
        <div className="glass-panel border border-white/10 p-8 rounded-3xl mb-6">
          <h2 className="text-xl font-serif text-gold mb-6 border-l-2 border-gold/50 pl-3">头像设置</h2>
          
          <div className="flex flex-col items-center space-y-6">
            {/* Avatar Preview */}
            <div className="relative">
              <div className="w-32 h-32 rounded-full bg-gradient-to-br from-gold/20 to-transparent border-2 border-gold/30 flex items-center justify-center overflow-hidden">
                {avatarPreview ? (
                  <img 
                    src={avatarPreview} 
                    alt="Avatar" 
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <svg className="w-16 h-16 text-gold/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                  </svg>
                )}
              </div>
              {uploading && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/50 rounded-full">
                  <div className="w-8 h-8 border-2 border-gold border-t-transparent rounded-full animate-spin"></div>
                </div>
              )}
            </div>

            {/* Upload Buttons */}
            <div className="flex gap-4 w-full max-w-md">
              <label className="flex-1 cursor-pointer">
                <input
                  type="file"
                  accept="image/*"
                  onChange={handleAvatarUpload}
                  disabled={uploading}
                  className="hidden"
                />
                <div className="w-full group flex items-center justify-center gap-2 bg-gold/10 hover:bg-gold/20 border border-gold/30 hover:border-gold/60 backdrop-blur-md rounded-full px-5 py-3 transition-all duration-300 shadow-lg disabled:opacity-50 disabled:cursor-not-allowed">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  <span className="text-sm font-bold uppercase tracking-wider text-gold group-hover:text-white transition-colors">
                    {uploading ? '上传中...' : '上传头像'}
                  </span>
                </div>
              </label>

              {user.avatar_url && (
                <button
                  onClick={handleRemoveAvatar}
                  disabled={uploading}
                  className="flex-1 group flex items-center justify-center gap-2 bg-black/30 hover:bg-red-500/10 border border-white/10 hover:border-red-500/30 backdrop-blur-md rounded-full px-5 py-3 transition-all duration-300 shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                  <span className="text-sm font-medium uppercase tracking-wider text-red-400 group-hover:text-red-300 transition-colors">
                    删除
                  </span>
                </button>
              )}
            </div>

            <p className="text-xs text-gray-500 text-center">
              支持 JPG、PNG、GIF 格式，最大 5MB
            </p>
          </div>
        </div>

        {/* Account Information */}
        <div className="glass-panel border border-white/10 p-8 rounded-3xl mb-6">
          <h2 className="text-xl font-serif text-gold mb-6 border-l-2 border-gold/50 pl-3">账户信息</h2>
          
          <div className="space-y-4">
            <div>
              <label className="block text-gold/70 text-[10px] uppercase tracking-widest mb-2">邮箱 Email</label>
              <div className="bg-black/30 border border-white/10 rounded-lg px-4 py-3 text-gray-200 text-sm">
                {user.email}
              </div>
            </div>

            <div>
              <label className="block text-gold/70 text-[10px] uppercase tracking-widest mb-2">用户 ID</label>
              <div className="bg-black/30 border border-white/10 rounded-lg px-4 py-3 text-gray-400 text-xs font-mono break-all">
                {user.id}
              </div>
            </div>

            {user.created_at && (
              <div>
                <label className="block text-gold/70 text-[10px] uppercase tracking-widest mb-2">注册时间</label>
                <div className="bg-black/30 border border-white/10 rounded-lg px-4 py-3 text-gray-400 text-xs">
                  {new Date(user.created_at).toLocaleString('zh-CN')}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Account Actions */}
        <div className="glass-panel border border-white/10 p-8 rounded-3xl mb-6">
          <h2 className="text-xl font-serif text-gold mb-6 border-l-2 border-gold/50 pl-3">账户操作</h2>
          
          <div className="space-y-4">
            <button
              onClick={handlePasswordReset}
              disabled={loading}
              className="w-full group flex items-center justify-center gap-2 bg-gold/10 hover:bg-gold/20 border border-gold/30 hover:border-gold/60 backdrop-blur-md rounded-full px-5 py-3 transition-all duration-300 shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
              <span className="text-sm font-bold uppercase tracking-wider text-gold group-hover:text-white transition-colors">
                重置密码
              </span>
            </button>

            <div className="h-px bg-white/5 my-4"></div>

            <button
              onClick={handleLogout}
              className="w-full group flex items-center justify-center gap-2 bg-black/30 hover:bg-red-500/10 border border-white/10 hover:border-red-500/30 backdrop-blur-md rounded-full px-5 py-3 transition-all duration-300 shadow-lg"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
              <span className="text-sm font-medium uppercase tracking-wider text-red-400 group-hover:text-red-300 transition-colors">
                登出账户
              </span>
            </button>
          </div>
        </div>

        {/* App Settings */}
        <div className="glass-panel border border-white/10 p-8 rounded-3xl">
          <h2 className="text-xl font-serif text-gold mb-6 border-l-2 border-gold/50 pl-3">应用设置</h2>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-black/20 rounded-lg border border-white/5">
              <div>
                <p className="text-sm text-gray-200 mb-1">数据同步</p>
                <p className="text-xs text-gray-500">自动保存您的推演记录到云端</p>
              </div>
              <div className="w-12 h-6 bg-gold/20 rounded-full relative cursor-pointer">
                <div className="w-5 h-5 bg-gold rounded-full absolute top-0.5 left-0.5 transition-transform"></div>
              </div>
            </div>

            <div className="flex items-center justify-between p-4 bg-black/20 rounded-lg border border-white/5">
              <div>
                <p className="text-sm text-gray-200 mb-1">通知提醒</p>
                <p className="text-xs text-gray-500">接收重要更新和系统通知</p>
              </div>
              <div className="w-12 h-6 bg-gray-700 rounded-full relative cursor-pointer">
                <div className="w-5 h-5 bg-gray-400 rounded-full absolute top-0.5 left-0.5 transition-transform"></div>
              </div>
            </div>
          </div>
        </div>
      </main>

      <footer className="relative z-10 py-8 text-center text-gray-600 text-xs">
        <p>© {new Date().getFullYear()} Huangji Jingshi Platform</p>
      </footer>
    </div>
  );
}

