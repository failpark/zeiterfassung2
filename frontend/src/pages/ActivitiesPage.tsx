import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import DataTable from '../components/ui/DataTable';
import Button from '../components/ui/Button';
import Dialog from '../components/ui/Dialog';
import Input from '../components/ui/Input';
import { Activity, CreateActivity, PaginationResult, UpdateActivity } from '../types';
import { activityApi } from '../api';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { formatDate } from '../utils/date-utils';
import { PencilIcon, TrashIcon, PlusIcon } from '@heroicons/react/24/outline';

const activitySchema = z.object({
  name: z.string().min(1, 'Activity name is required'),
  token: z.string().optional(),
});

type ActivityFormValues = z.infer<typeof activitySchema>;

const ActivitiesPage: React.FC = () => {
  const [activities, setActivities] = useState<Activity[]>([]);
  const [pagination, setPagination] = useState<PaginationResult<Activity> | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;
  
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedActivity, setSelectedActivity] = useState<Activity | null>(null);
  const [error, setError] = useState<string | null>(null);

  const {
    register: registerAdd,
    handleSubmit: handleSubmitAdd,
    reset: resetAdd,
    formState: { errors: errorsAdd, isSubmitting: isSubmittingAdd },
  } = useForm<ActivityFormValues>({
    resolver: zodResolver(activitySchema),
  });

  const {
    register: registerEdit,
    handleSubmit: handleSubmitEdit,
    reset: resetEdit,
    formState: { errors: errorsEdit, isSubmitting: isSubmittingEdit },
  } = useForm<ActivityFormValues>({
    resolver: zodResolver(activitySchema),
  });

  useEffect(() => {
    fetchActivities();
  }, [currentPage]);

  const fetchActivities = async () => {
    try {
      setIsLoading(true);
      const data = await activityApi.getActivities(currentPage, pageSize);
      setActivities(data.items);
      setPagination(data);
    } catch (error) {
      console.error('Error fetching activities:', error);
      setError('Failed to load activities. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddActivity = async (data: ActivityFormValues) => {
    try {
      setError(null);
      const newActivity: CreateActivity = {
        name: data.name,
        token: data.token || undefined,
      };
      await activityApi.createActivity(newActivity);
      setIsAddDialogOpen(false);
      resetAdd();
      fetchActivities();
    } catch (error) {
      console.error('Error adding activity:', error);
      setError('Failed to add activity. Please try again.');
    }
  };

  const handleEditActivity = async (data: ActivityFormValues) => {
    if (!selectedActivity) return;
    
    try {
      setError(null);
      const updatedActivity: UpdateActivity = {
        name: data.name,
        token: data.token ? data.token : selectedActivity.token,
      };
      await activityApi.updateActivity(selectedActivity.id, updatedActivity);
      setIsEditDialogOpen(false);
      resetEdit();
      fetchActivities();
    } catch (error) {
      console.error('Error updating activity:', error);
      setError('Failed to update activity. Please try again.');
    }
  };

  const handleDeleteActivity = async () => {
    if (!selectedActivity) return;
    
    try {
      setError(null);
      await activityApi.deleteActivity(selectedActivity.id);
      setIsDeleteDialogOpen(false);
      fetchActivities();
    } catch (error) {
      console.error('Error deleting activity:', error);
      setError('Failed to delete activity. Please try again.');
    }
  };

  const openEditDialog = (activity: Activity) => {
    setSelectedActivity(activity);
    resetEdit({
      name: activity.name,
      token: activity.token || '',
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (activity: Activity) => {
    setSelectedActivity(activity);
    setIsDeleteDialogOpen(true);
  };

  const columns = [
    {
      header: 'Token',
      accessorKey: (activity: Activity) => activity.token || '-',
      className: 'w-1/6',
    },
    {
      header: 'Name',
      accessorKey: 'name',
    },
    {
      header: 'Created At',
      accessorKey: (activity: Activity) => formatDate(activity.created_at),
    },
    {
      header: 'Updated At',
      accessorKey: (activity: Activity) => formatDate(activity.updated_at),
    },
    {
      header: 'Actions',
      accessorKey: (activity: Activity) => (
        <div className="flex space-x-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              openEditDialog(activity);
            }}
            className="rounded p-1 text-primary hover:bg-muted"
          >
            <PencilIcon className="h-4 w-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openDeleteDialog(activity);
            }}
            className="rounded p-1 text-destructive hover:bg-muted"
          >
            <TrashIcon className="h-4 w-4" />
          </button>
        </div>
      ),
      className: 'w-1/6',
    },
  ];

  return (
    <Layout>
      <div className="py-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold text-foreground">Activities</h1>
          <Button
            onClick={() => {
              resetAdd();
              setIsAddDialogOpen(true);
            }}
            size="sm"
            className="gap-1"
          >
            <PlusIcon className="h-4 w-4" />
            Add Activity
          </Button>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-destructive/10 p-3 text-destructive">
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div className="mt-6">
          <DataTable
            data={activities}
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

      {/* Add Activity Dialog */}
      <Dialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title="Add New Activity"
      >
        <form onSubmit={handleSubmitAdd(handleAddActivity)} className="space-y-4">
          <Input
            label="Activity Name"
            {...registerAdd('name')}
            error={errorsAdd.name?.message}
          />
          
          <Input
            label="Token (optional)"
            {...registerAdd('token')}
            error={errorsAdd.token?.message}
          />
          
          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsAddDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" isLoading={isSubmittingAdd}>
              Add Activity
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Edit Activity Dialog */}
      <Dialog
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        title="Edit Activity"
      >
        <form onSubmit={handleSubmitEdit(handleEditActivity)} className="space-y-4">
          <Input
            label="Activity Name"
            {...registerEdit('name')}
            error={errorsEdit.name?.message}
          />
          
          <Input
            label="Token (optional)"
            {...registerEdit('token')}
            error={errorsEdit.token?.message}
          />
          
          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsEditDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" isLoading={isSubmittingEdit}>
              Update Activity
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Delete Activity Dialog */}
      <Dialog
        isOpen={isDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        title="Delete Activity"
        description="Are you sure you want to delete this activity? This action cannot be undone."
      >
        <div className="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => setIsDeleteDialogOpen(false)}
          >
            Cancel
          </Button>
          <Button type="button" variant="destructive" onClick={handleDeleteActivity}>
            Delete
          </Button>
        </div>
      </Dialog>
    </Layout>
  );
};

export default ActivitiesPage;