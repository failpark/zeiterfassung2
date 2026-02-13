import { CreateProject, PaginationResult, Project, UpdateProject } from '../types';
import { apiRequest } from './axiosClient';

export const projectApi = {
  // Get a single project by ID
  getProject: async (id: number): Promise<Project> => {
    return await apiRequest<Project>({
      method: 'GET',
      url: `/project/${id}`,
    });
  },

  // Get paginated projects
  getProjects: async (page: number, pageSize: number): Promise<PaginationResult<Project>> => {
    return await apiRequest<PaginationResult<Project>>({
      method: 'GET',
      url: `/project/page/${pageSize}/${page}`,
    });
  },

  // Get the last page of projects
  getLastPage: async (pageSize: number): Promise<PaginationResult<Project>> => {
    return await apiRequest<PaginationResult<Project>>({
      method: 'GET',
      url: `/project/page/${pageSize}/last`,
    });
  },

  // Create a new project
  createProject: async (project: CreateProject): Promise<Project> => {
    return await apiRequest<Project>({
      method: 'POST',
      url: '/project',
      data: project,
    });
  },

  // Update an existing project
  updateProject: async (id: number, project: UpdateProject): Promise<Project> => {
    return await apiRequest<Project>({
      method: 'PATCH',
      url: `/project/${id}`,
      data: project,
    });
  },

  // Delete a project
  deleteProject: async (id: number): Promise<void> => {
    return await apiRequest<void>({
      method: 'DELETE',
      url: `/project/${id}`,
    });
  },
};

export default projectApi;