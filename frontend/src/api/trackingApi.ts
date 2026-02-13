import { CreateTracking, PaginationResult, Tracking, UpdateTracking } from '../types';
import { apiRequest } from './axiosClient';

export const trackingApi = {
  // Get a single tracking by ID
  getTracking: async (id: number): Promise<Tracking> => {
    return await apiRequest<Tracking>({
      method: 'GET',
      url: `/tracking/${id}`,
    });
  },

  // Get paginated trackings
  getTrackings: async (page: number, pageSize: number): Promise<PaginationResult<Tracking>> => {
    return await apiRequest<PaginationResult<Tracking>>({
      method: 'GET',
      url: `/tracking/page/${pageSize}/${page}`,
    });
  },

  // Get the last page of trackings
  getLastPage: async (pageSize: number): Promise<PaginationResult<Tracking>> => {
    return await apiRequest<PaginationResult<Tracking>>({
      method: 'GET',
      url: `/tracking/page/${pageSize}/last`,
    });
  },

  // Create a new tracking
  createTracking: async (tracking: CreateTracking): Promise<Tracking> => {
    return await apiRequest<Tracking>({
      method: 'POST',
      url: '/tracking',
      data: tracking,
    });
  },

  // Update an existing tracking
  updateTracking: async (id: number, tracking: UpdateTracking): Promise<Tracking> => {
    return await apiRequest<Tracking>({
      method: 'PATCH',
      url: `/tracking/${id}`,
      data: tracking,
    });
  },

  // Delete a tracking
  deleteTracking: async (id: number): Promise<number> => {
    return await apiRequest<number>({
      method: 'DELETE',
      url: `/tracking/${id}`,
    });
  },
};

export default trackingApi;