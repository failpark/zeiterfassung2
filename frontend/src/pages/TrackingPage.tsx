import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import DataTable from '../components/ui/DataTable';
import Button from '../components/ui/Button';
import Dialog from '../components/ui/Dialog';
import Input from '../components/ui/Input';
import { 
  Activity, 
  Client, 
  CreateTracking, 
  PaginationResult, 
  Project, 
  Tracking, 
  UpdateTracking 
} from '../types';
import { 
  activityApi, 
  clientApi, 
  projectApi, 
  trackingApi 
} from '../api';
import { useAuth } from '../contexts/AuthContext';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { 
  formatDate, 
  formatTime, 
  calculateDuration, 
  getCurrentDateISO,
  getCurrentTimeISO 
} from '../utils/date-utils';
import { PencilIcon, TrashIcon, PlusIcon, ClockIcon } from '@heroicons/react/24/outline';

const trackingSchema = z.object({
  client_id: z.string().min(1, 'Client is required'),
  project_id: z.string().min(1, 'Project is required'),
  date: z.string().min(1, 'Date is required'),
  begin: z.string().min(1, 'Start time is required'),
  end: z.string().min(1, 'End time is required'),
  pause: z.string().optional(),
  description: z.string().optional(),
  activities: z.array(z.string()).min(1, 'At least one activity is required'),
});

type TrackingFormValues = z.infer<typeof trackingSchema>;

const TrackingPage: React.FC = () => {
  const { user } = useAuth();
  const [trackings, setTrackings] = useState<Tracking[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [filteredProjects, setFilteredProjects] = useState<Project[]>([]);
  const [activities, setActivities] = useState<Activity[]>([]);
  const [pagination, setPagination] = useState<PaginationResult<Tracking> | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;
  
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedTracking, setSelectedTracking] = useState<Tracking | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedClientId, setSelectedClientId] = useState<string>('');

  const {
    register: registerAdd,
    handleSubmit: handleSubmitAdd,
    reset: resetAdd,
    setValue: setValueAdd,
    watch: watchAdd,
    formState: { errors: errorsAdd, isSubmitting: isSubmittingAdd },
  } = useForm<TrackingFormValues>({
    resolver: zodResolver(trackingSchema),
    defaultValues: {
      date: getCurrentDateISO(),
      begin: getCurrentTimeISO().slice(0, 5),
      end: getCurrentTimeISO().slice(0, 5),
      activities: [],
    }
  });

  const {
    register: registerEdit,
    handleSubmit: handleSubmitEdit,
    reset: resetEdit,
    setValue: setValueEdit,
    watch: watchEdit,
    formState: { errors: errorsEdit, isSubmitting: isSubmittingEdit },
  } = useForm<TrackingFormValues>({
    resolver: zodResolver(trackingSchema),
  });

  // Watch for client changes in the add form
  const watchAddClientId = watchAdd('client_id');
  
  // Watch for client changes in the edit form
  const watchEditClientId = watchEdit('client_id');

  useEffect(() => {
    const fetchInitialData = async () => {
      try {
        setIsLoading(true);
        // Fetch all reference data
        const [clientsData, projectsData, activitiesData] = await Promise.all([
          clientApi.getLastPage(100),
          projectApi.getLastPage(100),
          activityApi.getLastPage(100),
        ]);
        
        setClients(clientsData.items);
        setProjects(projectsData.items);
        setActivities(activitiesData.items);
        
        fetchTrackings();
      } catch (error) {
        console.error('Error fetching initial data:', error);
        setError('Failed to load initial data. Please try again.');
      }
    };
    
    fetchInitialData();
  }, []);
  
  useEffect(() => {
    if (clients.length > 0 && projects.length > 0 && activities.length > 0) {
      fetchTrackings();
    }
  }, [currentPage, clients.length, projects.length, activities.length]);

  // Filter projects by client in add form
  useEffect(() => {
    if (watchAddClientId) {
      const clientProjects = projects.filter(
        project => project.client_id === parseInt(watchAddClientId)
      );
      setFilteredProjects(clientProjects);
      // Reset project selection
      setValueAdd('project_id', '');
    }
  }, [watchAddClientId, projects, setValueAdd]);
  
  // Filter projects by client in edit form
  useEffect(() => {
    if (watchEditClientId) {
      const clientProjects = projects.filter(
        project => project.client_id === parseInt(watchEditClientId)
      );
      setFilteredProjects(clientProjects);
    }
  }, [watchEditClientId, projects]);

  const fetchTrackings = async () => {
    try {
      setIsLoading(true);
      const data = await trackingApi.getTrackings(currentPage, pageSize);
      setTrackings(data.items);
      setPagination(data);
    } catch (error) {
      console.error('Error fetching trackings:', error);
      setError('Failed to load time entries. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddTracking = async (data: TrackingFormValues) => {
    if (!user) return;
    
    try {
      setError(null);
      
      // Calculate performed and billed hours
      const performed = calculateDuration(data.begin, data.end, data.pause);
      
      const newTracking: CreateTracking = {
        client_id: parseInt(data.client_id),
        user_id: user.id,
        project_id: parseInt(data.project_id),
        date: data.date,
        begin: `${data.begin}:00`,
        end: `${data.end}:00`,
        pause: data.pause ? `${data.pause}:00` : undefined,
        performed,
        billed: performed, // Default to same as performed
        description: data.description,
        activities: data.activities.map(id => parseInt(id)),
      };
      
      await trackingApi.createTracking(newTracking);
      setIsAddDialogOpen(false);
      resetAdd({
        date: getCurrentDateISO(),
        begin: getCurrentTimeISO().slice(0, 5),
        end: getCurrentTimeISO().slice(0, 5),
        activities: [],
      });
      fetchTrackings();
    } catch (error) {
      console.error('Error adding tracking:', error);
      setError('Failed to add time entry. Please try again.');
    }
  };

  const handleEditTracking = async (data: TrackingFormValues) => {
    if (!selectedTracking) return;
    
    try {
      setError(null);
      
      // Calculate performed and billed hours
      const performed = calculateDuration(data.begin, data.end, data.pause);
      
      const updatedTracking: UpdateTracking = {
        date: data.date,
        begin: `${data.begin}:00`,
        end: `${data.end}:00`,
        pause: data.pause ? `${data.pause}:00` : null,
        performed,
        billed: performed, // Update to match performed
        description: data.description,
        activities: data.activities.map(id => parseInt(id)),
      };
      
      await trackingApi.updateTracking(selectedTracking.id, updatedTracking);
      setIsEditDialogOpen(false);
      resetEdit();
      fetchTrackings();
    } catch (error) {
      console.error('Error updating tracking:', error);
      setError('Failed to update time entry. Please try again.');
    }
  };

  const handleDeleteTracking = async () => {
    if (!selectedTracking) return;
    
    try {
      setError(null);
      await trackingApi.deleteTracking(selectedTracking.id);
      setIsDeleteDialogOpen(false);
      fetchTrackings();
    } catch (error) {
      console.error('Error deleting tracking:', error);
      setError('Failed to delete time entry. Please try again.');
    }
  };

  const openEditDialog = (tracking: Tracking) => {
    setSelectedTracking(tracking);
    
    // Filter projects for this client
    const clientProjects = projects.filter(
      project => project.client_id === tracking.client_id
    );
    setFilteredProjects(clientProjects);
    
    // Convert timestrings to format expected by time inputs (HH:MM)
    const beginTime = formatTime(tracking.begin);
    const endTime = formatTime(tracking.end);
    const pauseTime = tracking.pause ? formatTime(tracking.pause) : '';
    
    resetEdit({
      client_id: tracking.client_id.toString(),
      project_id: tracking.project_id.toString(),
      date: tracking.date,
      begin: beginTime,
      end: endTime,
      pause: pauseTime,
      description: tracking.description || '',
      activities: tracking.activities?.map(id => id.toString()) || [],
    });
    
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (tracking: Tracking) => {
    setSelectedTracking(tracking);
    setIsDeleteDialogOpen(true);
  };

  const getClientName = (clientId: number) => {
    const client = clients.find(c => c.id === clientId);
    return client?.name || 'Unknown Client';
  };

  const getProjectName = (projectId: number) => {
    const project = projects.find(p => p.id === projectId);
    return project?.name || 'Unknown Project';
  };

  const getActivityNames = (activityIds: number[] | undefined) => {
    if (!activityIds || activityIds.length === 0) return '-';
    
    return activityIds
      .map(id => {
        const activity = activities.find(a => a.id === id);
        return activity?.name;
      })
      .filter(Boolean)
      .join(', ');
  };

  const columns = [
    {
      header: 'Date',
      accessorKey: (tracking: Tracking) => formatDate(tracking.date),
    },
    {
      header: 'Client',
      accessorKey: (tracking: Tracking) => getClientName(tracking.client_id),
    },
    {
      header: 'Project',
      accessorKey: (tracking: Tracking) => getProjectName(tracking.project_id),
    },
    {
      header: 'Time',
      accessorKey: (tracking: Tracking) => (
        <div>
          {formatTime(tracking.begin)} - {formatTime(tracking.end)}
          {tracking.pause && <span className="ml-2 text-xs text-muted-foreground">(Pause: {formatTime(tracking.pause)})</span>}
        </div>
      ),
    },
    {
      header: 'Duration',
      accessorKey: (tracking: Tracking) => `${tracking.performed}h`,
    },
    {
      header: 'Activities',
      accessorKey: (tracking: Tracking) => getActivityNames(tracking.activities),
    },
    {
      header: 'Description',
      accessorKey: (tracking: Tracking) => tracking.description || '-',
    },
    {
      header: 'Actions',
      accessorKey: (tracking: Tracking) => (
        <div className="flex space-x-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              openEditDialog(tracking);
            }}
            className="rounded p-1 text-primary hover:bg-muted"
          >
            <PencilIcon className="h-4 w-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openDeleteDialog(tracking);
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
          <h1 className="text-2xl font-semibold text-foreground">Time Tracking</h1>
          <div className="flex gap-2">
            <Button
              onClick={() => {
                resetAdd({
                  date: getCurrentDateISO(),
                  begin: getCurrentTimeISO().slice(0, 5),
                  end: getCurrentTimeISO().slice(0, 5),
                  activities: [],
                });
                setFilteredProjects([]);
                setIsAddDialogOpen(true);
              }}
              size="sm"
              className="gap-1"
            >
              <PlusIcon className="h-4 w-4" />
              Add Entry
            </Button>
          </div>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-destructive/10 p-3 text-destructive">
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div className="mt-6">
          <DataTable
            data={trackings}
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

      {/* Add Tracking Dialog */}
      <Dialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title="Add Time Entry"
        contentClassName="max-w-xl"
      >
        <form onSubmit={handleSubmitAdd(handleAddTracking)} className="space-y-4">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <label htmlFor="date" className="block text-sm font-medium text-foreground">
                Date
              </label>
              <Input
                id="date"
                type="date"
                {...registerAdd('date')}
                error={errorsAdd.date?.message}
              />
            </div>
            
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
            
            <div className="space-y-2">
              <label htmlFor="project_id" className="block text-sm font-medium text-foreground">
                Project
              </label>
              <select
                id="project_id"
                className={`w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${
                  errorsAdd.project_id ? 'border-destructive' : ''
                }`}
                {...registerAdd('project_id')}
                disabled={!watchAddClientId}
              >
                <option value="">Select a project</option>
                {filteredProjects.map(project => (
                  <option key={project.id} value={project.id}>
                    {project.name}
                  </option>
                ))}
              </select>
              {errorsAdd.project_id && (
                <p className="text-sm font-medium text-destructive">
                  {errorsAdd.project_id.message}
                </p>
              )}
              {!watchAddClientId && (
                <p className="text-xs text-muted-foreground">
                  Select a client first
                </p>
              )}
            </div>
            
            <div className="space-y-2">
              <label htmlFor="begin" className="block text-sm font-medium text-foreground">
                Start Time
              </label>
              <Input
                id="begin"
                type="time"
                {...registerAdd('begin')}
                error={errorsAdd.begin?.message}
              />
            </div>
            
            <div className="space-y-2">
              <label htmlFor="end" className="block text-sm font-medium text-foreground">
                End Time
              </label>
              <Input
                id="end"
                type="time"
                {...registerAdd('end')}
                error={errorsAdd.end?.message}
              />
            </div>
            
            <div className="space-y-2">
              <label htmlFor="pause" className="block text-sm font-medium text-foreground">
                Pause (optional)
              </label>
              <Input
                id="pause"
                type="time"
                {...registerAdd('pause')}
                error={errorsAdd.pause?.message}
              />
            </div>
          </div>
          
          <div className="space-y-2">
            <label className="block text-sm font-medium text-foreground">
              Activities
            </label>
            <div className="grid max-h-40 grid-cols-2 gap-2 overflow-y-auto rounded-md border border-input p-2 md:grid-cols-3">
              {activities.map(activity => (
                <div key={activity.id} className="flex items-center">
                  <input
                    type="checkbox"
                    id={`activity-${activity.id}`}
                    value={activity.id}
                    {...registerAdd('activities')}
                    className="h-4 w-4 rounded border-input"
                  />
                  <label
                    htmlFor={`activity-${activity.id}`}
                    className="ml-2 block text-sm"
                  >
                    {activity.name}
                  </label>
                </div>
              ))}
            </div>
            {errorsAdd.activities && (
              <p className="text-sm font-medium text-destructive">
                {errorsAdd.activities.message}
              </p>
            )}
          </div>
          
          <div className="space-y-2">
            <label htmlFor="description" className="block text-sm font-medium text-foreground">
              Description (optional)
            </label>
            <textarea
              id="description"
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              rows={3}
              {...registerAdd('description')}
            ></textarea>
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
              Add Time Entry
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Edit Tracking Dialog */}
      <Dialog
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        title="Edit Time Entry"
        contentClassName="max-w-xl"
      >
        <form onSubmit={handleSubmitEdit(handleEditTracking)} className="space-y-4">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <label htmlFor="edit_date" className="block text-sm font-medium text-foreground">
                Date
              </label>
              <Input
                id="edit_date"
                type="date"
                {...registerEdit('date')}
                error={errorsEdit.date?.message}
              />
            </div>
            
            <div className="space-y-2">
              <label htmlFor="edit_client_id" className="block text-sm font-medium text-foreground">
                Client
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
            </div>
            
            <div className="space-y-2">
              <label htmlFor="edit_project_id" className="block text-sm font-medium text-foreground">
                Project
              </label>
              <select
                id="edit_project_id"
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm opacity-60 ring-offset-background"
                {...registerEdit('project_id')}
                disabled
              >
                {filteredProjects.map(project => (
                  <option key={project.id} value={project.id}>
                    {project.name}
                  </option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground">
                Client and project cannot be changed
              </p>
            </div>
            
            <div className="space-y-2">
              <label htmlFor="edit_begin" className="block text-sm font-medium text-foreground">
                Start Time
              </label>
              <Input
                id="edit_begin"
                type="time"
                {...registerEdit('begin')}
                error={errorsEdit.begin?.message}
              />
            </div>
            
            <div className="space-y-2">
              <label htmlFor="edit_end" className="block text-sm font-medium text-foreground">
                End Time
              </label>
              <Input
                id="edit_end"
                type="time"
                {...registerEdit('end')}
                error={errorsEdit.end?.message}
              />
            </div>
            
            <div className="space-y-2">
              <label htmlFor="edit_pause" className="block text-sm font-medium text-foreground">
                Pause (optional)
              </label>
              <Input
                id="edit_pause"
                type="time"
                {...registerEdit('pause')}
                error={errorsEdit.pause?.message}
              />
            </div>
          </div>
          
          <div className="space-y-2">
            <label className="block text-sm font-medium text-foreground">
              Activities
            </label>
            <div className="grid max-h-40 grid-cols-2 gap-2 overflow-y-auto rounded-md border border-input p-2 md:grid-cols-3">
              {activities.map(activity => (
                <div key={activity.id} className="flex items-center">
                  <input
                    type="checkbox"
                    id={`edit-activity-${activity.id}`}
                    value={activity.id}
                    {...registerEdit('activities')}
                    className="h-4 w-4 rounded border-input"
                  />
                  <label
                    htmlFor={`edit-activity-${activity.id}`}
                    className="ml-2 block text-sm"
                  >
                    {activity.name}
                  </label>
                </div>
              ))}
            </div>
            {errorsEdit.activities && (
              <p className="text-sm font-medium text-destructive">
                {errorsEdit.activities.message}
              </p>
            )}
          </div>
          
          <div className="space-y-2">
            <label htmlFor="edit_description" className="block text-sm font-medium text-foreground">
              Description (optional)
            </label>
            <textarea
              id="edit_description"
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              rows={3}
              {...registerEdit('description')}
            ></textarea>
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
              Update Time Entry
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Delete Tracking Dialog */}
      <Dialog
        isOpen={isDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        title="Delete Time Entry"
        description="Are you sure you want to delete this time entry? This action cannot be undone."
      >
        <div className="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => setIsDeleteDialogOpen(false)}
          >
            Cancel
          </Button>
          <Button type="button" variant="destructive" onClick={handleDeleteTracking}>
            Delete
          </Button>
        </div>
      </Dialog>
    </Layout>
  );
};

export default TrackingPage;