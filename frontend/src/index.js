import { fetchHello } from '../src/api/api.js';

const fetchButton = document.getElementById('fetchButton');
const responseElement = document.getElementById('response');

fetchButton.addEventListener('click', async () => {
    try {
        // Change the text of the button while fetching data
        fetchButton.textContent = 'Fetching...';

        const data = await fetchHello();

        // Update the content of the response element
        responseElement.textContent = data;

        // Change the text of the button back to its original text
        fetchButton.textContent = 'Fetch Data';
    } catch (error) {
        console.error('Error:', error);

        // Reset the button text in case of an error
        fetchButton.textContent = 'Fetch Data';
    }
});
