<script>
	import { onMount } from 'svelte';

	async function getUserData(token) {
		const response = await fetch('https://login.yandex.ru/info', {
			method: 'GET',
			headers: { Authorization: `OAuth ${token}` }
		});
		if (!response.ok) throw new Error(`Ошибка Яндекс API: ${response.status}`);
		return await response.json();
	}

	async function sendData(query) {
		const response = await fetch('http://localhost:8000/register/user', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(query)
		});
		if (!response.ok) throw new Error(`Ошибка сервера: ${response.status}`);
		return await response.json();
	}

	onMount(async () => {
		try {
			const hash = window.location.hash.slice(1);
			const params = new URLSearchParams(hash);
			const accessToken = params.get('access_token');
			if (!accessToken) throw new Error('Токен не найден');

			const userData = await getUserData(accessToken);
			const request = {
				nickname: userData.login || 'unknown',
				email: userData.default_email || '',
				phone: userData.default_phone?.number || '',
				auth: { Yandex: { provider_user_id: userData.client_id || userData.id } }
			};

			const serverResponse = await sendData(request);

			if (window.opener) {
				window.opener.postMessage(serverResponse, '*');
				window.close();
			}
		} catch (err) {
			console.error('Authentication error:', err);

			if (window.opener) {
				window.opener.postMessage({ success: false, error: err.message }, '*');
				window.close();
			}
		}
	});
</script>

<div>
	<p>Обработка авторизации Яндекс...</p>
</div>

<style>
	div {
		display: flex;
		justify-content: center;
		align-items: center;
		height: 100vh;
		background-color: #333;
	}
	p {
		color: #fff;
		font-family: Arial, sans-serif;
		font-size: 18px;
	}
</style>
