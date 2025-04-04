<script>
	import { onMount } from 'svelte';
	onMount(() => {
		const script = document.createElement('script');
		script.src =
			'https://yastatic.net/s3/passport-sdk/autofill/v1/sdk-suggest-with-polyfills-latest.js';
		script.onload = () => {
			YaAuthSuggest.init(
				{
					client_id: 'dccd034c8bf54b02a6e0923e02285e68',
					response_type: 'token'
				},
				'https://examplesite.com',
				{
					view: 'button',
					parentId: 'ya-auth',
					buttonView: 'main',
					buttonTheme: 'light',
					buttonSize: 'xl',
					buttonBorderRadius: 60
				}
			)
				.then(({ handler }) => handler())
				.then((data) => console.log('Сообщение с токеном', data))
				.catch((error) => console.log('Обработка ошибки', error));
		};
		document.head.appendChild(script);
	});
</script>

<div class="backdrop">
	<form class="auth-form" id="container">
		<h1>Войти</h1>
		<input type="text" placeholder="Введите email или номер телефона" />
		<input type="password" placeholder="Введите пароль" />
		<button class="auth-btn" type="submit" tabindex="0">Войти</button>
		<div id="ya-auth"></div>
	</form>
</div>

<style>
	.backdrop {
		width: 100%;
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	input {
		width: 80%;
		padding: 1em;
		border: 1px solid black;
		border-radius: 15.5px;
	}

	input:focus {
		outline: none;
	}

	button {
		width: 80%;
		border: 1px solid black;
		color: white;
		background-color: black;
		padding: 1em;
		border-radius: 15.5px;
	}

	.auth-form {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2em;
		width: 40%;
		height: 60vh;
		border: 1px solid black;
		border-radius: 20px;
	}

	#ya-auth {
		width: 80%;
	}
</style>
