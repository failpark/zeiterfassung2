import { Activity, CreateActivity, PaginationResult, UpdateActivity } from '../types';
import { apiRequest } from './axiosClient';

export const activityApi = {
  // Get a single activity by ID
  getActivity: async (id: number): Promise<Activity> => {
    return await apiRequest<Activity>({
      method: 'GET',
      url: `/activity/${id}`,
    });
  },

  // Get paginated activities
  getActivities: async (page: number, pageSize: number): Promise<PaginationResult<Activity>> => {
    return await apiRequest<PaginationResult<Activity>>({
      method: 'GET',
      url: `/activity/page/${pageSize}/${page}`,
    });
  },

  // Get the last page of activities
  getLastPage: async (pageSize: number): Promise<PaginationResult<Activity>> => {
    return await apiRequest<PaginationResult<Activity>>({
      method: 'GET',
      url: `/activity/page/${pageSize}/last`,
    });
  },

  // Create a new activity
  createActivity: async (activity: CreateActivity): Promise<Activity> => {
    return await apiRequest<Activity>({
      method: 'POST',
      url: '/activity',
      data: activity,
    });
  },

  // Update an existing activity
  updateActivity: async (id: number, activity: UpdateActivity): Promise<Activity> => {
    return await apiRequest<Activity>({
      method: 'PATCH',
      url: `/activity/${id}`,
      data: activity,
    });
  },

  // Delete an activity
  deleteActivity: async (id: number): Promise<void> => {
    return await apiRequest<void>({
      method: 'DELETE',
      url: `/activity/${id}`,
    });
  },
};

export default activityApi;