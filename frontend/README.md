# Time Tracking Frontend

A React frontend for the time tracking application. This application provides interfaces to manage time entries, clients, projects, activities, and users.

## Features

- JWT authentication with protected routes
- Dashboard with summary statistics
- Time tracking management (create, edit, delete time entries)
- Client management
- Project management
- Activity management
- User management (admin only)
- Dark/light mode toggle
- Responsive design for mobile and desktop

## Tech Stack

- React + TypeScript
- Vite as the build tool
- TanStack Router for routing
- Tailwind CSS for styling
- Headless UI and Heroicons for UI components
- Axios for API requests
- React Hook Form with Zod for form validation
- JWT for authentication

## Project Structure

```
├── public/            # Static files
├── src/
│   ├── api/           # API integration
│   ├── assets/        # Images, fonts, etc.
│   ├── components/    # Reusable components
│   │   ├── layout/    # Layout components
│   │   └── ui/        # UI components
│   ├── contexts/      # React contexts
│   ├── hooks/         # Custom hooks
│   ├── pages/         # Page components
│   ├── types/         # TypeScript types
│   ├── utils/         # Utility functions
│   ├── App.tsx        # Main App component
│   ├── main.tsx       # Entry point
│   └── router.tsx     # Router configuration
├── .env.development   # Development environment variables
├── .env.production    # Production environment variables
├── index.html         # HTML template
├── package.json       # Dependencies and scripts
└── tailwind.config.js # Tailwind CSS configuration
```

## Getting Started

### Prerequisites

- Node.js 16+ and npm

### Installation

1. Clone the repository
2. Navigate to the frontend directory
3. Install dependencies:

```bash
npm install
```

### Development

Run the development server:

```bash
npm run dev
```

### Building for Production

Create a production build:

```bash
npm run build
```

### Environment Variables

- `VITE_API_URL`: The URL of the backend API

## API Integration

The frontend integrates with the following API endpoints:

- `/login` - Authentication
- `/user` - User management
- `/client` - Client management
- `/project` - Project management
- `/activity` - Activity management
- `/tracking` - Time tracking

## License

MIT