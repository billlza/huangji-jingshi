import { createClient } from '@supabase/supabase-js';

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL;
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY;

// 最佳实践：在缺少配置时提供 Mock 客户端，防止应用崩溃
const createSafeClient = () => {
  if (!supabaseUrl || !supabaseAnonKey) {
    console.warn('⚠️ Supabase environment variables missing. Running in offline/mock mode.');
    
    // 返回一个最小化的 Mock 对象，模拟 supabase.auth 的行为
    return {
      auth: {
        getSession: async () => ({ data: { session: null }, error: null }),
        getUser: async () => ({ data: { user: null }, error: null }),
        onAuthStateChange: () => ({ data: { subscription: { unsubscribe: () => {} } } }),
        signInWithPassword: async () => ({ data: null, error: { message: '后端服务未配置 (Supabase not configured)' } }),
        signUp: async () => ({ data: null, error: { message: '后端服务未配置 (Supabase not configured)' } }),
        signOut: async () => ({ error: null }),
        updateUser: async () => ({ data: { user: null }, error: { message: 'Supabase not configured' } }),
        resetPasswordForEmail: async () => ({ error: { message: 'Supabase not configured' } }),
      },
      storage: {
        from: () => ({
          upload: async () => ({ error: { message: 'Supabase not configured' } }),
          remove: async () => ({ error: { message: 'Supabase not configured' } }),
          getPublicUrl: () => ({ data: { publicUrl: '' } }),
        }),
      },
      from: () => ({
        select: () => ({
          order: () => ({ data: [], error: null }), // Mock 数据查询
        }),
        // 可以根据需要添加更多 Mock
      })
    } as unknown as ReturnType<typeof createClient>;
  }
  
  return createClient(supabaseUrl, supabaseAnonKey);
};

export const supabase = createSafeClient();