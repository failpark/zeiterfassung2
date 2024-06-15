import axios from "axios";

const API_URL = "http://localhost:8000/";

class AuthService {
	login(email: string, password: string) {
		return axios
			.post(API_URL + "login", {
				email,
				password,
			})
			.then((response) => {
				if (response.data.token) {
					localStorage.setItem("user", JSON.stringify(response.data));
				}

				return response.data;
			});
	}

	logout() {
		localStorage.removeItem("user");
	}

	getCurrentUser() {
		const userStr = localStorage.getItem("user");
		if (userStr) return JSON.parse(userStr);

		return null;
	}
}

export default new AuthService();
