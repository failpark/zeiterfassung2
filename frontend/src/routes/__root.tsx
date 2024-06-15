import * as React from "react";
import {
	Outlet,
	redirect,
	createRootRouteWithContext,
} from "@tanstack/react-router";
// import { TanStackRouterDevtools } from "@tanstack/router-devtools";
const TanStackRouterDevtools =
	process.env.NODE_ENV === "production"
		? () => null
		: React.lazy(() =>
				// Lazy load in development
				import("@tanstack/router-devtools").then((res) => ({
					default: res.TanStackRouterDevtools,
					// For Embedded Mode
					// default: res.TanStackRouterDevtoolsPanel
				})),
		);

import { useAuth, type AuthContext } from "../auth";
import Nav from "@/components/tw/nav";
import { Toaster } from "sonner";

interface MyRouterContext {
	auth: AuthContext;
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
	beforeLoad: ({ context, location }) => {
		if (!context.auth.isAuthenticated && location.pathname !== "/login") {
			throw redirect({
				to: "/login",
				search: {
					redirect: location.href,
				},
			});
		}
	},

	component: RootComponent,
});

function RootComponent() {
	const [showRouteDevtools, setShowRouteDevtools] = React.useState(
		process.env.NODE_ENV === "development",
	);
	React.useEffect(() => {
		// @ts-expect-error
		window.toggleRouteDevtools = () => setShowRouteDevtools((old) => !old);
	});
	const auth = useAuth();
	return (
		<div className="min-h-full">
			{auth.isAuthenticated ? <Nav /> : ""}
			<div className="py-10">
				<Outlet />
				{showRouteDevtools && (
					<React.Suspense>
						<TanStackRouterDevtools
							position="bottom-right"
							initialIsOpen={false}
						/>
					</React.Suspense>
				)}
			</div>
			<Toaster richColors={true} closeButton />
		</div>
	);
}
