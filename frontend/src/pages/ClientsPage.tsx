import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import DataTable from '../components/ui/DataTable';
import Button from '../components/ui/Button';
import Dialog from '../components/ui/Dialog';
import Input from '../components/ui/Input';
import { Client, CreateClient, PaginationResult, UpdateClient } from '../types';
import { clientApi } from '../api';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { formatDate } from '../utils/date-utils';
import { PencilIcon, TrashIcon, PlusIcon } from '@heroicons/react/24/outline';

const clientSchema = z.object({
  name: z.string().min(1, 'Client name is required'),
});

type ClientFormValues = z.infer<typeof clientSchema>;

const ClientsPage: React.FC = () => {
  const [clients, setClients] = useState<Client[]>([]);
  const [pagination, setPagination] = useState<PaginationResult<Client> | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;
  
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedClient, setSelectedClient] = useState<Client | null>(null);
  const [error, setError] = useState<string | null>(null);

  const {
    register: registerAdd,
    handleSubmit: handleSubmitAdd,
    reset: resetAdd,
    formState: { errors: errorsAdd, isSubmitting: isSubmittingAdd },
  } = useForm<ClientFormValues>({
    resolver: zodResolver(clientSchema),
  });

  const {
    register: registerEdit,
    handleSubmit: handleSubmitEdit,
    reset: resetEdit,
    formState: { errors: errorsEdit, isSubmitting: isSubmittingEdit },
  } = useForm<ClientFormValues>({
    resolver: zodResolver(clientSchema),
  });

  useEffect(() => {
    fetchClients();
  }, [currentPage]);

  const fetchClients = async () => {
    try {
      setIsLoading(true);
      const data = await clientApi.getClients(currentPage, pageSize);
      setClients(data.items);
      setPagination(data);
    } catch (error) {
      console.error('Error fetching clients:', error);
      setError('Failed to load clients. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddClient = async (data: ClientFormValues) => {
    try {
      setError(null);
      const newClient: CreateClient = {
        name: data.name,
      };
      await clientApi.createClient(newClient);
      setIsAddDialogOpen(false);
      resetAdd();
      fetchClients();
    } catch (error) {
      console.error('Error adding client:', error);
      setError('Failed to add client. Please try again.');
    }
  };

  const handleEditClient = async (data: ClientFormValues) => {
    if (!selectedClient) return;
    
    try {
      setError(null);
      const updatedClient: UpdateClient = {
        name: data.name,
      };
      await clientApi.updateClient(selectedClient.id, updatedClient);
      setIsEditDialogOpen(false);
      resetEdit();
      fetchClients();
    } catch (error) {
      console.error('Error updating client:', error);
      setError('Failed to update client. Please try again.');
    }
  };

  const handleDeleteClient = async () => {
    if (!selectedClient) return;
    
    try {
      setError(null);
      await clientApi.deleteClient(selectedClient.id);
      setIsDeleteDialogOpen(false);
      fetchClients();
    } catch (error) {
      console.error('Error deleting client:', error);
      setError('Failed to delete client. Please try again.');
    }
  };

  const openEditDialog = (client: Client) => {
    setSelectedClient(client);
    resetEdit({
      name: client.name,
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (client: Client) => {
    setSelectedClient(client);
    setIsDeleteDialogOpen(true);
  };

  const columns = [
    {
      header: 'Name',
      accessorKey: 'name',
    },
    {
      header: 'Created At',
      accessorKey: (client: Client) => formatDate(client.created_at),
    },
    {
      header: 'Updated At',
      accessorKey: (client: Client) => formatDate(client.updated_at),
    },
    {
      header: 'Actions',
      accessorKey: (client: Client) => (
        <div className="flex space-x-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              openEditDialog(client);
            }}
            className="rounded p-1 text-primary hover:bg-muted"
          >
            <PencilIcon className="h-4 w-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openDeleteDialog(client);
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
          <h1 className="text-2xl font-semibold text-foreground">Clients</h1>
          <Button
            onClick={() => {
              resetAdd();
              setIsAddDialogOpen(true);
            }}
            size="sm"
            className="gap-1"
          >
            <PlusIcon className="h-4 w-4" />
            Add Client
          </Button>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-destructive/10 p-3 text-destructive">
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div className="mt-6">
          <DataTable
            data={clients}
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

      {/* Add Client Dialog */}
      <Dialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title="Add New Client"
      >
        <form onSubmit={handleSubmitAdd(handleAddClient)} className="space-y-4">
          <Input
            label="Client Name"
            {...registerAdd('name')}
            error={errorsAdd.name?.message}
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
              Add Client
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Edit Client Dialog */}
      <Dialog
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        title="Edit Client"
      >
        <form onSubmit={handleSubmitEdit(handleEditClient)} className="space-y-4">
          <Input
            label="Client Name"
            {...registerEdit('name')}
            error={errorsEdit.name?.message}
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
              Update Client
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Delete Client Dialog */}
      <Dialog
        isOpen={isDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        title="Delete Client"
        description="Are you sure you want to delete this client? This action cannot be undone."
      >
        <div className="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => setIsDeleteDialogOpen(false)}
          >
            Cancel
          </Button>
          <Button type="button" variant="destructive" onClick={handleDeleteClient}>
            Delete
          </Button>
        </div>
      </Dialog>
    </Layout>
  );
};

export default ClientsPage;