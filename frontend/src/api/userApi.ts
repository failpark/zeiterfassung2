import { CreateUser, PaginationResult, UpdateUser, User } from '../types';
import { apiRequest } from './axiosClient';

export const userApi = {
  // Get a single user by ID
  getUser: async (id: number): Promise<User> => {
    return await apiRequest<User>({
      method: 'GET',
      url: `/user/${id}`,
    });
  },

  // Get paginated users
  getUsers: async (page: number, pageSize: number): Promise<PaginationResult<User>> => {
    return await apiRequest<PaginationResult<User>>({
      method: 'GET',
      url: `/user/page/${pageSize}/${page}`,
    });
  },

  // Get the last page of users
  getLastPage: async (pageSize: number): Promise<PaginationResult<User>> => {
    return await apiRequest<PaginationResult<User>>({
      method: 'GET',
      url: `/user/page/${pageSize}/last`,
    });
  },

  // Create a new user
  createUser: async (user: CreateUser): Promise<User> => {
    return await apiRequest<User>({
      method: 'POST',
      url: '/user',
      data: user,
    });
  },

  // Update an existing user
  updateUser: async (id: number, user: UpdateUser): Promise<User> => {
    return await apiRequest<User>({
      method: 'PATCH',
      url: `/user/${id}`,
      data: user,
    });
  },

  // Delete a user
  deleteUser: async (id: number): Promise<void> => {
    return await apiRequest<void>({
      method: 'DELETE',
      url: `/user/${id}`,
    });
  },
};

export default userApi;