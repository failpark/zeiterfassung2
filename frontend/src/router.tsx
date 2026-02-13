import { 
  Router, 
  Route, 
  RootRoute, 
  createReactRouter, 
  createRouteConfig,
  Outlet,
  redirect
} from '@tanstack/react-router';
import LoginPage from './pages/LoginPage';
import DashboardPage from './pages/DashboardPage';
import ClientsPage from './pages/ClientsPage';
import ProjectsPage from './pages/ProjectsPage';
import ActivitiesPage from './pages/ActivitiesPage';
import TrackingPage from './pages/TrackingPage';
import UsersPage from './pages/UsersPage';
import { authApi } from './api';

// Root route
const rootRoute = new RootRoute({
  component: () => <Outlet />,
});

// Create a route for the login page
const loginRoute = new Route({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
  beforeLoad: async () => {
    // Redirect to dashboard if already authenticated
    if (authApi.isAuthenticated()) {
      throw redirect({
        to: '/',
      });
    }
  },
});

// Create a layout route for authenticated routes
const authenticatedRoute = new Route({
  getParentRoute: () => rootRoute,
  id: 'authenticated',
  beforeLoad: async () => {
    // Redirect to login if not authenticated
    if (!authApi.isAuthenticated()) {
      throw redirect({
        to: '/login',
      });
    }
  },
});

// Create routes for various pages
const indexRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/',
  component: DashboardPage,
});

const clientsRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/clients',
  component: ClientsPage,
});

const projectsRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/projects',
  component: ProjectsPage,
});

const activitiesRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/activities',
  component: ActivitiesPage,
});

const trackingRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/tracking',
  component: TrackingPage,
});

const usersRoute = new Route({
  getParentRoute: () => authenticatedRoute,
  path: '/users',
  component: UsersPage,
});

// Create the route tree
const routeTree = rootRoute.addChildren([
  loginRoute,
  authenticatedRoute.addChildren([
    indexRoute,
    clientsRoute,
    projectsRoute,
    activitiesRoute,
    trackingRoute,
    usersRoute,
  ]),
]);

// Create the router
const router = createReactRouter({ routeTree });

export default router;