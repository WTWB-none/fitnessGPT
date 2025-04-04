<script>
	import { json } from '@sveltejs/kit';
	import { onMount } from 'svelte';

	let show_reg = false;
	let login = '',
		email = '',
		password = '',
		repeatPassword = '',
		phone = '';
	let errors = {};
	let isValid = false;

	async function registerUser() {
		let request_phone = phone.replace(/[^\d+]/g, '');
		console.log(request_phone);
		let request = {
			name: login,
			email: email,
			phone: request_phone,
			auth: {
				Password: {
					password: password
				}
			}
		};
		let response = await fetch('http://localhost:8000/auth/user', {
			method: 'POST',
			body: JSON.stringify(request)
		});

		if (response.ok) {
			window.location.href = '/profile';
		}
	}

	function validate() {
		errors = {};

		if (!show_reg) {
			if (!/^[\w.-]+@[\w.-]+\.\w{2,}$/.test(email)) {
				errors.email = 'Некорректный email';
			}
			if (password.length < 6) {
				errors.password = 'Минимум 6 символов';
			}
		} else {
			if (!/^[a-zA-Z0-9]{3,15}$/.test(login)) {
				errors.login = 'Логин: 3-15 букв или цифр';
			}
			if (!/^[\w.-]+@[\w.-]+\.\w{2,}$/.test(email)) {
				errors.email = 'Некорректный email';
			}
			if (!/^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{6,}$/.test(password)) {
				errors.password = 'Мин. 6 символов, 1 буква, 1 цифра';
			}
			if (repeatPassword !== password) {
				errors.repeatPassword = 'Пароли не совпадают';
			}
			if (phone.replace(/\D/g, '').length !== 11) {
				errors.phone = 'Неполный номер';
			}
		}

		isValid = Object.keys(errors).length === 0;
	}

	function formatPhone(event) {
		let digits = event.target.value.replace(/\D/g, '');
		if (digits.startsWith('7')) digits = digits.slice(1);
		if (digits.length > 10) digits = digits.slice(0, 10);

		let formatted = '+7';
		if (digits.length > 0) formatted += ` (${digits.slice(0, 3)}`;
		if (digits.length > 3) formatted += `) ${digits.slice(3, 6)}`;
		if (digits.length > 6) formatted += `-${digits.slice(6, 8)}`;
		if (digits.length > 8) formatted += `-${digits.slice(8, 10)}`;

		phone = formatted;
	}

	onMount(() => {
		const script = document.createElement('script');
		script.src =
			'https://yastatic.net/s3/passport-sdk/autofill/v1/sdk-suggest-with-polyfills-latest.js';
		script.onload = () => {
			YaAuthSuggest.init(
				{ client_id: 'dccd034c8bf54b02a6e0923e02285e68', response_type: 'token' },
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
				.then((data) => console.log('Токен:', data))
				.catch((error) => console.log('Ошибка:', error));
		};
		document.head.appendChild(script);
	});
</script>

<div class="backdrop">
	<form class="auth-form" id="container" onsubmit={() => console.log('Форма отправлена!')}>
		{#if !show_reg}
			<h1>Вход</h1>
			<input type="text" placeholder="Email" bind:value={email} oninput={validate} />
			<p class="error">{errors.email}</p>
			<input type="password" placeholder="Пароль" bind:value={password} oninput={validate} />
			<p class="error">{errors.password}</p>
			<button class="auth-btn" type="submit" disabled={!isValid}>Войти</button>
			<p>
				Нет аккаунта? <span
					class="link"
					onclick={() => {
						show_reg = true;
						validate();
					}}
					aria-label="register"
					role="button"
					onkeydown={() => {}}
					tabindex="0">Зарегистрироваться</span
				>
			</p>
		{:else}
			<h1>Регистрация</h1>
			<input type="text" placeholder="Логин" bind:value={login} oninput={validate} />
			<p class="error">{errors.login}</p>
			<input type="email" placeholder="Email" bind:value={email} oninput={validate} />
			<p class="error">{errors.email}</p>
			<input type="password" placeholder="Пароль" bind:value={password} oninput={validate} />
			<p class="error">{errors.password}</p>
			<input
				type="password"
				placeholder="Повторите пароль"
				bind:value={repeatPassword}
				oninput={validate}
			/>
			<p class="error">{errors.repeatPassword}</p>
			<input
				type="text"
				placeholder="+7 (___) ___-__-__"
				bind:value={phone}
				oninput={formatPhone}
				onblur={validate}
			/>
			<p class="error">{errors.phone}</p>
			<button class="auth-btn" type="submit" disabled={!isValid} onclick={() => registerUser()}
				>Зарегистрироваться</button
			>
			<p>
				Уже есть аккаунт? <span
					class="link"
					onclick={() => {
						show_reg = false;
						validate();
					}}
					aria-label="register"
					role="button"
					onkeydown={() => {}}
					tabindex="0">Войти</span
				>
			</p>
		{/if}
		<div id="ya-auth"></div>
	</form>
</div>

<style>
	:global(body) {
		margin: 0;
		font-family: 'Segoe UI', sans-serif;
		background: linear-gradient(225deg, #2d0c5e, #000000);
		color: white;
	}

	.backdrop {
		width: 100%;
		height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.auth-form {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1em;
		width: 90%;
		max-width: 400px;
		padding: 2.5em;
		border-radius: 35px;
		background-color: rgba(255, 255, 255, 0.05);
		backdrop-filter: blur(8px);
	}

	h1 {
		font-size: 1.5em;
		text-align: center;
	}

	input {
		width: 100%;
		padding: 1em;
		border: none;
		border-radius: 12px;
		background-color: rgba(255, 255, 255, 0.08);
		color: white;
		font-size: 1em;
	}

	.error {
		color: #e91e63;
		font-size: 0.8em;
		margin: -0.8em 0 0.5em 0;
	}

	button.auth-btn {
		width: 100%;
		padding: 1em;
		border: none;
		border-radius: 12px;
		background: linear-gradient(90deg, #8e44ad, #3498db, #e91e63);
		color: white;
		font-weight: bold;
		cursor: pointer;
		transition: opacity 0.3s;
	}

	button.auth-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.link {
		color: #9b59b6;
		cursor: pointer;
		text-decoration: underline;
	}

	#ya-auth {
		width: 100%;
	}
</style>
