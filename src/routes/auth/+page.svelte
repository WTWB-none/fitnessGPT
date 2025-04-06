<script>
	import { json } from '@sveltejs/kit';
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';

	let show_reg = false;
	let login = '',
		email = '',
		password = '',
		repeatPassword = '',
		phone = '';
	let errors = {};
	let isValid = false;
	let yandexData = null;

	async function registerUser() {
		let request_phone = phone.replace(/[^\d+]/g, '');
		let request = {
			nickname: login,
			email: email,
			phone: request_phone,
			auth: { Password: { password: password } }
		};
		let response = await fetch('http://localhost:8000/register/user', {
			method: 'POST',
			body: JSON.stringify(request)
		});
		let result = await response.json();
		if (result.error) {
			errors.backend_error = result.error;
		} else {
			goto('/complete_info#' + result.data.user_id);
		}
	}

	async function AuthUser() {
		let request_phone = email.startsWith('+') ? email.replace(/[^\d+]/g, '') : '';
		let request = {
			login: email.startsWith('+') ? request_phone : email,
			ident: email.startsWith('+') ? 'phone' : 'email',
			password: password
		};
		let response = await fetch('http://localhost:8000/auth/login', {
			method: 'POST',
			body: JSON.stringify(request)
		});
		let result = await response.json();
		console.log(result.data.id);
		let checkpass = await fetch('http://localhost:8000/get_user_data/' + result.data.id);
		let checkresult = await checkpass.json();
		console.log(checkresult);
		if (result.error) {
			errors.backend_error = result.error;
		} else if (checkresult.data.age === null) {
			console.log(checkresult);
			goto('/complete_info#' + result.data.id);
		} else {
			console.log('pisun');
			goto('/plan#' + result.data.id);
		}
	}

	function validate() {
		errors = {};
		const input = email.trim();
		const digitsOnly = input.replace(/\D/g, '');
		const isValidEmail = /^[\w.-]+@[\w.-]+\.\w{2,}$/.test(input);
		const isValidPhone = digitsOnly.length === 11 && digitsOnly.startsWith('7');

		if (!show_reg) {
			if (!isValidEmail && !isValidPhone)
				errors.email = 'Введите корректный email или номер телефона';
			if (password.length < 6) errors.password = 'Минимум 6 символов';
		} else {
			if (!/^[a-zA-Z0-9]{3,15}$/.test(login)) errors.login = 'Логин: 3-15 букв или цифр';
			if (!isValidEmail) errors.email = 'Некорректный email';
			if (!/^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{6,}$/.test(password))
				errors.password = 'Мин. 6 символов, 1 буква, 1 цифра';
			if (repeatPassword !== password) errors.repeatPassword = 'Пароли не совпадают';
			if (phone.replace(/\D/g, '').length !== 11) errors.phone = 'Неполный номер';
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
		validate();
	}

	function formatLoginInput(event) {
		let value = event.target.value;
		let digits = value.replace(/\D/g, '');
		if (digits.length >= 5 || value.startsWith('+7') || digits.startsWith('8')) {
			if (digits.startsWith('8')) digits = digits.slice(1);
			if (digits.startsWith('7')) digits = digits.slice(1);
			if (digits.length > 10) digits = digits.slice(0, 10);
			let formatted = '+7';
			if (digits.length > 0) formatted += ` (${digits.slice(0, 3)}`;
			if (digits.length > 3) formatted += `) ${digits.slice(3, 6)}`;
			if (digits.length > 6) formatted += `-${digits.slice(6, 8)}`;
			if (digits.length > 8) formatted += `-${digits.slice(8, 10)}`;
			email = formatted;
		} else {
			email = value;
		}
		validate();
	}

	function toggleForm(toRegister) {
		show_reg = toRegister;
		validate();
	}

	onMount(() => {
		const script = document.createElement('script');
		script.src =
			'https://yastatic.net/s3/passport-sdk/autofill/v1/sdk-suggest-with-polyfills-latest.js';
		script.onload = () => {
			YaAuthSuggest.init(
				{
					client_id: 'dccd034c8bf54b02a6e0923e02285e68',
					response_type: 'token',
					redirect_uri: 'http://localhost:5173/summon_user'
				},
				'http://localhost:5173/auth',
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
				.catch((error) => {
					errors.backend_error = 'Ошибка инициализации Яндекс авторизации';
				});
		};
		script.onerror = () => console.error('Ошибка загрузки скрипта');
		document.head.appendChild(script);
		window.addEventListener(
			'message',
			async (event) => {
				console.log(event.data.data.user_id);
				let checkpass = await fetch(
					'http://localhost:8000/get_user_data/' + event.data.data.user_id
				);
				let checkresult = await checkpass.json();
				console.log(checkresult);
				if (event.data.error) {
					errors.backend_error = result.error;
				} else if (checkresult.data.age == null) {
					goto('/complete_info#' + event.data.data.user_id);
				} else {
					console.log('pisun');
					goto('/plan#' + event.data.data.user_id);
				}
			},
			false
		);
	});
</script>

<div class="backdrop">
	<form class="auth-form" on:submit|preventDefault>
		{#if !show_reg}
			<h1>Вход</h1>
			<input
				type="text"
				placeholder="Email или номер телефона"
				bind:value={email}
				on:input={formatLoginInput}
			/>
			<p class="error">{errors.email || ''}</p>
			<input type="password" placeholder="Пароль" bind:value={password} on:input={validate} />
			<p class="error">{errors.password || ''}</p>
			<button class="auth-btn" disabled={!isValid} on:click={AuthUser}>Войти</button>
			<p>
				Нет аккаунта?
				<span
					class="link"
					on:click={() => toggleForm(true)}
					on:keydown={(e) => e.key === 'Enter' && toggleForm(true)}
					aria-label="register"
					role="button"
					tabindex="0"
				>
					Зарегистрироваться
				</span>
			</p>
		{:else}
			<h1>Регистрация</h1>
			<input type="text" placeholder="Логин" bind:value={login} on:input={validate} />
			<p class="error">{errors.login || ''}</p>
			<input type="email" placeholder="Email" bind:value={email} on:input={validate} />
			<p class="error">{errors.email || ''}</p>
			<input type="password" placeholder="Пароль" bind:value={password} on:input={validate} />
			<p class="error">{errors.password || ''}</p>
			<input
				type="password"
				placeholder="Повторите пароль"
				bind:value={repeatPassword}
				on:input={validate}
			/>
			<p class="error">{errors.repeatPassword || ''}</p>
			<input
				type="text"
				placeholder="+7 (___) ___-__-__"
				bind:value={phone}
				on:input={formatPhone}
				on:blur={validate}
			/>
			<p class="error">{errors.phone || ''}</p>
			<button class="auth-btn" disabled={!isValid} on:click={registerUser}
				>Зарегистрироваться</button
			>
			<p>
				Уже есть аккаунт?
				<span
					class="link"
					on:click={() => toggleForm(false)}
					on:keydown={(e) => e.key === 'Enter' && toggleForm(false)}
					aria-label="login"
					role="button"
					tabindex="0"
				>
					Войти
				</span>
			</p>
		{/if}
		{#if errors.backend_error}
			<p class="error">{errors.backend_error}</p>
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
	input,
	button {
		width: 100%;
		padding: 1em;
		border: none;
		border-radius: 12px;
		background-color: rgba(255, 255, 255, 0.08);
		color: white;
		font-size: 1em;
	}
	button.auth-btn {
		background: linear-gradient(90deg, #8e44ad, #3498db, #e91e63);
		font-weight: bold;
		cursor: pointer;
		transition: opacity 0.3s;
	}
	button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.error {
		color: #e91e63;
		font-size: 0.8em;
		margin: -0.8em 0 0.5em 0;
	}
	.link {
		color: #9b59b6;
		cursor: pointer;
		text-decoration: underline;
	}
</style>
