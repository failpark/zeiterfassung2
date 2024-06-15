import * as React from "react";
import { flushSync } from "react-dom";
import {
	createFileRoute,
	getRouteApi,
	useNavigate,
} from "@tanstack/react-router";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, useFormContext } from "react-hook-form";
import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { ExclamationCircleIcon } from "@heroicons/react/20/solid";
import { toast } from "sonner";

import { useAuth } from "../auth";
import AuthService from "@/services/auth.service";

export const Route = createFileRoute("/login")({
	validateSearch: z.object({
		redirect: z.string().catch("/"),
	}),
	component: LoginComponent,
});

const formSchema = z.object({
	email: z.string().email({ message: "Invalid email address" }),
	password: z.string(),
});

const routeApi = getRouteApi("/login");

function LoginComponent() {
	const auth = useAuth();
	const navigate = useNavigate();

	const [isSubmitting, setIsSubmitting] = React.useState(false);
	const [name, setName] = React.useState("");

	const search = routeApi.useSearch();

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			email: "",
			password: "",
		},
	});

	if (!form.formState.isValid && form.formState.errors.email) {
		toast.error(form.formState.errors.email.message);
	}

	function onSubmit(values: z.infer<typeof formSchema>) {
		setIsSubmitting(true);
		AuthService.login(values.email, values.password).then(
			() => {
				setIsSubmitting(false);
				auth.setUser(values.email);
				navigate({ to: search.redirect });
			},
			error => {
				toast.error(error);
				setIsSubmitting(false);
			}
	);
	}

	return (
		<div className="flex min-h-full flex-1 items-center justify-center px-4 py-12 sm:px-6 lg:px-8">
			<div className="w-full max-w-sm space-y-10">
				<div>
					<img
						className="mx-auto h-12 w-auto"
						src="/vendamus-logo.svg"
						alt="Vendamus"
					/>
					<h2 className="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
						Sign in to your account
					</h2>
				</div>
				<Form {...form}>
					<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
						<div className="relative -space-y-px rounded-md shadow-sm">
							<div className="pointer-events-none absolute inset-0 z-10 rounded-md ring-1 ring-inset ring-gray-300" />
							<FormField
								control={form.control}
								name="email"
								render={({ field }) => (
									<FormItem>
										<FormLabel className="sr-only">Email Address</FormLabel>
										<FormControl>
											<Input
												id="email-address"
												type="email"
												autoComplete="email"
												required
												className="relative block w-full rounded-t-md border-0 py-1.5 text-gray-900 ring-1 ring-inset ring-gray-100 placeholder:text-gray-400 focus:z-10 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
												placeholder="Email address"
												{...field}
											/>
										</FormControl>
									</FormItem>
								)}
							/>
							<FormField
								control={form.control}
								name="password"
								render={({ field }) => (
									<FormItem>
										<FormLabel className="sr-only">Password</FormLabel>
										<FormControl>
											<Input
												id="password"
												type="password"
												autoComplete="current-password"
												required
												className="relative block w-full rounded-b-md border-0 py-1.5 text-gray-900 ring-1 ring-inset ring-gray-100 placeholder:text-gray-400 focus:z-10 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
												placeholder="Password"
												{...field}
											/>
										</FormControl>
									</FormItem>
								)}
							/>
						</div>
						<div>
							<button
								type="submit"
								className="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
							>
								{isSubmitting ? "Loading..." : "Login"}
							</button>
						</div>
					</form>
				</Form>
			</div>
		</div>
	);
}
