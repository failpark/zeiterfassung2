import { Client, CreateClient, PaginationResult, UpdateClient } from '../types';
import { apiRequest } from './axiosClient';

export const clientApi = {
  // Get a single client by ID
  getClient: async (id: number): Promise<Client> => {
    return await apiRequest<Client>({
      method: 'GET',
      url: `/client/${id}`,
    });
  },

  // Get paginated clients
  getClients: async (page: number, pageSize: number): Promise<PaginationResult<Client>> => {
    return await apiRequest<PaginationResult<Client>>({
      method: 'GET',
      url: `/client/page/${pageSize}/${page}`,
    });
  },

  // Get the last page of clients
  getLastPage: async (pageSize: number): Promise<PaginationResult<Client>> => {
    return await apiRequest<PaginationResult<Client>>({
      method: 'GET',
      url: `/client/page/${pageSize}/last`,
    });
  },

  // Create a new client
  createClient: async (client: CreateClient): Promise<Client> => {
    return await apiRequest<Client>({
      method: 'POST',
      url: '/client',
      data: client,
    });
  },

  // Update an existing client
  updateClient: async (id: number, client: UpdateClient): Promise<Client> => {
    return await apiRequest<Client>({
      method: 'PATCH',
      url: `/client/${id}`,
      data: client,
    });
  },

  // Delete a client
  deleteClient: async (id: number): Promise<void> => {
    return await apiRequest<void>({
      method: 'DELETE',
      url: `/client/${id}`,
    });
  },
};

export default clientApi;