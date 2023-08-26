import { fetchHello } from '../src/api/api.js';

const fetchButton = document.getElementById('fetchButton');
const responseElement = document.getElementById('response');

fetchButton.addEventListener('click', async () => {
    try {
        const data = await fetchHello();
        // Update the content of the response element
        responseElement.textContent = data;
    } catch (error) {
        console.error('Error:', error);

        // Reset the button text in case of an error
        fetchButton.textContent = 'Fetch Data';
    }
});
