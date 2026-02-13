import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import jwt_decode from 'jwt-decode';
import { authApi } from '../api';
import { LoginRequest, User } from '../types';

interface AuthContextType {
  user: User | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  login: (credentials: LoginRequest) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Check if there's a saved token and decode it to get the user
  useEffect(() => {
    const initAuth = async () => {
      const token = authApi.getToken();

      if (token) {
        try {
          // Decode the JWT token to get the user data
          const decodedToken = jwt_decode<{ custom: User }>(token);
          setUser(decodedToken.custom);
        } catch (error) {
          console.error('Invalid token', error);
          authApi.removeToken();
        }
      }
      
      setIsLoading(false);
    };

    initAuth();
  }, []);

  // Login function
  const login = async (credentials: LoginRequest) => {
    setIsLoading(true);
    try {
      const response = await authApi.login(credentials);
      authApi.setToken(response.token);
      
      // Decode token to get user data
      const decodedToken = jwt_decode<{ custom: User }>(response.token);
      setUser(decodedToken.custom);
    } catch (error) {
      console.error('Login failed', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  // Logout function
  const logout = () => {
    authApi.removeToken();
    setUser(null);
  };

  const isAuthenticated = !!user;

  const value = {
    user,
    isLoading,
    isAuthenticated,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

// Custom hook to use the auth context
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}