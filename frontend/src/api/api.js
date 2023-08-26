const fetchHello = async () => {
    try {
        const response = await fetch('http://localhost:8080/api/hello');
        const data = await response.json(); // Parse JSON response
        return data.message; // Return the message from the response
    } catch (error) {
        console.error('Error fetching data:', error);
        return 'Error fetching data';
    }
};

export { fetchHello };




  