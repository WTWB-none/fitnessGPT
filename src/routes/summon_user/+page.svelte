<script>
	import { onMount } from 'svelte';

	let params = $state();
	let accessToken = $state('');
	let userData = $state({});
	let request = $state({});
	async function getUserData() {
		let response = await fetch('https://login.yandex.ru/info', {
			method: 'GET',
			headers: {
				Authorization: `OAuth ${accessToken}`
			}
		});
		return response.json();
	}

	async function sendData(query) {
		let response = await fetch('http://localhost:8000/auth/user', {
			method: 'POST',
			body: JSON.stringify(query)
		});
		console.log(response);
		return response;
	}

	onMount(async () => {
		params = window.location.hash.slice(1);
		accessToken = params.split('&')[0].split('=')[1];
		userData = await getUserData();
		request = {
			name: userData.login,
			email: userData.default_email,
			phone: userData.default_phone.number,
			auth: {
				Yandex: {
					provider_user_id: userData.client_id
				}
			}
		};
		let result = await sendData(request);
		console.log(result);
		if (result.status === 200) {
			window.close();
		} else {
			alert('Error creating user');
		}
	});
</script>
