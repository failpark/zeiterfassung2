import React, { useEffect, useState } from 'react';
import Layout from '../components/layout/Layout';
import {
  ClockIcon,
  UserGroupIcon,
  FolderIcon,
  BuildingOfficeIcon,
} from '@heroicons/react/24/outline';
import { trackingApi, clientApi, projectApi, userApi } from '../api';
import { formatDate, formatHours } from '../utils/date-utils';
import { Tracking } from '../types';
import { Link } from '@tanstack/react-router';
import DataTable from '../components/ui/DataTable';

const DashboardPage: React.FC = () => {
  const [recentTrackings, setRecentTrackings] = useState<Tracking[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [stats, setStats] = useState({
    clients: 0,
    projects: 0,
    users: 0,
    trackings: 0,
  });

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        
        // Fetch recent trackings
        const trackingData = await trackingApi.getTrackings(0, 5);
        setRecentTrackings(trackingData.items);
        
        // Fetch stats
        const [clientData, projectData, userData, trackingData2] = await Promise.all([
          clientApi.getLastPage(1),
          projectApi.getLastPage(1),
          userApi.getLastPage(1),
          trackingApi.getLastPage(1),
        ]);
        
        setStats({
          clients: clientData.total_items,
          projects: projectData.total_items,
          users: userData.total_items,
          trackings: trackingData2.total_items,
        });
      } catch (error) {
        console.error('Error fetching dashboard data:', error);
      } finally {
        setIsLoading(false);
      }
    };
    
    fetchData();
  }, []);

  const columns = [
    {
      header: 'Date',
      accessorKey: (tracking: Tracking) => formatDate(tracking.date),
    },
    {
      header: 'Description',
      accessorKey: (tracking: Tracking) => tracking.description || '-',
    },
    {
      header: 'Duration',
      accessorKey: (tracking: Tracking) => formatHours(tracking.performed),
    },
    {
      header: 'Billed',
      accessorKey: (tracking: Tracking) => formatHours(tracking.billed),
    },
  ];

  const StatCard = ({ title, value, icon: Icon, linkTo }: any) => (
    <div className="overflow-hidden rounded-lg bg-card shadow">
      <div className="p-5">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <Icon className="h-6 w-6 text-primary" aria-hidden="true" />
          </div>
          <div className="ml-5 w-0 flex-1">
            <dl>
              <dt className="truncate text-sm font-medium text-muted-foreground">{title}</dt>
              <dd>
                <div className="text-lg font-medium text-foreground">{value}</div>
              </dd>
            </dl>
          </div>
        </div>
      </div>
      {linkTo && (
        <div className="bg-muted px-5 py-3">
          <div className="text-sm">
            <Link
              to={linkTo}
              className="font-medium text-primary hover:text-primary/80"
            >
              View all
            </Link>
          </div>
        </div>
      )}
    </div>
  );

  return (
    <Layout>
      <div className="py-6">
        <h1 className="text-2xl font-semibold text-foreground">Dashboard</h1>
        
        {/* Stats grid */}
        <div className="mt-6 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            title="Total Trackings"
            value={isLoading ? 'Loading...' : stats.trackings}
            icon={ClockIcon}
            linkTo="/tracking"
          />
          <StatCard
            title="Total Clients"
            value={isLoading ? 'Loading...' : stats.clients}
            icon={BuildingOfficeIcon}
            linkTo="/clients"
          />
          <StatCard
            title="Total Projects"
            value={isLoading ? 'Loading...' : stats.projects}
            icon={FolderIcon}
            linkTo="/projects"
          />
          <StatCard
            title="Total Users"
            value={isLoading ? 'Loading...' : stats.users}
            icon={UserGroupIcon}
            linkTo="/users"
          />
        </div>
        
        {/* Recent trackings */}
        <div className="mt-8">
          <h2 className="text-lg font-medium text-foreground">Recent Time Entries</h2>
          <div className="mt-4">
            <DataTable
              data={recentTrackings}
              columns={columns}
              isLoading={isLoading}
              onRowClick={(tracking) => {
                // Handle clicking on a tracking entry
                window.location.href = `/tracking/${tracking.id}`;
              }}
            />
          </div>
          <div className="mt-4 text-right">
            <Link
              to="/tracking"
              className="text-sm font-medium text-primary hover:text-primary/80"
            >
              View all time entries →
            </Link>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default DashboardPage;