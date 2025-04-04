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
		let response = await fetch('http://localhost:8000/create_user', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(query)
		});

		return response;
	}

	onMount(async () => {
		params = window.location.hash.slice(1);
		accessToken = params.split('&')[0].split('=')[1];
		userData = await getUserData();
		request = {
			password: userData.client_id,
			name: userData.login,
			email: userData.default_email,
			phone: userData.default_phone.number
		};
		let result = sendData(request);
		if (result.ok) {
			window.close();
		}
	});
</script>

<h1>{JSON.stringify(request)}</h1>
