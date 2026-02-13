import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import DataTable from '../components/ui/DataTable';
import Button from '../components/ui/Button';
import Dialog from '../components/ui/Dialog';
import Input from '../components/ui/Input';
import { PaginationResult, Project, Client, CreateProject, UpdateProject } from '../types';
import { projectApi, clientApi } from '../api';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { formatDate } from '../utils/date-utils';
import { PencilIcon, TrashIcon, PlusIcon } from '@heroicons/react/24/outline';

const projectSchema = z.object({
  name: z.string().min(1, 'Project name is required'),
  client_id: z.string().min(1, 'Client is required'),
});

type ProjectFormValues = z.infer<typeof projectSchema>;

const ProjectsPage: React.FC = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [pagination, setPagination] = useState<PaginationResult<Project> | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;
  
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [error, setError] = useState<string | null>(null);

  const {
    register: registerAdd,
    handleSubmit: handleSubmitAdd,
    reset: resetAdd,
    formState: { errors: errorsAdd, isSubmitting: isSubmittingAdd },
  } = useForm<ProjectFormValues>({
    resolver: zodResolver(projectSchema),
  });

  const {
    register: registerEdit,
    handleSubmit: handleSubmitEdit,
    reset: resetEdit,
    formState: { errors: errorsEdit, isSubmitting: isSubmittingEdit },
  } = useForm<ProjectFormValues>({
    resolver: zodResolver(projectSchema),
  });

  useEffect(() => {
    const fetchInitialData = async () => {
      try {
        setIsLoading(true);
        // Fetch all clients for dropdown menus
        const clientsData = await clientApi.getLastPage(100);
        setClients(clientsData.items);
        fetchProjects();
      } catch (error) {
        console.error('Error fetching initial data:', error);
        setError('Failed to load initial data. Please try again.');
      }
    };
    
    fetchInitialData();
  }, []);
  
  useEffect(() => {
    if (clients.length > 0) {
      fetchProjects();
    }
  }, [currentPage, clients.length]);

  const fetchProjects = async () => {
    try {
      setIsLoading(true);
      const data = await projectApi.getProjects(currentPage, pageSize);
      setProjects(data.items);
      setPagination(data);
    } catch (error) {
      console.error('Error fetching projects:', error);
      setError('Failed to load projects. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddProject = async (data: ProjectFormValues) => {
    try {
      setError(null);
      const newProject: CreateProject = {
        name: data.name,
        client_id: parseInt(data.client_id),
      };
      await projectApi.createProject(newProject);
      setIsAddDialogOpen(false);
      resetAdd();
      fetchProjects();
    } catch (error) {
      console.error('Error adding project:', error);
      setError('Failed to add project. Please try again.');
    }
  };

  const handleEditProject = async (data: ProjectFormValues) => {
    if (!selectedProject) return;
    
    try {
      setError(null);
      const updatedProject: UpdateProject = {
        name: data.name,
      };
      await projectApi.updateProject(selectedProject.id, updatedProject);
      setIsEditDialogOpen(false);
      resetEdit();
      fetchProjects();
    } catch (error) {
      console.error('Error updating project:', error);
      setError('Failed to update project. Please try again.');
    }
  };

  const handleDeleteProject = async () => {
    if (!selectedProject) return;
    
    try {
      setError(null);
      await projectApi.deleteProject(selectedProject.id);
      setIsDeleteDialogOpen(false);
      fetchProjects();
    } catch (error) {
      console.error('Error deleting project:', error);
      setError('Failed to delete project. Please try again.');
    }
  };

  const openEditDialog = (project: Project) => {
    setSelectedProject(project);
    resetEdit({
      name: project.name,
      client_id: project.client_id.toString(),
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (project: Project) => {
    setSelectedProject(project);
    setIsDeleteDialogOpen(true);
  };

  const getClientName = (clientId: number) => {
    const client = clients.find(c => c.id === clientId);
    return client?.name || 'Unknown Client';
  };

  const columns = [
    {
      header: 'Project Name',
      accessorKey: 'name',
    },
    {
      header: 'Client',
      accessorKey: (project: Project) => getClientName(project.client_id),
    },
    {
      header: 'Created At',
      accessorKey: (project: Project) => formatDate(project.created_at),
    },
    {
      header: 'Updated At',
      accessorKey: (project: Project) => formatDate(project.updated_at),
    },
    {
      header: 'Actions',
      accessorKey: (project: Project) => (
        <div className="flex space-x-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              openEditDialog(project);
            }}
            className="rounded p-1 text-primary hover:bg-muted"
          >
            <PencilIcon className="h-4 w-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openDeleteDialog(project);
            }}
            className="rounded p-1 text-destructive hover:bg-muted"
          >
            <TrashIcon className="h-4 w-4" />
          </button>
        </div>
      ),
    },
  ];

  return (
    <Layout>
      <div className="py-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold text-foreground">Projects</h1>
          <Button
            onClick={() => {
              resetAdd();
              setIsAddDialogOpen(true);
            }}
            size="sm"
            className="gap-1"
          >
            <PlusIcon className="h-4 w-4" />
            Add Project
          </Button>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-destructive/10 p-3 text-destructive">
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div className="mt-6">
          <DataTable
            data={projects}
            columns={columns}
            isLoading={isLoading}
            pagination={
              pagination
                ? {
                    pageIndex: pagination.page,
                    pageSize: pagination.page_size,
                    pageCount: pagination.num_pages,
                    totalItems: pagination.total_items,
                  }
                : undefined
            }
            onPageChange={setCurrentPage}
          />
        </div>
      </div>

      {/* Add Project Dialog */}
      <Dialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title="Add New Project"
      >
        <form onSubmit={handleSubmitAdd(handleAddProject)} className="space-y-4">
          <Input
            label="Project Name"
            {...registerAdd('name')}
            error={errorsAdd.name?.message}
          />
          
          <div className="space-y-2">
            <label htmlFor="client_id" className="block text-sm font-medium text-foreground">
              Client
            </label>
            <select
              id="client_id"
              className={`w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${
                errorsAdd.client_id ? 'border-destructive' : ''
              }`}
              {...registerAdd('client_id')}
            >
              <option value="">Select a client</option>
              {clients.map(client => (
                <option key={client.id} value={client.id}>
                  {client.name}
                </option>
              ))}
            </select>
            {errorsAdd.client_id && (
              <p className="text-sm font-medium text-destructive">
                {errorsAdd.client_id.message}
              </p>
            )}
          </div>
          
          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsAddDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" isLoading={isSubmittingAdd}>
              Add Project
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Edit Project Dialog */}
      <Dialog
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        title="Edit Project"
      >
        <form onSubmit={handleSubmitEdit(handleEditProject)} className="space-y-4">
          <Input
            label="Project Name"
            {...registerEdit('name')}
            error={errorsEdit.name?.message}
          />
          
          <div className="space-y-2">
            <label htmlFor="edit_client_id" className="block text-sm font-medium text-foreground">
              Client (Cannot be changed)
            </label>
            <select
              id="edit_client_id"
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm opacity-60 ring-offset-background"
              {...registerEdit('client_id')}
              disabled
            >
              {clients.map(client => (
                <option key={client.id} value={client.id}>
                  {client.name}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground">
              Note: Client assignment cannot be changed after creation
            </p>
          </div>
          
          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsEditDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" isLoading={isSubmittingEdit}>
              Update Project
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Delete Project Dialog */}
      <Dialog
        isOpen={isDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        title="Delete Project"
        description="Are you sure you want to delete this project? This action cannot be undone."
      >
        <div className="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => setIsDeleteDialogOpen(false)}
          >
            Cancel
          </Button>
          <Button type="button" variant="destructive" onClick={handleDeleteProject}>
            Delete
          </Button>
        </div>
      </Dialog>
    </Layout>
  );
};

export default ProjectsPage;