# OpenWebHMI - OpenSource HMI Project @2023

Description
------

The objetive is create an opensource HMI - Human Machine Interface
  - Web Development for User Interface.
  - Use OpenSource Tools, Libraries and Frameworks.
  - Connect with Databases.
  - Connect with PLC's , include librarys like TCP/IP
    - I'm going to start testing on AllenBradley
  - Use as HMI or SCADA to run in any Operation System (Windows, Linux, MacOS)

Features
------

Real Time Graphic Interface, Menus, Buttons, Messages, Alarms, Graphics, Reports, OEE, Visual KPIs, Part Traceability, Recipes, Predictive Maintenance and More.

Software Tools
------

- Front End : Javascript, HTML, CSS.
- Back End: Golang.
- Networking: TCP/IP
- Databases: SQLite.
- Framework: Gin
- Javascript Library: htmx 
- ORM Library: GORM.

Project Structure
------

- `backend/`: Contains the Go backend code.
  - `main.go`: Entry point for the backend server.
  - `api/`: API routes and handlers.
  - `models/`: Data models and GORM schemas.
  - `services/`: Business logic and services.
  - `config/`: Configuration files.
  - `tcp/`: TPC/IP for communication.
  - `tests/`: Directory for unit tests.
  - `docs/`: Directory for backend documentation.
  - ...

- `frontend/`: Contains the JavaScript frontend code.
  - `public/`: Static assets (HTML, CSS, images).
  - `src/`: Main source code directory.
    - `components/`: Reusable UI components.
    - `pages/`: Page components and routing.
    - `api/`: Frontend API calls (htmx).
    - `utils/`: Utility functions.
    - `App.js`: Main App component.
    - `index.js`: Entry point for the frontend.
  - `docs/`: Directory for frontend documentation.
  - `package.json`: Frontend dependencies and scripts.
  - ...

- `database/`: Database-related files (migrations, seeds, etc.).
  - `migrations/`: Database migration scripts.
  - ...

- `scripts/`: Deployment and automation scripts.
  - `deploy.sh`: Deployment script.
  - ...

- `README.md`: Project documentation.
- `.gitignore`: Git ignore rules.


Machine Builders and System Integrators
------

For all the machine builders and system integrators that are looking for a modular, flexible and opensuorce HMI/SCADA that can run on any PC + Monitor or PC with embeeded Touch Monitor with 
Linux, Windows or MacOS.

Let's create the Next Generation of HMI for Industry 4.0 OpenSource Project.

Maintainers
------
Sergio Gallegos - Repo Owner  - sergiogallegos.net

Sponsors
------
- Not Sponsor at this time

License
------

OpenWebHMI is [MIT licensed](./LICENSE).
