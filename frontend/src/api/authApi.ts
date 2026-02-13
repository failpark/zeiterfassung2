import { LoginRequest, LoginResponse } from '../types';
import { apiRequest } from './axiosClient';

export const authApi = {
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    return await apiRequest<LoginResponse>({
      method: 'POST',
      url: '/login',
      data: credentials,
    });
  },

  // Store token in localStorage
  setToken: (token: string): void => {
    localStorage.setItem('auth_token', token);
  },

  // Get token from localStorage
  getToken: (): string | null => {
    return localStorage.getItem('auth_token');
  },

  // Remove token from localStorage
  removeToken: (): void => {
    localStorage.removeItem('auth_token');
  },

  // Check if user is authenticated
  isAuthenticated: (): boolean => {
    const token = localStorage.getItem('auth_token');
    return !!token;
  },
};

export default authApi;