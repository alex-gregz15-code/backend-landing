const messageElement = document.querySelector('#message');
const reloadButton = document.querySelector('#reload');
const registerForm = document.querySelector('#register-form');
const saveStatus = document.querySelector('#save-status');

async function fetchBackendMessage() {
  try {
    const data = await window.backendApi.hello();
    messageElement.textContent = data.message;
  } catch (error) {
    messageElement.textContent = 'Could not reach backend server.';
    console.error(error);
  }
}

reloadButton.addEventListener('click', fetchBackendMessage);
fetchBackendMessage();

registerForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  saveStatus.textContent = 'Saving...';

  const payload = {
    fullName: document.querySelector('#full-name').value.trim(),
    email: document.querySelector('#email').value.trim(),
    password: document.querySelector('#password').value,
    role: document.querySelector('#role').value,
  };

  try {
    const data = await window.backendApi.register(payload);
    saveStatus.textContent = `Saved user ${data.user.email} with id ${data.user.id}.`;
    registerForm.reset();
  } catch (error) {
    saveStatus.textContent = error.message;
    console.error(error);
  }
});
