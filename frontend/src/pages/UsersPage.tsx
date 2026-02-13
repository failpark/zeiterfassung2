import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import DataTable from '../components/ui/DataTable';
import Button from '../components/ui/Button';
import Dialog from '../components/ui/Dialog';
import Input from '../components/ui/Input';
import { CreateUser, PaginationResult, UpdateUser, User } from '../types';
import { userApi } from '../api';
import { useAuth } from '../contexts/AuthContext';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';
import { formatDate } from '../utils/date-utils';
import { PencilIcon, TrashIcon, PlusIcon } from '@heroicons/react/24/outline';

const userSchema = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().min(1, 'First name is required'),
  lastname: z.string().min(1, 'Last name is required'),
  email: z.string().email('Invalid email address'),
  password: z.string().min(6, 'Password must be at least 6 characters'),
  sys_role: z.enum(['admin', 'user']),
});

const updateUserSchema = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().min(1, 'First name is required'),
  lastname: z.string().min(1, 'Last name is required'),
  email: z.string().email('Invalid email address'),
  password: z.string().optional(),
  sys_role: z.enum(['admin', 'user']),
});

type UserFormValues = z.infer<typeof userSchema>;
type UpdateUserFormValues = z.infer<typeof updateUserSchema>;

const UsersPage: React.FC = () => {
  const { user: currentUser } = useAuth();
  const [users, setUsers] = useState<User[]>([]);
  const [pagination, setPagination] = useState<PaginationResult<User> | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;
  
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [error, setError] = useState<string | null>(null);

  const {
    register: registerAdd,
    handleSubmit: handleSubmitAdd,
    reset: resetAdd,
    formState: { errors: errorsAdd, isSubmitting: isSubmittingAdd },
  } = useForm<UserFormValues>({
    resolver: zodResolver(userSchema),
    defaultValues: {
      sys_role: 'user',
    },
  });

  const {
    register: registerEdit,
    handleSubmit: handleSubmitEdit,
    reset: resetEdit,
    formState: { errors: errorsEdit, isSubmitting: isSubmittingEdit },
  } = useForm<UpdateUserFormValues>({
    resolver: zodResolver(updateUserSchema),
  });

  useEffect(() => {
    fetchUsers();
  }, [currentPage]);

  const fetchUsers = async () => {
    try {
      setIsLoading(true);
      const data = await userApi.getUsers(currentPage, pageSize);
      setUsers(data.items);
      setPagination(data);
    } catch (error) {
      console.error('Error fetching users:', error);
      setError('Failed to load users. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddUser = async (data: UserFormValues) => {
    try {
      setError(null);
      const newUser: CreateUser = {
        username: data.username,
        firstname: data.firstname,
        lastname: data.lastname,
        email: data.email,
        password: data.password,
        sys_role: data.sys_role,
      };
      await userApi.createUser(newUser);
      setIsAddDialogOpen(false);
      resetAdd({
        sys_role: 'user',
      });
      fetchUsers();
    } catch (error: any) {
      console.error('Error adding user:', error);
      setError(error.message || 'Failed to add user. Please try again.');
    }
  };

  const handleEditUser = async (data: UpdateUserFormValues) => {
    if (!selectedUser) return;
    
    try {
      setError(null);
      const updatedUser: UpdateUser = {
        username: data.username,
        firstname: data.firstname,
        lastname: data.lastname,
        email: data.email,
        sys_role: data.sys_role,
      };
      
      // Only include password if it's provided
      if (data.password) {
        updatedUser.password = data.password;
      }
      
      await userApi.updateUser(selectedUser.id, updatedUser);
      setIsEditDialogOpen(false);
      resetEdit();
      fetchUsers();
    } catch (error: any) {
      console.error('Error updating user:', error);
      setError(error.message || 'Failed to update user. Please try again.');
    }
  };

  const handleDeleteUser = async () => {
    if (!selectedUser) return;
    
    try {
      setError(null);
      await userApi.deleteUser(selectedUser.id);
      setIsDeleteDialogOpen(false);
      fetchUsers();
    } catch (error: any) {
      console.error('Error deleting user:', error);
      setError(error.message || 'Failed to delete user. Please try again.');
    }
  };

  const openEditDialog = (user: User) => {
    setSelectedUser(user);
    resetEdit({
      username: user.username,
      firstname: user.firstname,
      lastname: user.lastname,
      email: user.email,
      sys_role: user.sys_role as 'admin' | 'user',
    });
    setIsEditDialogOpen(true);
  };

  const openDeleteDialog = (user: User) => {
    setSelectedUser(user);
    setIsDeleteDialogOpen(true);
  };

  const columns = [
    {
      header: 'Username',
      accessorKey: 'username',
    },
    {
      header: 'Name',
      accessorKey: (user: User) => `${user.firstname} ${user.lastname}`,
    },
    {
      header: 'Email',
      accessorKey: 'email',
    },
    {
      header: 'Role',
      accessorKey: 'sys_role',
    },
    {
      header: 'Created At',
      accessorKey: (user: User) => formatDate(user.created_at),
    },
    {
      header: 'Actions',
      accessorKey: (user: User) => (
        <div className="flex space-x-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              openEditDialog(user);
            }}
            className="rounded p-1 text-primary hover:bg-muted"
            disabled={!currentUser || currentUser.sys_role !== 'admin'}
          >
            <PencilIcon className="h-4 w-4" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              openDeleteDialog(user);
            }}
            className="rounded p-1 text-destructive hover:bg-muted"
            disabled={!currentUser || currentUser.sys_role !== 'admin' || user.id === currentUser.id}
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
          <h1 className="text-2xl font-semibold text-foreground">Users</h1>
          {currentUser?.sys_role === 'admin' && (
            <Button
              onClick={() => {
                resetAdd({
                  sys_role: 'user',
                });
                setIsAddDialogOpen(true);
              }}
              size="sm"
              className="gap-1"
            >
              <PlusIcon className="h-4 w-4" />
              Add User
            </Button>
          )}
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-destructive/10 p-3 text-destructive">
            <p className="text-sm">{error}</p>
          </div>
        )}

        <div className="mt-6">
          <DataTable
            data={users}
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

      {/* Add User Dialog */}
      <Dialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title="Add New User"
      >
        <form onSubmit={handleSubmitAdd(handleAddUser)} className="space-y-4">
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <Input
              label="Username"
              {...registerAdd('username')}
              error={errorsAdd.username?.message}
            />
            
            <Input
              label="Email"
              type="email"
              {...registerAdd('email')}
              error={errorsAdd.email?.message}
            />
            
            <Input
              label="First Name"
              {...registerAdd('firstname')}
              error={errorsAdd.firstname?.message}
            />
            
            <Input
              label="Last Name"
              {...registerAdd('lastname')}
              error={errorsAdd.lastname?.message}
            />
            
            <Input
              label="Password"
              type="password"
              {...registerAdd('password')}
              error={errorsAdd.password?.message}
            />
            
            <div className="space-y-2">
              <label htmlFor="sys_role" className="block text-sm font-medium text-foreground">
                Role
              </label>
              <select
                id="sys_role"
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                {...registerAdd('sys_role')}
              >
                <option value="user">User</option>
                <option value="admin">Admin</option>
              </select>
              {errorsAdd.sys_role && (
                <p className="text-sm font-medium text-destructive">
                  {errorsAdd.sys_role.message}
                </p>
              )}
            </div>
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
              Add User
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Edit User Dialog */}
      <Dialog
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        title="Edit User"
      >
        <form onSubmit={handleSubmitEdit(handleEditUser)} className="space-y-4">
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <Input
              label="Username"
              {...registerEdit('username')}
              error={errorsEdit.username?.message}
            />
            
            <Input
              label="Email"
              type="email"
              {...registerEdit('email')}
              error={errorsEdit.email?.message}
            />
            
            <Input
              label="First Name"
              {...registerEdit('firstname')}
              error={errorsEdit.firstname?.message}
            />
            
            <Input
              label="Last Name"
              {...registerEdit('lastname')}
              error={errorsEdit.lastname?.message}
            />
            
            <Input
              label="New Password (leave blank to keep current)"
              type="password"
              {...registerEdit('password')}
              error={errorsEdit.password?.message}
            />
            
            <div className="space-y-2">
              <label htmlFor="edit_sys_role" className="block text-sm font-medium text-foreground">
                Role
              </label>
              <select
                id="edit_sys_role"
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                {...registerEdit('sys_role')}
              >
                <option value="user">User</option>
                <option value="admin">Admin</option>
              </select>
              {errorsEdit.sys_role && (
                <p className="text-sm font-medium text-destructive">
                  {errorsEdit.sys_role.message}
                </p>
              )}
            </div>
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
              Update User
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Delete User Dialog */}
      <Dialog
        isOpen={isDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        title="Delete User"
        description="Are you sure you want to delete this user? This action cannot be undone."
      >
        <div className="flex justify-end space-x-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => setIsDeleteDialogOpen(false)}
          >
            Cancel
          </Button>
          <Button type="button" variant="destructive" onClick={handleDeleteUser}>
            Delete
          </Button>
        </div>
      </Dialog>
    </Layout>
  );
};

export default UsersPage;